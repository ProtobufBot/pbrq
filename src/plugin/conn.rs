use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures::{SinkExt, StreamExt};
use rand::seq::SliceRandom;
use rand::thread_rng;
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::http::{Request, Uri};
use tokio_tungstenite::tungstenite::Message;

use crate::api_handler::handle_api_frame;
use crate::bot::Bot;
use crate::error::{RCError, RCResult};
use crate::idl::pbbot;
use crate::idl::pbbot::frame::Data;

use super::pb_to_bytes::PbToBytes;
use super::Plugin;

pub struct PluginConnection {
    pub plugin: Plugin,
    url_index: AtomicU32,
    out_channel: broadcast::Sender<Message>,
    pub stop_channel: broadcast::Sender<()>,
    event_seq: AtomicU32,
}

impl PluginConnection {
    pub fn new(mut plugin: Plugin) -> Self {
        let mut rng = thread_rng();
        plugin.urls.shuffle(&mut rng);
        let (out_channel, _) = broadcast::channel(128);
        let (stop_channel, _) = broadcast::channel(1);
        Self {
            url_index: AtomicU32::new(0),
            plugin,
            out_channel,
            stop_channel,
            event_seq: AtomicU32::new(0),
        }
    }

    pub fn send_msg(&self, msg: Message) {
        self.out_channel.send(msg).ok();
    }

    pub fn stop(&self) {
        self.stop_channel.send(()).ok();
    }

    pub async fn start(self: &Arc<Self>, bot: &Arc<Bot>) -> RCResult<()> {
        let url_index = self.url_index.fetch_add(1, Ordering::Relaxed);
        let uri: Uri = self
            .plugin
            .urls
            .get(url_index as usize % self.plugin.urls.len())
            .cloned()
            .unwrap_or_default()
            .parse()
            .map_err(RCError::InvalidUri)?;
        let addr = format!(
            "{}:{}",
            uri.host().unwrap_or("localhost"),
            uri.port()
                .map(|p| p.to_string())
                .unwrap_or_else(|| "8081".into())
        );
        let stream = TcpStream::connect(addr).await.map_err(RCError::IO)?;
        tracing::info!("succeed to connect plugin [{}]", self.plugin.name);
        let req = Request::builder()
            .uri(uri)
            .header("x-self-id", bot.client.uin().await)
            .body(())
            .map_err(RCError::TungsteniteHttp)?;
        let (stream, _) = tokio_tungstenite::client_async(req, stream)
            .await
            .map_err(RCError::WS)?;
        let (mut w, mut r) = stream.split();
        let mut out_channel = self.out_channel.subscribe();
        let mut stop_channel = self.stop_channel.subscribe();

        let name = self.plugin.name.clone();
        loop {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(5))=>{
                    tracing::trace!("plugin send ping {}", name);
                    self.send_msg(Message::Ping("ping".as_bytes().to_vec()));
                }
                out_message = out_channel.recv() => {
                    w.send(out_message.map_err(|e|RCError::Other(format!("failed to recv out_message {}",e)))?).await.map_err(RCError::WS)?;
                }
                in_message = r.next()=>{
                    let msg=in_message.ok_or_else(||RCError::Other("failed to recv ws in_message".into()))??;
                    match msg{
                        Message::Binary(m) => {
                            let b=bot.clone();
                            let conn=self.clone();
                                let _: JoinHandle<Result<(),RCError>> =tokio::spawn(async move {
                                    let req = pbbot::Frame::from_bytes(&m).map_err(RCError::PB)?;
                                    // TODO check api permission
                                    let resp = handle_api_frame(&b,req).await;
                                    conn.send_msg(Message::Binary(resp.to_bytes()));
                                    Ok(())
                                });
                            }
                            Message::Ping(m) => {
                                self.send_msg(Message::Pong(m))
                            }
                            Message::Close(_) => {
                                return Err(RCError::Other("connection is closed".into()))
                            }
                            _=>{}
                        }
                }
                _ = stop_channel.recv() => {
                    return Err(RCError::Other("plugin is stopped".into()))
                }
            }
        }
    }

    pub async fn handle_event(&self, bot_id: i64, event: pbbot::frame::Data) {
        // TODO frame_type
        // TODO filter event type
        let frame = pbbot::Frame {
            bot_id,
            frame_type: match event {
                Data::PrivateMessageEvent(_) => pbbot::frame::FrameType::TPrivateMessageEvent,
                Data::GroupMessageEvent(_) => pbbot::frame::FrameType::TGroupMessageEvent,
                Data::GroupUploadNoticeEvent(_) => pbbot::frame::FrameType::TGroupUploadNoticeEvent,
                Data::GroupAdminNoticeEvent(_) => pbbot::frame::FrameType::TGroupAdminNoticeEvent,
                Data::GroupDecreaseNoticeEvent(_) => {
                    pbbot::frame::FrameType::TGroupDecreaseNoticeEvent
                }
                Data::GroupIncreaseNoticeEvent(_) => {
                    pbbot::frame::FrameType::TGroupIncreaseNoticeEvent
                }
                Data::GroupBanNoticeEvent(_) => pbbot::frame::FrameType::TGroupBanNoticeEvent,
                Data::FriendAddNoticeEvent(_) => pbbot::frame::FrameType::TFriendAddNoticeEvent,
                Data::GroupRecallNoticeEvent(_) => pbbot::frame::FrameType::TGroupRecallNoticeEvent,
                Data::FriendRecallNoticeEvent(_) => {
                    pbbot::frame::FrameType::TFriendRecallNoticeEvent
                }
                Data::FriendRequestEvent(_) => pbbot::frame::FrameType::TFriendRequestEvent,
                Data::GroupRequestEvent(_) => pbbot::frame::FrameType::TGroupRequestEvent,
                _ => pbbot::frame::FrameType::Tunknown,
            } as i32,
            echo: self.event_seq.fetch_add(1, Ordering::Relaxed).to_string(),
            ok: true,
            data: Some(event),
            extra: Default::default(),
        };
        self.send_msg(Message::Binary(frame.to_bytes()));
    }
}
