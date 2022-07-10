use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use cached::Cached;
use ricq::client::NetworkStatus;
use ricq::handler::QEvent;
use ricq::Client;
use ricq_core::structs::GroupMemberPermission;
use tokio::sync::{broadcast, Mutex};

use crate::error::RCResult;
use crate::event::to_proto_event;
use crate::plugin::conn::PluginConnection;
use crate::plugin::Plugin;

pub mod bots;

pub struct Bot {
    pub client: Arc<Client>,
    pub plugin_connections: HashMap<String, Arc<PluginConnection>>,
    pub stop_channel: broadcast::Sender<()>,
    pub group_role_cache: Mutex<cached::TimedCache<(i64, i64), GroupMemberPermission>>,
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
            group_role_cache: Mutex::new(cached::TimedCache::with_lifespan(30)),
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

    pub async fn cached_group_role(
        &self,
        group_id: i64,
        user_id: i64,
    ) -> RCResult<GroupMemberPermission> {
        if let Some(role) = self
            .group_role_cache
            .lock()
            .await
            .cache_get(&(group_id, user_id))
            .cloned()
        {
            return Ok(role);
        }
        let admins = self.client.get_group_admin_list(group_id).await?;
        let user_permission = admins.get(&user_id).cloned().unwrap_or_default();
        let mut cache = self.group_role_cache.lock().await;
        for (uin, permission) in admins {
            cache.cache_set((group_id, uin), permission);
        }
        if cache.cache_misses().unwrap_or_default() > 100 {
            cache.flush();
            cache.cache_reset_metrics();
        }
        Ok(user_permission)
    }
}

impl Drop for Bot {
    fn drop(&mut self) {
        self.stop()
    }
}
