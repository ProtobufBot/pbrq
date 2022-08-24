use std::sync::Arc;

use axum::Json;
use dashmap::DashMap;
use lazy_static::lazy_static;
use rand::{prelude::StdRng, SeedableRng};
use ricq::client::{Connector, DefaultConnector};
use ricq::{
    client::NetworkStatus,
    device::Device,
    ext::reconnect::{Credential, Password},
    handler::QEvent,
    version::{get_version, Protocol},
    Client, LoginDeviceLocked, LoginNeedCaptcha, LoginResponse,
};
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

use crate::bot::bots::on_login;
use crate::error::{RCError, RCResult};
use crate::handler::ConvertU8;

pub struct PasswordClient {
    pub client: Arc<Client>,
    pub login_response: LoginResponse,
    pub event_receiver: tokio::sync::broadcast::Receiver<QEvent>,
    pub network_join_handle: JoinHandle<()>,
    pub credential: Credential,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CreateClientReq {
    pub uin: i64,
    pub protocol: u8,
    pub password: String,
    pub device_seed: Option<u64>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SubmitTicketReq {
    pub uin: i64,
    pub protocol: u8,
    pub ticket: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestSmsReq {
    pub uin: i64,
    pub protocol: u8,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SubmitSmsReq {
    pub uin: i64,
    pub protocol: u8,
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
            LoginResponse::UnknownStatus(status) => {
                resp.state = "unknown".into();
                resp.message = Some(format!(
                    "status: {} message: {}",
                    status.status, status.message
                ));
            }
        };
        resp
    }
}

lazy_static! {
    static ref CLIENTS: DashMap<(i64, u8), PasswordClient> = Default::default();
}

pub async fn login(Json(req): Json<CreateClientReq>) -> RCResult<Json<PasswordLoginResp>> {
    let mut rand_seed = req.device_seed.unwrap_or(req.uin as u64);
    if rand_seed == 0 {
        rand_seed = req.uin as u64;
    }
    let device = Device::random_with_rng(&mut StdRng::seed_from_u64(rand_seed));
    let protocol = Protocol::from_u8(req.protocol);
    let (sender, receiver) = tokio::sync::broadcast::channel(10);
    let cli = Arc::new(Client::new(device, get_version(protocol.clone()), sender));
    let connector = DefaultConnector;
    let stream = connector.connect(&cli).await.map_err(RCError::IO)?;
    let c = cli.clone();
    let network_join_handle = tokio::spawn(async move { c.start(stream).await });
    tokio::task::yield_now().await;
    let mut resp = cli.password_login(req.uin, &req.password).await?;
    if let LoginResponse::DeviceLockLogin(_) = resp {
        resp = cli.device_lock_login().await.map_err(RCError::RQ)?;
    }
    let credential = Credential::Password(Password {
        uin: req.uin,
        password: req.password,
    });
    if let LoginResponse::Success(_) = resp {
        tracing::info!("login success: {} {:?}", req.uin, req.protocol);
        on_login(cli, receiver, credential, network_join_handle).await;
    } else if let Some(old) = CLIENTS.insert(
        (req.uin, protocol.to_u8()),
        PasswordClient {
            client: cli,
            login_response: resp.clone(),
            event_receiver: receiver,
            network_join_handle,
            credential,
        },
    ) {
        old.client.stop(NetworkStatus::Stop);
    }
    Ok(Json(PasswordLoginResp::from(resp)))
}

pub async fn submit_ticket(Json(req): Json<SubmitTicketReq>) -> RCResult<Json<PasswordLoginResp>> {
    let mut resp = CLIENTS
        .get(&(req.uin, req.protocol))
        .ok_or(RCError::ClientNotFound)?
        .client
        .submit_ticket(&req.ticket)
        .await
        .map_err(RCError::RQ)?;
    if let LoginResponse::DeviceLockLogin(_) = resp {
        resp = CLIENTS
            .get(&(req.uin, req.protocol))
            .ok_or(RCError::ClientNotFound)?
            .client
            .device_lock_login()
            .await
            .map_err(RCError::RQ)?;
    }
    if let LoginResponse::Success(_) = resp {
        if let Some(((uin, protocol), client)) = CLIENTS.remove(&(req.uin, req.protocol)) {
            tracing::info!("login success: {} {:?}", uin, Protocol::from_u8(protocol));
            on_login(
                client.client,
                client.event_receiver,
                client.credential,
                client.network_join_handle,
            )
            .await;
        } else {
            tracing::warn!("failed to remove client: {}", req.uin);
        }
    } else {
        CLIENTS
            .get_mut(&(req.uin, req.protocol))
            .ok_or(RCError::ClientNotFound)?
            .login_response = resp.clone();
    }
    Ok(Json(PasswordLoginResp::from(resp)))
}

pub async fn request_sms(Json(req): Json<RequestSmsReq>) -> RCResult<Json<PasswordLoginResp>> {
    let resp = CLIENTS
        .get(&(req.uin, req.protocol))
        .ok_or(RCError::ClientNotFound)?
        .client
        .request_sms()
        .await
        .map_err(RCError::RQ)?;
    CLIENTS
        .get_mut(&(req.uin, req.protocol))
        .ok_or(RCError::ClientNotFound)?
        .login_response = resp.clone();
    Ok(Json(PasswordLoginResp::from(resp)))
}

pub async fn submit_sms(Json(req): Json<SubmitSmsReq>) -> RCResult<Json<PasswordLoginResp>> {
    let mut resp = CLIENTS
        .get(&(req.uin, req.protocol))
        .ok_or(RCError::ClientNotFound)?
        .client
        .submit_sms_code(&req.sms)
        .await
        .map_err(RCError::RQ)?;
    if let LoginResponse::DeviceLockLogin(_) = resp {
        resp = CLIENTS
            .get(&(req.uin, req.protocol))
            .ok_or(RCError::ClientNotFound)?
            .client
            .device_lock_login()
            .await
            .map_err(RCError::RQ)?;
    }
    if let LoginResponse::Success(_) = resp {
        let cli = CLIENTS.remove(&(req.uin, req.protocol));
        if let Some(((uin, protocol), client)) = cli {
            tracing::info!("login success: {} {:?}", uin, Protocol::from_u8(protocol));
            on_login(
                client.client,
                client.event_receiver,
                client.credential,
                client.network_join_handle,
            )
            .await;
        } else {
            tracing::warn!("failed to remove client: {}", req.uin);
        }
    } else {
        CLIENTS
            .get_mut(&(req.uin, req.protocol))
            .ok_or(RCError::ClientNotFound)?
            .login_response = resp.clone();
    }
    Ok(Json(PasswordLoginResp::from(resp)))
}

#[derive(Default, Serialize)]
pub struct ListClientResp {
    pub clients: Vec<ListClientRespClient>,
}

#[derive(Default, Serialize)]
pub struct ListClientRespClient {
    pub uin: i64,
    pub protocol: u8,
    pub resp: PasswordLoginResp,
}

pub async fn list() -> RCResult<Json<ListClientResp>> {
    let mut clients = Vec::new();
    for c in CLIENTS.iter() {
        clients.push(ListClientRespClient {
            uin: c.key().0,
            protocol: c.client.version().await.protocol.to_u8(),
            resp: PasswordLoginResp::from(c.login_response.clone()),
        })
    }
    Ok(Json(ListClientResp { clients }))
}

#[derive(Default, Serialize, Deserialize)]
pub struct DeleteClientReq {
    pub uin: i64,
    pub protocol: u8,
}

#[derive(Default, Serialize, Deserialize)]
pub struct DeleteClientResp {}

pub async fn delete(Json(req): Json<DeleteClientReq>) -> RCResult<Json<DeleteClientResp>> {
    if let Some((_, cli)) = CLIENTS.remove(&(req.uin, req.protocol)) {
        cli.client.stop(NetworkStatus::Stop);
    }
    Ok(Json(DeleteClientResp {}))
}
