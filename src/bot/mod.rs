use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use ricq::client::NetworkStatus;
use ricq::handler::QEvent;
use ricq::Client;
use tokio::sync::broadcast;

use crate::event::to_proto_event;
use crate::plugin::conn::PluginConnection;
use crate::plugin::Plugin;

pub mod bots;

pub struct Bot {
    pub client: Arc<Client>,
    pub plugin_connections: HashMap<String, Arc<PluginConnection>>,
    pub stop_channel: broadcast::Sender<()>,
}

impl Bot {
    pub fn new(client: Arc<Client>, plugins: Vec<Plugin>) -> Self {
        let (stop_channel, _) = broadcast::channel(1);
        Self {
            client,
            stop_channel,
            plugin_connections: plugins
                .into_iter()
                .map(|p| (p.name.clone(), Arc::new(PluginConnection::new(p))))
                .collect(),
        }
    }

    // 开始处理 event
    pub fn start_handle_event(self: &Arc<Self>, mut event_receiver: broadcast::Receiver<QEvent>) {
        let bot = self.clone();
        let mut stop_signal = self.stop_channel.subscribe();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    e = event_receiver.recv() => {
                        if let Ok(e) = e {
                            if let Some(e) = to_proto_event(&bot, e).await {
                                for (_, plugin) in bot.plugin_connections.iter() {
                                    // TODO convert event
                                    plugin.handle_event(bot.client.uin().await,e.clone()).await;
                                }
                            }
                        }
                    }
                    _ = stop_signal.recv() => {
                        break;
                    }
                }
            }
        });
    }

    // 连接插件地址
    pub fn start_plugins(self: &Arc<Self>) {
        for (name, p) in self.plugin_connections.iter() {
            let name = name.clone();
            let plugin = p.clone();

            let bot = self.clone();
            tokio::spawn(async move {
                // TODO stop?
                let mut stop_signal = plugin.stop_channel.subscribe();
                loop {
                    tokio::select! {
                        reason = plugin.start(&bot) => {
                            // 阻塞到断开
                            tracing::warn!("plugin [{}] error: {:?}", name, reason);
                        }
                        _ = stop_signal.recv() => {
                            break;
                        }
                    }
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            });
        }
    }

    // 停止机器人，暂时无法重启
    pub fn stop(&self) {
        self.stop_channel.send(()).ok();
        for (_, p) in self.plugin_connections.iter() {
            p.stop();
        }
        self.client.stop(NetworkStatus::Stop);
    }
}

impl Drop for Bot {
    fn drop(&mut self) {
        self.stop()
    }
}
