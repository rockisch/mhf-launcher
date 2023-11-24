#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![feature(result_option_inspect)]
#![feature(iterator_try_collect)]
#![feature(absolute_path)]

mod config;
mod endpoint;
mod patcher;
mod server;
mod store;
mod user;

use std::{
    collections::HashMap,
    fs::File,
    path::{self, Path, PathBuf},
    sync::Arc,
};

use log::{error, info, warn};
use mhf_iel::MhfConfig;
use serde::Serialize;
use serde_json::Value;
use server::{AuthResponse, JsonRequest, LauncherResponse, MessageData, PatcherResponse};
use store::StoreHelper;
use tauri::{async_runtime::Mutex, PhysicalSize};
use tauri::{Manager, Window};
use tauri_plugin_log::LogTarget;
use tauri_plugin_store::StoreBuilder;
use tokio_util::sync::CancellationToken;
use user::{UserData, UserManager};

use crate::config::{CLASSIC_STYLE, DEFAULT_MESSAGELIST_URL, DEFAULT_SERVERLIST_URL, MODERN_STYLE};
use crate::endpoint::{Endpoint, EndpointConfig, EndpointVecExt};

enum ExitSignal {
    RunGame(u32, bool),
}

#[derive()]
struct TauriState {
    client: reqwest::Client,
    state_sync: Arc<Mutex<TauriStateSync>>,
}

#[derive(Default)]
struct TauriStateSync {
    style: u32,
    locale: String,
    store: StoreHelper,
    endpoints: Vec<Endpoint>,
    remote_endpoints: Vec<Endpoint>,
    remote_endpoints_config: HashMap<String, EndpointConfig>,
    current_endpoint: Endpoint,
    user_manager: UserManager,
    game_folder: Option<PathBuf>,
    last_char_id: Option<u32>,
    serverlist_url: String,
    messagelist_url: String,

    exit_reason: Option<ExitSignal>,

    auth_resp: Option<AuthResponse>,
    launcher_resp: Option<LauncherResponse>,
    patcher_resp: Option<PatcherResponse>,

    cancel_shared: CancellationToken,
    cancel_launcher: CancellationToken,
    cancel_serverlist: CancellationToken,
    cancel_messagelist: CancellationToken,
}

impl TauriStateSync {
    fn first_endpoint(&self) -> Option<&Endpoint> {
        self.remote_endpoints
            .first()
            .or_else(|| self.endpoints.first())
    }

    fn contains_endpoint(&self, endpoint: &Endpoint) -> bool {
        if self.current_endpoint.is_remote {
            self.remote_endpoints.contains(endpoint)
        } else {
            self.endpoints.contains(endpoint)
        }
    }

    fn ensure_current_endpoint(&mut self) -> Result<(), &'static str> {
        let endpoints = if self.current_endpoint.is_remote {
            &self.remote_endpoints
        } else {
            &self.endpoints
        };

        self.current_endpoint = endpoints
            .iter()
            .find(|&e| e == &self.current_endpoint)
            .or_else(|| self.first_endpoint())
            .ok_or("internal-error")?
            .clone();
        Ok(())
    }

    fn auth_resp_err(&self) -> Result<&AuthResponse, &str> {
        self.auth_resp.as_ref().ok_or("internal-error")
    }

    fn effective_folder(&self) -> PathBuf {
        self.current_endpoint
            .game_folder
            .as_ref()
            .or(self.game_folder.as_ref())
            .cloned()
            .unwrap_or_else(|| std::env::current_dir().unwrap())
    }
}

#[derive(Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EndpointsPayload {
    endpoints: Option<Vec<Endpoint>>,
    remote_endpoints: Option<Vec<Endpoint>>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthPayload {
    response: AuthResponse,
    has_patch: bool,
}

#[derive(Serialize, Clone)]
pub struct LogPayload {
    level: String,
    message: String,
}

impl LogPayload {
    fn error(message: impl Into<String>) -> Self {
        Self {
            level: "error".into(),
            message: message.into(),
        }
    }

    fn warning(message: impl Into<String>) -> Self {
        Self {
            level: "warning".into(),
            message: message.into(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InitialDataPayload {
    style: u32,
    locale: String,
    endpoints: Vec<Endpoint>,
    remote_endpoints: Vec<Endpoint>,
    current_endpoint: Endpoint,
    username: String,
    password: String,
    remember_me: bool,
    game_folder: Option<PathBuf>,
    current_folder: PathBuf,
    last_char_id: Option<u32>,
    serverlist_url: String,
    messagelist_url: String,
}

#[tauri::command]
async fn initial_data(state: tauri::State<'_, TauriState>) -> Result<InitialDataPayload, ()> {
    let state_sync = state.state_sync.lock().await;
    let (userdata, password) = state_sync.user_manager.get(&state_sync.current_endpoint);
    Ok(InitialDataPayload {
        style: state_sync.style,
        endpoints: state_sync.endpoints.clone(),
        remote_endpoints: state_sync.remote_endpoints.clone(),
        current_endpoint: state_sync.current_endpoint.clone(),
        username: userdata.username,
        password,
        remember_me: userdata.remember_me,
        game_folder: state_sync.game_folder.clone(),
        current_folder: std::env::current_dir().unwrap(),
        locale: state_sync.locale.clone(),
        last_char_id: state_sync.last_char_id,
        serverlist_url: state_sync.serverlist_url.clone(),
        messagelist_url: state_sync.messagelist_url.clone(),
    })
}

#[tauri::command]
async fn set_style(
    mut window: Window,
    state: tauri::State<'_, TauriState>,
    style: u32,
) -> Result<(), String> {
    let mut state_sync = state.state_sync.lock().await;
    state_sync.style = style;
    state_sync.store.with(|s| s.set("style", style));
    handle_style(&mut window, style);
    Ok(())
}

#[tauri::command]
async fn set_locale(state: tauri::State<'_, TauriState>, locale: String) -> Result<(), String> {
    let mut state_sync = state.state_sync.lock().await;
    state_sync.locale = locale.clone();
    state_sync.store.with(|s| s.set("locale", locale));
    Ok(())
}

#[tauri::command]
async fn set_endpoints(
    state: tauri::State<'_, TauriState>,
    endpoints: Vec<Endpoint>,
) -> Result<Endpoint, String> {
    endpoints.check_valid()?;
    let mut state_sync = state.state_sync.lock().await;
    state_sync.endpoints = endpoints;
    if !state_sync.current_endpoint.is_remote {
        state_sync.ensure_current_endpoint()?;
    }
    let endpoints = state_sync.endpoints.clone();
    let current_endpoint = state_sync.current_endpoint.clone();
    state_sync.store.with(|s| {
        s.set("endpoints", endpoints);
        s.set("current_endpoint", current_endpoint);
    });
    Ok(state_sync.current_endpoint.clone())
}

#[tauri::command]
async fn set_remote_endpoints(
    state: tauri::State<'_, TauriState>,
    endpoints: Vec<Endpoint>,
) -> Result<Endpoint, String> {
    endpoints.check_valid()?;
    let state_sync = &mut *state.state_sync.lock().await;
    state_sync.remote_endpoints = endpoints;
    if state_sync.current_endpoint.is_remote {
        state_sync.ensure_current_endpoint()?;
    }
    state_sync
        .remote_endpoints
        .update_config(&mut state_sync.remote_endpoints_config);
    let current_endpoint = state_sync.current_endpoint.clone();
    let remote_endpoints_config = state_sync.remote_endpoints_config.clone();
    state_sync.store.with(|s| {
        s.set("remote_endpoints_config", remote_endpoints_config);
        s.set("current_endpoint", current_endpoint);
    });
    Ok(state_sync.current_endpoint.clone())
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct UserDataPayload {
    userdata: UserData,
    password: String,
}

#[tauri::command]
async fn set_current_endpoint(
    window: Window,
    state: tauri::State<'_, TauriState>,
    current_endpoint: Endpoint,
) -> Result<LauncherResponse, String> {
    let req = {
        let mut state_sync = state.state_sync.lock().await;
        state_sync.cancel_shared.cancel();
        state_sync.cancel_launcher.cancel();
        state_sync.cancel_launcher = CancellationToken::new();
        if state_sync.current_endpoint == current_endpoint {
            if let Some(launcher_resp) = &state_sync.launcher_resp {
                return Ok(launcher_resp.clone());
            }
        }
        state_sync.launcher_resp = None;
        state_sync.current_endpoint = current_endpoint.clone();
        let (userdata, password) = state_sync.user_manager.get(&state_sync.current_endpoint);
        window
            .emit("userdata", UserDataPayload { userdata, password })
            .unwrap_or_else(|e| warn!("failed to emit message: {}", e));
        if !state_sync.contains_endpoint(&current_endpoint) {
            let payload = if current_endpoint.is_remote {
                state_sync
                    .remote_endpoints
                    .insert(0, current_endpoint.clone());
                EndpointsPayload {
                    remote_endpoints: Some(state_sync.remote_endpoints.clone()),
                    ..Default::default()
                }
            } else {
                state_sync.endpoints.insert(0, current_endpoint.clone());
                let endpoints = state_sync.endpoints.clone();
                state_sync.store.with(|s| s.set("endpoints", endpoints));
                EndpointsPayload {
                    endpoints: Some(state_sync.endpoints.clone()),
                    ..Default::default()
                }
            };
            window
                .emit("endpoints", payload)
                .unwrap_or_else(|e| warn!("failed to emit message: {}", e));
        }
        state_sync
            .store
            .with(|s| s.set("current_endpoint", current_endpoint.clone()));
        server::launcher_request(
            &state.client,
            state_sync.cancel_launcher.clone(),
            &state_sync.current_endpoint,
        )
    };
    let launcher_resp = req.send().await.map_err(|e| e.into_frontend())?;
    let mut state_sync = state.state_sync.lock().await;
    state_sync.launcher_resp = Some(launcher_resp.clone());
    Ok(launcher_resp)
}

#[tauri::command]
async fn set_game_folder(
    state: tauri::State<'_, TauriState>,
    game_folder: Option<String>,
) -> Result<(), String> {
    let mut state_sync = state.state_sync.lock().await;
    let game_folder = game_folder.map(PathBuf::from);
    if let Some(f) = game_folder.as_ref() {
        if !f.is_dir() {
            return Err("path-folder-error".into());
        } else if !f.exists() {
            return Err("path-exists-error".into());
        }
    }
    state_sync.game_folder = game_folder.clone();
    state_sync.store.with(|s| s.set("game_folder", game_folder));
    Ok(())
}

#[tauri::command]
async fn set_serverlist_url(
    window: Window,
    state: tauri::State<'_, TauriState>,
    serverlist_url: String,
) -> Result<(), String> {
    if serverlist_url.is_empty() {
        let state_sync = &mut *state.state_sync.lock().await;
        state_sync.remote_endpoints = config::get_default_endpoints();
        if state_sync.current_endpoint.is_remote
            && !state_sync
                .remote_endpoints
                .contains(&state_sync.current_endpoint)
        {
            let current_endpoint = state_sync.current_endpoint.clone();
            state_sync.remote_endpoints.push(current_endpoint);
        }
        state_sync
            .remote_endpoints
            .apply_config(&state_sync.remote_endpoints_config);
        let payload = EndpointsPayload {
            remote_endpoints: Some(state_sync.remote_endpoints.clone()),
            ..Default::default()
        };
        window
            .emit("endpoints", payload)
            .unwrap_or_else(|e| warn!("failed to emit message: {}", e));
    } else {
        let req = {
            let mut state_sync = state.state_sync.lock().await;
            if serverlist_url == state_sync.serverlist_url {
                return Ok(());
            }
            state_sync.cancel_serverlist.cancel();
            state_sync.cancel_serverlist = CancellationToken::new();
            server::simple_request(
                &state.client,
                state_sync.cancel_serverlist.clone(),
                &serverlist_url,
            )
        };
        handle_remote_endpoints(&window, req, state.state_sync.clone()).await;
    }
    let mut state_sync = state.state_sync.lock().await;
    state_sync.serverlist_url = serverlist_url.clone();
    state_sync
        .store
        .with(|s| s.set("serverlist_url", serverlist_url));
    Ok(())
}

#[tauri::command]
async fn set_messagelist_url(
    window: Window,
    state: tauri::State<'_, TauriState>,
    messagelist_url: String,
) -> Result<(), String> {
    info!("messagelisturl: {}", messagelist_url);
    if !messagelist_url.is_empty() {
        let req = {
            let mut state_sync = state.state_sync.lock().await;
            if messagelist_url == state_sync.messagelist_url {
                return Ok(());
            }
            state_sync.cancel_messagelist.cancel();
            state_sync.cancel_messagelist = CancellationToken::new();
            server::simple_request(
                &state.client,
                state_sync.cancel_messagelist.clone(),
                &messagelist_url,
            )
        };
        handle_remote_messages(&window, req).await;
    }
    let mut state_sync = state.state_sync.lock().await;
    state_sync
        .store
        .with(|s| s.set("messagelist_url", messagelist_url));
    Ok(())
}

async fn auth(
    state: tauri::State<'_, TauriState>,
    username: String,
    password: String,
    remember_me: bool,
    auth_req: JsonRequest<AuthResponse>,
) -> Result<AuthPayload, String> {
    let auth_resp = auth_req.send().await.map_err(|e| e.into_frontend())?;
    let patcher_resp = if !auth_resp.patch_server.is_empty() {
        let patcher_req = {
            let state_sync = state.state_sync.lock().await;
            server::patcher_request(
                &state.client,
                state_sync.cancel_shared.clone(),
                &auth_resp.patch_server,
                &patcher::get_etag(&state_sync.effective_folder()),
            )
        };
        patcher_req.send().await.map_err(|e| e.into_frontend())?
    } else {
        None
    };
    let state_sync = &mut *state.state_sync.lock().await;
    state_sync.auth_resp = Some(auth_resp.clone());
    let has_patch = patcher_resp.is_some();
    state_sync.patcher_resp = patcher_resp;
    state_sync.user_manager.set(
        &state_sync.current_endpoint,
        UserData {
            username,
            remember_me,
        },
        password,
    );
    state_sync
        .store
        .with(|s| s.set("user_manager", &state_sync.user_manager));
    Ok(AuthPayload {
        response: auth_resp,
        has_patch,
    })
}

#[tauri::command]
async fn login(
    state: tauri::State<'_, TauriState>,
    username: String,
    password: String,
    remember_me: bool,
) -> Result<AuthPayload, String> {
    let auth_req = {
        let mut state_sync = state.state_sync.lock().await;
        if username.is_empty() || password.is_empty() {
            return Err("username-password-empty-error".into());
        }
        state_sync.cancel_shared.cancel();
        state_sync.cancel_shared = CancellationToken::new();
        server::login_request(
            &state.client,
            state_sync.cancel_shared.clone(),
            &state_sync.current_endpoint,
            &username,
            &password,
        )
    };
    auth(state, username, password, remember_me, auth_req).await
}

#[tauri::command]
async fn register(
    state: tauri::State<'_, TauriState>,
    username: String,
    password: String,
    remember_me: bool,
) -> Result<AuthPayload, String> {
    let auth_req = {
        let mut state_sync = state.state_sync.lock().await;
        if username.is_empty() || password.is_empty() {
            return Err("username-password-empty-error".into());
        }
        state_sync.cancel_shared.cancel();
        state_sync.cancel_shared = CancellationToken::new();
        server::register_request(
            &state.client,
            state_sync.cancel_shared.clone(),
            &state_sync.current_endpoint,
            &username,
            &password,
        )
    };
    auth(state, username, password, remember_me, auth_req).await
}

async fn reauth(state: &mut tauri::State<'_, TauriState>) -> Result<(), String> {
    let req = {
        let mut state_sync = state.state_sync.lock().await;
        state_sync.cancel_shared.cancel();
        state_sync.cancel_shared = CancellationToken::new();
        let (userdata, password) = state_sync.user_manager.get(&state_sync.current_endpoint);
        server::login_request(
            &state.client,
            state_sync.cancel_shared.clone(),
            &state_sync.current_endpoint,
            &userdata.username,
            &password,
        )
    };
    let data = req.send().await.map_err(|e| e.into_frontend())?;
    {
        let mut state_sync = state.state_sync.lock().await;
        state_sync.auth_resp = Some(data);
    }
    Ok(())
}

async fn get_create_character_request(
    state: &mut tauri::State<'_, TauriState>,
) -> Result<server::JsonRequest<server::CharacterData>, String> {
    let mut state_sync = state.state_sync.lock().await;
    state_sync.cancel_shared.cancel();
    state_sync.cancel_shared = CancellationToken::new();
    let req = server::create_character_request(
        &state.client,
        state_sync.cancel_shared.clone(),
        &state_sync.current_endpoint,
        &state_sync.auth_resp_err()?.user.token,
    );
    Ok(req)
}

#[tauri::command]
async fn create_character(
    window: Window,
    mut state: tauri::State<'_, TauriState>,
) -> Result<(), String> {
    let req = get_create_character_request(&mut state).await?;
    let character = match req.send().await {
        Ok(data) => data,
        Err(server::Error::Server(401, _)) => {
            reauth(&mut state).await?;
            let req = get_create_character_request(&mut state).await?;
            req.send().await.map_err(|e| e.into_frontend())?
        }
        Err(e) => return Err(e.into_frontend()),
    };
    let mut state_sync = state.state_sync.lock().await;
    state_sync.exit_reason = Some(ExitSignal::RunGame(character.id, true));
    state_sync
        .auth_resp
        .as_mut()
        .ok_or("Auth data was not set")?
        .characters
        .push(character.clone());
    state_sync.store.with(|s| {
        s.set("last_char_id", character.id);
    });
    window.close().map_err(|e| {
        error!("failed to close window: {}", e);
        "internal-error"
    })?;
    Ok(())
}

#[tauri::command]
async fn select_character(
    window: Window,
    state: tauri::State<'_, TauriState>,
    character_id: u32,
) -> Result<(), String> {
    let mut state_sync = state.state_sync.lock().await;
    state_sync.exit_reason = Some(ExitSignal::RunGame(character_id, false));
    state_sync.store.with(|s| {
        s.set("last_char_id", character_id);
    });
    window.close().map_err(|e| {
        error!("failed to close window: {}", e);
        "internal-error"
    })?;
    Ok(())
}

async fn get_delete_character_request(
    state: &mut tauri::State<'_, TauriState>,
    character_id: i32,
) -> Result<server::JsonRequest<server::EmptyResponse>, String> {
    let mut state_sync = state.state_sync.lock().await;
    state_sync.cancel_shared.cancel();
    state_sync.cancel_shared = CancellationToken::new();
    let req = server::delete_character_request(
        &state.client,
        state_sync.cancel_shared.clone(),
        &state_sync.current_endpoint,
        &state_sync.auth_resp_err()?.user.token,
        character_id,
    );
    Ok(req)
}

#[tauri::command]
async fn delete_character(
    mut state: tauri::State<'_, TauriState>,
    character_id: i32,
) -> Result<(), String> {
    let req = get_delete_character_request(&mut state, character_id).await?;
    let _ = match req.send().await {
        Ok(data) => data,
        Err(server::Error::Server(401, _)) => {
            reauth(&mut state).await?;
            let req = get_delete_character_request(&mut state, character_id).await?;
            req.send().await.map_err(|e| e.into_frontend())?
        }
        Err(e) => return Err(e.into_frontend()),
    };
    Ok(())
}

async fn get_export_character_request(
    state: &mut tauri::State<'_, TauriState>,
    character_id: i32,
) -> Result<server::JsonRequest<Value>, String> {
    let state_sync = state.state_sync.lock().await;
    let req = server::export_save_request(
        &state.client,
        CancellationToken::new(),
        &state_sync.current_endpoint,
        &state_sync.auth_resp_err()?.user.token,
        character_id,
    );
    Ok(req)
}

#[tauri::command]
async fn export_character(
    mut state: tauri::State<'_, TauriState>,
    character_id: i32,
) -> Result<PathBuf, String> {
    let req = get_export_character_request(&mut state, character_id).await?;
    let data = match req.send().await {
        Ok(data) => data,
        Err(server::Error::Server(401, _)) => {
            reauth(&mut state).await?;
            let req = get_export_character_request(&mut state, character_id).await?;
            req.send().await.map_err(|e| e.into_frontend())?
        }
        Err(e) => return Err(e.into_frontend()),
    };
    let id = data.get("id").and_then(Value::as_i64).unwrap_or_default();
    let name = data.get("name").and_then(Value::as_str).unwrap_or_default();
    let folder_name = format!("./saves/{}-{}.json", id, name);
    let path = Path::new(&folder_name);
    path.parent()
        .and_then(|p| std::fs::create_dir_all(p).ok())
        .ok_or("file-error")?;
    File::options()
        .write(true)
        .create(true)
        .open(path)
        .ok()
        .and_then(|f| serde_json::to_writer_pretty(f, &data).ok())
        .ok_or("file-error")?;
    path::absolute(path).or(Err("file-error".into()))
}

#[tauri::command]
async fn patcher_start(window: Window, state: tauri::State<'_, TauriState>) -> Result<(), String> {
    let (patcher_url, patcher_resp, game_folder, cancel) = {
        let mut state_sync = state.state_sync.lock().await;
        state_sync.cancel_shared.cancel();
        state_sync.cancel_shared = CancellationToken::new();
        (
            state_sync.auth_resp_err()?.patch_server.clone(),
            state_sync.patcher_resp.take(),
            state_sync.effective_folder(),
            state_sync.cancel_shared.clone(),
        )
    };
    let Some(patcher_resp) = patcher_resp else {
        return Err("internal-error".into());
    };
    let _client = state.client.clone();
    tauri::async_runtime::spawn(patcher::patch(
        window,
        _client,
        patcher_url,
        patcher_resp,
        game_folder,
        cancel,
    ));
    Ok(())
}

#[tauri::command]
async fn patcher_stop(state: tauri::State<'_, TauriState>) -> Result<(), String> {
    let state_sync = state.state_sync.lock().await;
    state_sync.cancel_shared.cancel();
    Ok(())
}

fn handle_style(window: &mut Window, style: u32) {
    match style {
        CLASSIC_STYLE => {
            window.set_decorations(false).unwrap();
            window.set_size(PhysicalSize::new(1124, 600)).unwrap();
        }
        MODERN_STYLE => {
            window.set_decorations(true).unwrap();
            window.set_size(PhysicalSize::new(899, 480)).unwrap();
        }
        _ => {}
    }
}

async fn handle_remote_endpoints(
    window: &Window,
    req: server::JsonRequest<Vec<Endpoint>>,
    state_sync_mutex: Arc<Mutex<TauriStateSync>>,
) {
    let mut serverlist_endpoints = match req.send().await {
        Ok(endpoints) => endpoints,
        Err(e) => {
            warn!("failed to fetch remote servers: {}", e);
            window
                .emit("log", LogPayload::warning("remote-endpoint-error"))
                .unwrap_or_else(|e| warn!("failed to emit message: {}", e));
            return;
        }
    };
    for endpoint in &mut serverlist_endpoints {
        endpoint.is_remote = true;
    }
    let mut remote_endpoints = config::get_default_endpoints();
    let default_len = remote_endpoints.len();
    remote_endpoints.extend_valid(serverlist_endpoints);
    let state_sync = &mut *state_sync_mutex.lock().await;
    if state_sync.current_endpoint.is_remote
        && !remote_endpoints.contains(&state_sync.current_endpoint)
    {
        remote_endpoints.insert(default_len, state_sync.current_endpoint.clone())
    }
    remote_endpoints.apply_config(&state_sync.remote_endpoints_config);
    state_sync.remote_endpoints = remote_endpoints;
    let payload = EndpointsPayload {
        remote_endpoints: Some(state_sync.remote_endpoints.clone()),
        ..Default::default()
    };
    window
        .emit("endpoints", payload)
        .unwrap_or_else(|e| warn!("failed to emit message: {}", e));
}

async fn handle_remote_messages(window: &Window, req: server::JsonRequest<Vec<MessageData>>) {
    match req.send().await {
        Ok(messages) => {
            info!("got: {:?}", messages);
            window.emit("messages", messages)
        }
        Err(e) => {
            warn!("failed to fetch global messages: {}", e);
            window.emit("log", LogPayload::warning("remote-messages-error"))
        }
    }
    .unwrap_or_else(|e| warn!("failed to emit message: {}", e));
}

fn main() {
    // Log plugin has an issue where it cannot be initialized twice.
    let mut log_plugin_initial = Some(
        tauri_plugin_log::Builder::default()
            .targets([LogTarget::LogDir, LogTarget::Stdout, LogTarget::Webview])
            .build(),
    );
    loop {
        let (config, run) = {
            let default_endpoints = config::get_default_endpoints();
            let current_endpoint = default_endpoints[0].clone();
            let state_sync = Arc::new(Mutex::new(TauriStateSync {
                remote_endpoints: default_endpoints,
                current_endpoint,
                locale: "en".into(),
                serverlist_url: DEFAULT_SERVERLIST_URL.into(),
                messagelist_url: DEFAULT_MESSAGELIST_URL.into(),
                ..Default::default()
            }));
            let mut builder =
                tauri::Builder::default().plugin(tauri_plugin_store::Builder::default().build());
            if let Some(log_plugin) = log_plugin_initial.take() {
                builder = builder.plugin(log_plugin);
            }
            let mut app = builder
                .manage(TauriState {
                    client: reqwest::ClientBuilder::new().gzip(true).build().unwrap(),
                    state_sync: state_sync.clone(),
                })
                .setup(|app| {
                    let mut window = app.get_window("main").unwrap();
                    window.hide().unwrap();
                    let state: tauri::State<'_, TauriState> = app.state();
                    let mut store = StoreBuilder::new(app.handle(), "config.json".parse()?).build();
                    let state_sync = &mut *state.state_sync.blocking_lock();
                    match &mut store.load() {
                        Ok(_) => {
                            store::get(&store, "style", &mut state_sync.style);
                            store::get(&store, "locale", &mut state_sync.locale);
                            store::get(&store, "endpoints", &mut state_sync.endpoints);
                            store::get(
                                &store,
                                "remote_endpoints_config",
                                &mut state_sync.remote_endpoints_config,
                            );
                            store::get(
                                &store,
                                "current_endpoint",
                                &mut state_sync.current_endpoint,
                            );
                            store::get(&store, "user_manager", &mut state_sync.user_manager);
                            store::get(&store, "game_folder", &mut state_sync.game_folder);
                            store::get(&store, "last_char_id", &mut state_sync.last_char_id);
                            store::get(&store, "serverlist_url", &mut state_sync.serverlist_url);
                            store::get(&store, "messagelist_url", &mut state_sync.messagelist_url);
                            state_sync
                                .remote_endpoints
                                .apply_config(&state_sync.remote_endpoints_config);
                            handle_style(&mut window, state_sync.style);
                        }
                        Err(e) => info!("unable to load config from disk: {}", e),
                    }
                    state_sync.store = StoreHelper::new(store);
                    window.show().unwrap();
                    if !state_sync.serverlist_url.is_empty() {
                        let endpoints_req = server::simple_request(
                            &state.client,
                            state_sync.cancel_serverlist.clone(),
                            &state_sync.serverlist_url,
                        );
                        let state_sync_mutex = state.state_sync.clone();
                        let window = window.clone();
                        tauri::async_runtime::spawn(async move {
                            handle_remote_endpoints(&window, endpoints_req, state_sync_mutex).await
                        });
                    }
                    if !state_sync.messagelist_url.is_empty() {
                        let messages_req = server::simple_request(
                            &state.client,
                            state_sync.cancel_messagelist.clone(),
                            &state_sync.messagelist_url,
                        );
                        let window = window.clone();
                        tauri::async_runtime::spawn(async move {
                            handle_remote_messages(&window, messages_req).await
                        });
                    }
                    Ok(())
                })
                .invoke_handler(tauri::generate_handler![
                    initial_data,
                    set_style,
                    set_locale,
                    set_endpoints,
                    set_remote_endpoints,
                    set_current_endpoint,
                    set_game_folder,
                    set_serverlist_url,
                    set_messagelist_url,
                    login,
                    register,
                    create_character,
                    select_character,
                    delete_character,
                    export_character,
                    patcher_start,
                    patcher_stop,
                ])
                .build(tauri::generate_context!())
                .expect("error while building tauri application");
            loop {
                let iteration = app.run_iteration();
                if iteration.window_count == 0 {
                    break;
                }
            }
            tauri::api::process::kill_children();

            let state_sync = state_sync.blocking_lock();
            if let Some(ExitSignal::RunGame(char_id, char_new)) = state_sync.exit_reason {
                let auth_resp = state_sync.auth_resp.as_ref().unwrap();
                let char = auth_resp
                    .characters
                    .iter()
                    .find(|c| c.id == char_id)
                    .unwrap();
                let char_ids = auth_resp.characters.iter().map(|c| c.id).collect();
                let notices = auth_resp
                    .notices
                    .iter()
                    .map(|n| mhf_iel::Notice {
                        flags: 0,
                        data: n.clone(),
                    })
                    .collect();
                let (userdata, password) =
                    state_sync.user_manager.get(&state_sync.current_endpoint);

                let mut config = MhfConfig {
                    char_id,
                    char_name: char.name.clone(),
                    char_gr: char.gr,
                    char_hr: char.hr,
                    char_ids,
                    char_new,
                    user_token: auth_resp.user.token.clone(),
                    user_name: userdata.username,
                    user_password: password,
                    user_rights: auth_resp.user.rights,
                    server_host: state_sync.current_endpoint.host(),
                    server_port: state_sync.current_endpoint.game_port.unwrap_or(53310) as u32,
                    entrance_count: auth_resp.entrance_count,
                    current_ts: auth_resp.current_ts,
                    expiry_ts: auth_resp.expiry_ts,
                    notices,
                    mez_event_id: 0,
                    mez_start: 0,
                    mez_end: 0,
                    mez_solo_tickets: 0,
                    mez_group_tickets: 0,
                    mez_stalls: vec![],
                    mhf_flags: None,
                    version: state_sync.current_endpoint.version,

                    mhf_folder: state_sync
                        .current_endpoint
                        .game_folder
                        .as_ref()
                        .or_else(|| state_sync.game_folder.as_ref())
                        .cloned(),
                };
                if let Some(mez_fes) = auth_resp.mez_fez.as_ref() {
                    config.mez_event_id = mez_fes.id;
                    config.mez_start = mez_fes.start;
                    config.mez_end = mez_fes.end;
                    config.mez_solo_tickets = mez_fes.solo_tickets;
                    config.mez_group_tickets = mez_fes.group_tickets;
                    config.mez_stalls = mez_fes
                        .stalls
                        .iter()
                        .map(|&s| mhf_iel::MezFesStall::try_from(s).unwrap())
                        .collect();
                }
                (config, true)
            } else {
                (MhfConfig::default(), false)
            }
        };
        if run {
            match mhf_iel::run(config).unwrap() {
                102 => {}
                code => info!("exited with code {}", code),
            };
        } else {
            break;
        }
    }
    info!("app exit");
}
