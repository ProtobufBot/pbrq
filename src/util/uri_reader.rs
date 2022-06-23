use std::time::Duration;

use tokio::io::AsyncReadExt;

use crate::error::{RCError, RCResult};

pub async fn get_binary(uri: &str) -> RCResult<Vec<u8>> {
    if let Some(s) = uri.strip_prefix("base64://") {
        return base64::decode(s).map_err(RCError::Base64Decode);
    }
    if let Some(file) = uri.strip_prefix("file://") {
        return read_binary_file(file).await;
    }
    if uri.starts_with("https") || uri.starts_with("http") {
        return http_get(uri).await;
    }
    read_binary_file(uri).await
}

pub async fn http_get(uri: &str) -> RCResult<Vec<u8>> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?
        .get(uri)
        .send()
        .await?
        .bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(RCError::from)
}

pub async fn read_binary_file(path: &str) -> RCResult<Vec<u8>> {
    let mut f = tokio::fs::File::open(path).await.map_err(RCError::IO)?;
    let mut b = Vec::new();
    f.read_to_end(&mut b).await.map_err(RCError::IO)?;
    Ok(b)
}
