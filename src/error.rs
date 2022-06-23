use std::io;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use ricq::RQError;
use thiserror::Error;

pub type RCResult<T> = Result<T, RCError>;

#[derive(Error, Debug)]
pub enum RCError {
    #[error("other error {0}")]
    Other(String),
    #[error("none error {0}")]
    None(&'static str),
    #[error("timeout error")]
    Timeout,
    #[error("client_not_found error")]
    ClientNotFound,
    #[error("protocol_not_supported error")]
    ProtocolNotSupported,
    #[error("io error, {0}")]
    IO(#[from] io::Error),
    #[error("websocket error, {0}")]
    WS(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("pb decode error, {0}")]
    PB(#[from] prost::DecodeError),
    #[error("rq error, {0}")]
    RQ(#[from] RQError),
    #[error("reqwest error, {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("base64 decode error, {0}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("invalid uri error, {0}")]
    InvalidUri(#[from] tokio_tungstenite::tungstenite::http::uri::InvalidUri),
    #[error("tungstenite http error, {0}")]
    TungsteniteHttp(#[from] tokio_tungstenite::tungstenite::http::Error),
}

impl IntoResponse for RCError {
    fn into_response(self) -> Response {
        let code = match self {
            Self::ClientNotFound => StatusCode::BAD_REQUEST,
            Self::ProtocolNotSupported => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (code, self.to_string()).into_response()
    }
}
