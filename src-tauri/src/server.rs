use core::fmt;
use std::marker::PhantomData;

use log::warn;
use reqwest::{RequestBuilder, Response};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::{endpoint::Endpoint, patcher};

const NETWORK_ERROR: &str = "launcher-network-error";

pub enum Error {
    Cancellation,
    Server(u16, String),
    Backend(String),
}

impl Error {
    pub fn into_frontend(self) -> String {
        match self {
            Self::Cancellation => "".into(),
            Self::Server(_, msg) | Self::Backend(msg) => msg,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancellation => write!(f, "request cancelled"),
            Self::Server(status, msg) => write!(f, "server error {}: {}", status, msg),
            Self::Backend(msg) => write!(f, "backend error: {}", msg),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BannerData {
    pub src: String,
    pub link: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MessageData {
    pub message: String,
    pub date: i32,
    pub link: String,
    pub kind: MessageKind,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, Clone)]
#[repr(u8)]
pub enum MessageKind {
    Default,
    New,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LinkData {
    pub name: String,
    pub link: String,
    pub icon: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LauncherResponse {
    pub banners: Vec<BannerData>,
    pub messages: Vec<MessageData>,
    pub links: Vec<LinkData>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserData {
    pub token_id: u32,
    pub token: String,
    pub rights: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CharacterData {
    pub id: u32,
    pub name: String,
    pub is_female: bool,
    pub weapon: u32,
    pub hr: u32,
    pub gr: u32,
    pub last_login: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MezFesData {
    pub id: u32,
    pub start: u32,
    pub end: u32,
    pub solo_tickets: u32,
    pub group_tickets: u32,
    pub stalls: Vec<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthRequest<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub current_ts: u32,
    pub expiry_ts: u32,
    pub entrance_count: u32,
    pub notices: Vec<String>,
    pub user: UserData,
    pub characters: Vec<CharacterData>,
    pub mez_fez: Option<MezFesData>,
    pub patch_server: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmptyResponse {}

pub struct PatcherResponse {
    pub etag: String,
    pub content: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UserRequest<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CharacterRequest<'a> {
    token: &'a str,
    char_id: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TokenRequest<'a> {
    token: &'a str,
}

async fn send(request: RequestBuilder, cancel: CancellationToken) -> Result<Response, Error> {
    let resp = select! {
        _ = cancel.cancelled() => return Err(Error::Cancellation),
        resp = request.send() => resp,
    };
    let resp = resp.map_err(|e| {
        warn!("request connection failed: {}", e);
        Error::Backend(NETWORK_ERROR.into())
    })?;
    let status = resp.status().as_u16();
    if status >= 400 {
        warn!("request status error: {}", status);
        let is_text = resp
            .headers()
            .get("Content-Type")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.starts_with("text/plain"))
            .unwrap_or(false);
        let message = if is_text {
            resp.text().await.unwrap_or(NETWORK_ERROR.into())
        } else {
            NETWORK_ERROR.into()
        };
        return Err(Error::Server(status, message));
    }
    Ok(resp)
}

pub struct JsonRequest<T: DeserializeOwned> {
    request: RequestBuilder,
    cancel: CancellationToken,
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned> JsonRequest<T> {
    fn new(request: RequestBuilder, cancel: CancellationToken) -> Self {
        Self {
            request,
            cancel,
            _phantom: PhantomData,
        }
    }

    pub async fn send(self) -> Result<T, Error> {
        let resp = send(self.request, self.cancel).await?;
        resp.json().await.map_err(|e| {
            warn!("request parsing failed: {}", e);
            Error::Backend(NETWORK_ERROR.into())
        })
    }
}

pub struct PatcherRequest {
    request: RequestBuilder,
    cancel: CancellationToken,
}

impl PatcherRequest {
    pub async fn send(self) -> Result<Option<PatcherResponse>, Error> {
        let resp = send(self.request, self.cancel).await?;
        let status = resp.status().as_u16();
        if status == 304 {
            return Ok(None);
        }
        let etag = resp
            .headers()
            .get("ETag")
            .and_then(|v| v.to_str().ok())
            .ok_or(Error::Server(status, patcher::NETWORK_ERROR.into()))?
            .to_owned();
        let content = resp.text().await.map_err(|e| {
            warn!("failed to read body of patcher request {}", e);
            Error::Server(status, patcher::NETWORK_ERROR.into())
        })?;
        Ok(Some(PatcherResponse { etag, content }))
    }
}

pub fn simple_request<T: DeserializeOwned>(
    client: &reqwest::Client,
    cancel: CancellationToken,
    url: &str,
) -> JsonRequest<T> {
    let req = client.get(url);
    JsonRequest::new(req, cancel)
}

pub fn launcher_request(
    client: &reqwest::Client,
    cancel: CancellationToken,
    endpoint: &Endpoint,
) -> JsonRequest<LauncherResponse> {
    let req = client.get(endpoint.get_url("/launcher"));
    JsonRequest::new(req, cancel)
}

pub fn login_request(
    client: &reqwest::Client,
    cancel: CancellationToken,
    endpoint: &Endpoint,
    username: &str,
    password: &str,
) -> JsonRequest<AuthResponse> {
    let user_request = UserRequest { username, password };
    let req = client.post(endpoint.get_url("/login")).json(&user_request);
    JsonRequest::new(req, cancel)
}

pub fn register_request(
    client: &reqwest::Client,
    cancel: CancellationToken,
    endpoint: &Endpoint,
    username: &str,
    password: &str,
) -> JsonRequest<AuthResponse> {
    let user_request = UserRequest { username, password };
    let req = client
        .post(endpoint.get_url("/register"))
        .json(&user_request);
    JsonRequest::new(req, cancel)
}

pub fn delete_character_request(
    client: &reqwest::Client,
    cancel: CancellationToken,
    endpoint: &Endpoint,
    token: &str,
    character_id: i32,
) -> JsonRequest<EmptyResponse> {
    let delete_request = CharacterRequest {
        token,
        char_id: character_id,
    };
    let req = client
        .post(endpoint.get_url("/character/delete"))
        .json(&delete_request);
    JsonRequest::new(req, cancel)
}

pub fn create_character_request(
    client: &reqwest::Client,
    cancel: CancellationToken,
    endpoint: &Endpoint,
    token: &str,
) -> JsonRequest<CharacterData> {
    let token_req = TokenRequest { token };
    let req = client
        .post(endpoint.get_url("/character/create"))
        .json(&token_req);
    JsonRequest::new(req, cancel)
}

pub fn export_save_request(
    client: &reqwest::Client,
    cancel: CancellationToken,
    endpoint: &Endpoint,
    token: &str,
    character_id: i32,
) -> JsonRequest<Value> {
    let export_request = CharacterRequest {
        token,
        char_id: character_id,
    };
    let req = client
        .post(endpoint.get_url("/character/export"))
        .json(&export_request);
    JsonRequest::new(req, cancel)
}

pub fn patcher_request(
    client: &reqwest::Client,
    cancel: CancellationToken,
    url: &str,
    client_etag: &str,
) -> PatcherRequest {
    let request = client
        .get(format!("{}/check", url))
        .header("If-None-Match", client_etag);
    PatcherRequest { request, cancel }
}
