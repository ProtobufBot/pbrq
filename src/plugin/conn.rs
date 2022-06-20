use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
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
    pub running: AtomicBool,
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
            running: AtomicBool::new(false),
            event_seq: AtomicU32::new(0),
        }
    }

    pub fn send_msg(&self, msg: Message) {
        self.out_channel.send(msg).ok();
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
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
        self.running.store(true, Ordering::Relaxed);
        while self.running.load(Ordering::Relaxed) {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(5))=>{
                    tracing::trace!("plugin send ping {}", name);
                    self.send_msg(Message::Ping("ping".as_bytes().to_vec()));
                }
                out_message = out_channel.recv() => {
                    if let Ok(out_message)=out_message{
                        w.send(out_message).await.map_err(RCError::WS)?;
                    }else {
                        break;
                    }
                }
                in_message = r.next()=>{
                    if let Some(Ok(msg))=in_message{
                        match msg{
                            Message::Binary(m) => {
                                let b=bot.clone();
                                let conn=self.clone();
                                let start=std::time::Instant::now();
                                let r:u32=rand::random();
                                let uin=b.client.uin().await;
                                if uin==2490390725{
                                    tracing::info!("handle_api {} start",r)
                                }
                                let _: JoinHandle<Result<(),RCError>> =tokio::spawn(async move {
                                    let req = pbbot::Frame::from_bytes(&m).map_err(RCError::PB)?;
                                    // TODO check api permission
                                    let resp = handle_api_frame(&b,req).await;
                                    conn.send_msg(Message::Binary(resp.to_bytes()));
                                    Ok(())
                                });
                                if uin==2490390725{
                                    tracing::info!("handle_api {} end {:?}",r,start.elapsed())
                                }
                            }
                            Message::Ping(m) => {
                                self.send_msg(Message::Pong(m))
                            }
                            Message::Close(_) => {
                                break;
                            }
                            _=>{}
                        }
                    }else{
                        break;
                    }
                }
                _ = stop_channel.recv() => {
                    break;
                }
            }
        }
        self.running.store(false, Ordering::Relaxed);
        Ok(())
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
