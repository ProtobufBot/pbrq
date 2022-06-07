use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use lazy_static::lazy_static;
use ricq::{
    ext::common::after_login,
    ext::reconnect::{auto_reconnect, Credential, DefaultConnector},
    handler::QEvent,
    Client,
};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::bot::Bot;
use crate::plugin::storage::{load_plugins, PLUGIN_PATH};

lazy_static! {
    static ref BOTS: DashMap<i64, Arc<Bot>> = Default::default();
}

pub async fn on_login(
    client: Arc<Client>,
    event_receiver: broadcast::Receiver<QEvent>,
    credential: Credential,
    network_join_handle: JoinHandle<()>,
) {
    let uin = client.uin().await;
    after_login(&client).await;
    let bot = Arc::new(Bot::new(
        client.clone(),
        load_plugins(PLUGIN_PATH)
            .await
            .expect("failed to load plugins"),
    ));
    if let Some(old) = BOTS.insert(uin, bot.clone()) {
        old.stop();
    }
    bot.start_plugins();
    bot.start_handle_event(event_receiver);
    tokio::spawn(async move {
        network_join_handle.await.ok();
        auto_reconnect(
            client,
            credential,
            Duration::from_secs(10),
            10,
            DefaultConnector,
        )
        .await;
    });
}

pub async fn delete_bot(uin: i64) {
    if let Some((_, bot)) = BOTS.remove(&uin) {
        bot.stop();
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BotInfo {
    pub uin: i64,
    pub nick: String,
    pub status: u8,
}

pub async fn list_bot() -> Vec<BotInfo> {
    let mut infos = Vec::new();
    for bot in BOTS.iter() {
        infos.push(BotInfo {
            uin: *bot.key(),
            nick: bot.client.account_info.read().await.nickname.clone(),
            status: bot.client.get_status(),
        })
    }
    infos
}
