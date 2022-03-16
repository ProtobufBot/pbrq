use std::path::Path;

use crate::plugin::Plugin;

pub const PLUGIN_PATH: &str = "plugins";

async fn ensure_path(path: &str) -> std::io::Result<()> {
    tokio::fs::create_dir_all(path).await
}

pub async fn load_plugins(path: &str) -> std::io::Result<Vec<Plugin>> {
    ensure_path(PLUGIN_PATH).await.ok();
    let mut dir = tokio::fs::read_dir(path).await?;
    let mut plugins = Vec::new();
    while let Some(e) = dir.next_entry().await? {
        if e.path().extension().unwrap_or_default().eq("json") {
            let content = tokio::fs::read(e.path()).await?;
            if let Ok(mut plugin) = serde_json::from_slice::<Plugin>(&content) {
                plugin.name = e
                    .file_name()
                    .to_str()
                    .unwrap_or_default()
                    .trim_end_matches(".json")
                    .to_string();
                plugins.push(plugin);
            }
        }
    }
    Ok(plugins)
}

pub async fn save_plugins(path: &str, plugins: Vec<Plugin>) -> std::io::Result<()> {
    for plugin in plugins {
        save_plugin(path, &plugin).await?;
    }
    Ok(())
}

pub async fn save_plugin(path: &str, plugin: &Plugin) -> std::io::Result<()> {
    ensure_path(PLUGIN_PATH).await.ok();
    tokio::fs::write(
        Path::new(path).join(format!("{}.json", plugin.name)),
        serde_json::to_string(&plugin).unwrap_or_default(),
    )
    .await
}

pub async fn delete_plugin(path: &str, name: &str) -> std::io::Result<()> {
    ensure_path(PLUGIN_PATH).await.ok();
    tokio::fs::remove_file(Path::new(path).join(format!("{}.json", name))).await
}
