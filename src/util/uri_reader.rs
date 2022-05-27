use hyper::{Body, Client, Request};
use hyper_tls::HttpsConnector;
use tokio::io::AsyncReadExt;

use crate::error::{RCError, RCResult};

pub async fn get_binary(uri: &str) -> RCResult<Vec<u8>> {
    if let Some(s) = uri.strip_prefix("base64://") {
        return base64::decode(s).map_err(RCError::Base64Decode);
    }
    if let Some(file) = uri.strip_prefix("file://") {
        let mut f = tokio::fs::File::open(file).await.map_err(RCError::IO)?;
        let mut b = Vec::new();
        f.read_buf(&mut b).await.map_err(RCError::IO)?;
        return Ok(b);
    }
    if uri.starts_with("https") || uri.starts_with("http") {
        return http_get(uri).await;
    }
    Err(RCError::Other("protocol is not supported".into()))
}

pub async fn http_get(uri: &str) -> RCResult<Vec<u8>> {
    let cli = Client::builder().build::<_, Body>(HttpsConnector::new());
    let req = Request::builder().uri(uri).body(Body::empty())?;
    let resp = cli.request(req).await?;
    hyper::body::to_bytes(resp.into_body())
        .await
        .map(|b| b.to_vec())
        .map_err(crate::error::RCError::Hyper)
}
