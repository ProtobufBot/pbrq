use axum::Json;
use serde::{Deserialize, Serialize};

use crate::bot::bots::{delete_bot, list_bot, BotInfo};
use crate::error::RCResult;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ListBotResp {
    pub bots: Vec<BotInfo>,
}

pub async fn list() -> RCResult<Json<ListBotResp>> {
    Ok(Json(ListBotResp {
        bots: list_bot().await,
    }))
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeleteBotReq {
    uin: i64,
    protocol: u8,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeleteBotResp {}

pub async fn delete(Json(req): Json<DeleteBotReq>) -> RCResult<Json<DeleteBotResp>> {
    delete_bot(req.uin, req.protocol).await;
    Ok(Json(DeleteBotResp {}))
}
