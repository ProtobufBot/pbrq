use std::collections::HashMap;

use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::{RCError, RCResult};
use crate::plugin::{
    storage::{delete_plugin, load_plugins, save_plugin, PLUGIN_PATH},
    Plugin,
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ListPluginResp {
    pub plugins: HashMap<String, Plugin>,
}

pub async fn list() -> RCResult<Json<ListPluginResp>> {
    Ok(Json(ListPluginResp {
        plugins: load_plugins(PLUGIN_PATH)
            .await
            .map_err(RCError::IO)?
            .into_iter()
            .map(|p| (p.name.clone(), p))
            .collect(),
    }))
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SavePluginReq {
    pub name: String,
    pub plugin: Plugin,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SavePluginResp {}

pub async fn save(Json(mut req): Json<SavePluginReq>) -> RCResult<Json<SavePluginResp>> {
    req.plugin.name = req.name.clone();
    save_plugin(PLUGIN_PATH, &req.plugin)
        .await
        .map(|_| Json(SavePluginResp {}))
        .map_err(RCError::IO)
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeletePluginReq {
    pub name: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeletePluginResp {}

pub async fn delete(Json(req): Json<DeletePluginReq>) -> RCResult<Json<DeletePluginResp>> {
    delete_plugin(PLUGIN_PATH, &req.name)
        .await
        .map(|_| Json(DeletePluginResp {}))
        .map_err(RCError::IO)
}
