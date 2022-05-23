use std::sync::atomic::Ordering;
use std::sync::Arc;

use dashmap::DashMap;
use lazy_static::lazy_static;
use ricq::ext::common::after_login;
use ricq::handler::QEvent;
use ricq::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::bot::Bot;
use crate::plugin::storage::{load_plugins, PLUGIN_PATH};

lazy_static! {
    static ref BOTS: DashMap<i64, Arc<Bot>> = Default::default();
}

pub async fn on_login(client: Arc<Client>, event_receiver: broadcast::Receiver<QEvent>) {
    let uin = client.uin().await;
    after_login(&client).await;
    // TODO auto reconnect
    let bot = Arc::new(Bot::new(
        client,
        load_plugins(PLUGIN_PATH)
            .await
            .expect("failed to load plugins"),
    ));
    BOTS.insert(uin, bot.clone());
    bot.start_plugins();
    bot.start_handle_event(event_receiver);
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
    pub running: bool,
}

pub async fn list_bot() -> Vec<BotInfo> {
    let mut infos = Vec::new();
    for bot in BOTS.iter() {
        infos.push(BotInfo {
            uin: *bot.key(),
            nick: bot.client.account_info.read().await.nickname.clone(),
            running: bot.client.running.load(Ordering::Relaxed),
        })
    }
    infos
}
