use std::sync::Arc;

use axum::Json;
use dashmap::DashMap;
use lazy_static::lazy_static;
use rand::{prelude::StdRng, SeedableRng};
use rs_qq::handler::QEvent;
use rs_qq::{
    device::Device,
    version::{get_version, Protocol},
    Client, LoginDeviceLocked, LoginNeedCaptcha, LoginResponse,
};
use serde::{Deserialize, Serialize};

use crate::bot::bots::on_login;
use crate::error::{RCError, RCResult};

pub struct PasswordClient {
    pub client: Arc<Client>,
    pub login_response: LoginResponse,
    pub event_receiver: tokio::sync::broadcast::Receiver<QEvent>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CreateClientReq {
    pub uin: i64,
    pub password: String,
    pub device_seed: Option<u64>,
    pub client_protocol: Option<i32>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SubmitTicketReq {
    pub uin: i64,
    pub ticket: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SubmitSmsReq {
    pub uin: i64,
    pub sms: String,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct PasswordLoginResp {
    pub state: String,
    pub captcha_url: Option<String>,
    pub verify_url: Option<String>,
    pub sms_phone: Option<String>,
    pub message: Option<String>,
}

impl From<LoginResponse> for PasswordLoginResp {
    fn from(login_response: LoginResponse) -> Self {
        let mut resp = PasswordLoginResp::default();
        match login_response {
            LoginResponse::Success(_) => {
                resp.state = "success".into();
            }
            LoginResponse::NeedCaptcha(LoginNeedCaptcha { ref verify_url, .. }) => {
                resp.state = "need_captcha".into();
                resp.captcha_url = verify_url.clone();
            }
            LoginResponse::AccountFrozen => {
                resp.state = "account_frozen".into();
            }
            LoginResponse::DeviceLocked(LoginDeviceLocked {
                ref verify_url,
                ref message,
                ref sms_phone,
                ..
            }) => {
                resp.state = "device_locked".into();
                resp.verify_url = verify_url.clone();
                resp.sms_phone = sms_phone.clone();
                resp.message = message.clone();
            }
            LoginResponse::TooManySMSRequest => {
                resp.state = "too_many_sms_request".into();
            }
            LoginResponse::DeviceLockLogin(_) => {
                resp.state = "device_lock_login".into();
            }
            LoginResponse::UnknownStatus(_) => {
                resp.state = "unknown".into();
            }
        };
        resp
    }
}

lazy_static! {
    static ref CLIENTS: DashMap<i64, PasswordClient> = Default::default();
}

pub async fn login(Json(req): Json<CreateClientReq>) -> RCResult<Json<PasswordLoginResp>> {
    let rand_seed = req.device_seed.unwrap_or(req.uin as u64);
    let device = Device::random_with_rng(&mut StdRng::seed_from_u64(rand_seed));
    let protocol = match req.client_protocol.unwrap_or(5) {
        1 => Protocol::AndroidPhone,
        2 => Protocol::AndroidWatch,
        3 => Protocol::MacOS,
        4 => Protocol::IPad,
        5 => Protocol::QiDian,
        _ => Protocol::IPad,
    };
    let (sender, receiver) = tokio::sync::broadcast::channel(10);
    let cli = Arc::new(Client::new(device, get_version(protocol), sender));
    let stream = tokio::net::TcpStream::connect(cli.get_address())
        .await
        .map_err(RCError::IO)?;
    let c = cli.clone();
    tokio::spawn(async move { c.start(stream).await });
    tokio::task::yield_now().await;
    let mut resp = cli.password_login(req.uin, &req.password).await?;
    if let LoginResponse::DeviceLockLogin(_) = resp {
        resp = cli.device_lock_login().await.map_err(RCError::RQ)?;
    }
    if let LoginResponse::Success(_) = resp {
        tracing::info!("login success: {}", req.uin);
        on_login(cli, receiver).await;
    } else {
        CLIENTS.insert(
            req.uin,
            PasswordClient {
                client: cli,
                login_response: resp.clone(),
                event_receiver: receiver,
            },
        );
    }
    return Ok(Json(PasswordLoginResp::from(resp)));
}

pub async fn submit_ticket(Json(req): Json<SubmitTicketReq>) -> RCResult<Json<PasswordLoginResp>> {
    let mut cli = CLIENTS.get_mut(&req.uin).ok_or(RCError::ClientNotFound)?;

    let mut resp = cli
        .client
        .submit_ticket(&req.ticket)
        .await
        .map_err(RCError::RQ)?;
    if let LoginResponse::DeviceLockLogin(_) = resp {
        resp = cli.client.device_lock_login().await.map_err(RCError::RQ)?;
    }
    if let LoginResponse::Success(_) = resp {
        if let Some((uin, client)) = CLIENTS.remove(&req.uin) {
            tracing::info!("login success: {}", uin);
            on_login(client.client, client.event_receiver).await;
        } else {
            tracing::warn!("failed to remove client: {}", req.uin);
        }
    } else {
        cli.login_response = resp.clone();
    }
    return Ok(Json(PasswordLoginResp::from(resp)));
}

pub async fn submit_sms(Json(req): Json<SubmitSmsReq>) -> RCResult<Json<PasswordLoginResp>> {
    let mut cli = CLIENTS.get_mut(&req.uin).ok_or(RCError::ClientNotFound)?;

    let mut resp = cli
        .client
        .submit_sms_code(&req.sms)
        .await
        .map_err(RCError::RQ)?;
    if let LoginResponse::DeviceLockLogin(_) = resp {
        resp = cli.client.device_lock_login().await.map_err(RCError::RQ)?;
    }
    if let LoginResponse::Success(_) = resp {
        if let Some((uin, client)) = CLIENTS.remove(&req.uin) {
            tracing::info!("login success: {}", uin);
            on_login(client.client, client.event_receiver).await;
        } else {
            tracing::warn!("failed to remove client: {}", req.uin);
        }
    } else {
        cli.login_response = resp.clone();
    }
    return Ok(Json(PasswordLoginResp::from(resp)));
}

#[derive(Default, Serialize)]
pub struct ListClientResp {
    pub clients: Vec<ListClientRespClient>,
}

#[derive(Default, Serialize)]
pub struct ListClientRespClient {
    pub uin: i64,
    pub resp: PasswordLoginResp,
}

pub async fn list() -> RCResult<Json<ListClientResp>> {
    Ok(Json(ListClientResp {
        clients: CLIENTS
            .iter()
            .map(|c| ListClientRespClient {
                uin: *c.key(),
                resp: PasswordLoginResp::from(c.login_response.clone()),
            })
            .collect(),
    }))
}

#[derive(Default, Serialize, Deserialize)]
pub struct DeleteClientReq {
    pub uin: i64,
}

#[derive(Default, Serialize, Deserialize)]
pub struct DeleteClientResp {}

pub async fn delete(Json(req): Json<DeleteClientReq>) -> RCResult<Json<DeleteClientResp>> {
    if let Some((_, cli)) = CLIENTS.remove(&req.uin) {
        cli.client.stop();
    }
    Ok(Json(DeleteClientResp {}))
}
