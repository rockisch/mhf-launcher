#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod config;
mod server;
mod store;

use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::Arc,
};

use log::{error, info, warn};
use mhf_iel::MhfConfig;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use server::{AuthResponse, LauncherResponse};
use store::StoreHelper;
use tauri::{async_runtime::Mutex, PhysicalSize};
use tauri::{Manager, Window};
use tauri_plugin_log::LogTarget;
use tauri_plugin_store::StoreBuilder;
use tokio_util::sync::CancellationToken;

use crate::config::{CLASSIC_STYLE, DEFAULT_SERVERLIST_URL, MODERN_STYLE};

const APP_NAME: &str = "mhf-launcher";

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
    exit_reason: Option<ExitSignal>,
    endpoints: Vec<Endpoint>,
    remote_endpoints: Vec<Endpoint>,
    current_endpoint: Endpoint,
    launcher_resp: Option<LauncherResponse>,
    cancel_launcher_resp: CancellationToken,
    auth_resp: Option<AuthResponse>,
    cancel_auth_resp: CancellationToken,
    cancel_delete_character_resp: CancellationToken,
    cancel_create_character_resp: CancellationToken,
    cancel_serverlist_resp: CancellationToken,
    username: String,
    password: String,
    remember_me: bool,
    game_folder: Option<PathBuf>,
    last_char_id: Option<u32>,
    serverlist_url: String,
}

impl TauriStateSync {
    fn first_endpoint(&self) -> Result<&Endpoint, &str> {
        self.remote_endpoints
            .first()
            .or_else(|| self.endpoints.first())
            .ok_or("Unable to find a valid endpoint")
    }

    fn contains_endpoint(&self, endpoint: &Endpoint) -> bool {
        if self.current_endpoint.is_remote {
            self.remote_endpoints.contains(endpoint)
        } else {
            self.endpoints.contains(endpoint)
        }
    }

    fn auth_resp_err(&self) -> Result<&AuthResponse, &str> {
        self.auth_resp
            .as_ref()
            .ok_or("Authentication data is not set")
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Endpoint {
    pub name: String,
    pub host: String,
    pub launcher_port: Option<u16>,
    pub game_port: Option<u16>,
    pub game_folder: Option<PathBuf>,
    #[serde(default)]
    pub is_remote: bool,
}

impl Endpoint {
    fn get_launcher_port(&self) -> u16 {
        self.launcher_port.unwrap_or(8080)
    }
}

#[derive(Default, Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EndpointsPayload {
    endpoints: Option<Vec<Endpoint>>,
    remote_endpoints: Option<Vec<Endpoint>>,
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
}

#[tauri::command]
async fn initial_data(state: tauri::State<'_, TauriState>) -> Result<InitialDataPayload, ()> {
    let state_sync = state.state_sync.lock().await;
    Ok(InitialDataPayload {
        style: state_sync.style,
        endpoints: state_sync.endpoints.clone(),
        remote_endpoints: state_sync.remote_endpoints.clone(),
        current_endpoint: state_sync.current_endpoint.clone(),
        username: if state_sync.remember_me {
            state_sync.username.clone()
        } else {
            "".into()
        },
        password: if state_sync.remember_me {
            state_sync.password.clone()
        } else {
            "".into()
        },
        remember_me: state_sync.remember_me,
        game_folder: state_sync.game_folder.clone(),
        current_folder: std::env::current_dir().unwrap(),
        locale: state_sync.locale.clone(),
        last_char_id: state_sync.last_char_id,
        serverlist_url: state_sync.serverlist_url.clone(),
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
    current_endpoint: Endpoint,
) -> Result<Endpoint, String> {
    let mut state_sync = state.state_sync.lock().await;
    for endpoint in &endpoints {
        if endpoint.name.is_empty() {
            return Err("Server name must not be empty".into());
        } else if endpoint.host.is_empty() {
            return Err("Server host must not be empty".into());
        } else if endpoints.iter().filter(|e| e.name == endpoint.name).count() > 1 {
            return Err("Server names must be unique".into());
        }
    }
    state_sync.endpoints = endpoints.clone();
    if state_sync.contains_endpoint(&current_endpoint) {
        state_sync.current_endpoint = current_endpoint.clone();
    } else {
        state_sync.current_endpoint = state_sync.first_endpoint()?.clone();
    }
    let current_endpoint = state_sync.current_endpoint.clone();
    state_sync.store.with(|s| {
        s.set("endpoints", endpoints);
        s.set("current_endpoint", current_endpoint);
    });
    Ok(state_sync.current_endpoint.clone())
}

#[tauri::command]
async fn set_current_endpoint(
    window: Window,
    state: tauri::State<'_, TauriState>,
    current_endpoint: Endpoint,
) -> Result<LauncherResponse, String> {
    let req = {
        let mut state_sync = state.state_sync.lock().await;
        state_sync.cancel_auth_resp.cancel();
        state_sync.cancel_launcher_resp.cancel();
        state_sync.cancel_launcher_resp = CancellationToken::new();
        if state_sync.current_endpoint == current_endpoint {
            if let Some(launcher_resp) = &state_sync.launcher_resp {
                return Ok(launcher_resp.clone());
            }
        }
        state_sync.launcher_resp = None;
        state_sync.current_endpoint = current_endpoint.clone();
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
                .unwrap_or_else(|e| error!("failed to send endpoints event: {}", e));
        }
        state_sync
            .store
            .with(|s| s.set("current_endpoint", current_endpoint));
        server::launcher_request(
            &state.client,
            state_sync.cancel_launcher_resp.clone(),
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
            return Err("Path must be a directory".into());
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
    mut serverlist_url: String,
) -> Result<(), String> {
    if serverlist_url.is_empty() {
        serverlist_url = DEFAULT_SERVERLIST_URL.into();
    }
    let req = {
        let mut state_sync = state.state_sync.lock().await;
        if serverlist_url == state_sync.serverlist_url {
            return Ok(());
        }
        state_sync.cancel_serverlist_resp.cancel();
        state_sync.cancel_serverlist_resp = CancellationToken::new();
        state_sync.serverlist_url = serverlist_url.clone();
        state_sync
            .store
            .with(|s| s.set("serverlist_url", serverlist_url));
        server::endpoints_request(
            &state.client,
            state_sync.cancel_serverlist_resp.clone(),
            &state_sync.serverlist_url,
        )
    };
    handle_remote_endpoints(&window, req, state.state_sync.clone()).await;
    Ok(())
}

#[tauri::command]
async fn login(
    state: tauri::State<'_, TauriState>,
    username: String,
    password: String,
    remember_me: bool,
) -> Result<AuthResponse, String> {
    let req = {
        let mut state_sync = state.state_sync.lock().await;
        state_sync.cancel_auth_resp.cancel();
        state_sync.cancel_auth_resp = CancellationToken::new();
        server::login_request(
            &state.client,
            state_sync.cancel_auth_resp.clone(),
            &state_sync.current_endpoint,
            &username,
            &password,
        )
    };
    let data = req.send().await.map_err(|e| e.into_frontend())?;
    let mut state_sync = state.state_sync.lock().await;
    state_sync.username = username.clone();
    state_sync.password = password.clone();
    state_sync.auth_resp = Some(data.clone());
    state_sync.store.with(|s| {
        if remember_me {
            s.set("username", &username);
            if let Err(e) = keyring::Entry::new(APP_NAME, &username)
                .and_then(|entry| entry.set_password(&password))
            {
                warn!("failed to save password: {}", e)
            }
        } else {
            s.del("username");
            _ = keyring::Entry::new(APP_NAME, &username).and_then(|entry| entry.delete_password());
        }
        s.set("remember_me", remember_me);
    });
    Ok(data)
}

#[tauri::command]
async fn register(
    state: tauri::State<'_, TauriState>,
    username: String,
    password: String,
    remember_me: bool,
) -> Result<AuthResponse, String> {
    let req = {
        let mut state_sync = state.state_sync.lock().await;
        state_sync.cancel_auth_resp.cancel();
        state_sync.cancel_auth_resp = CancellationToken::new();
        server::register_request(
            &state.client,
            state_sync.cancel_auth_resp.clone(),
            &state_sync.current_endpoint,
            &username,
            &password,
        )
    };
    let data = req.send().await.map_err(|e| e.into_frontend())?;
    let mut state_sync = state.state_sync.lock().await;
    state_sync.username = username.clone();
    state_sync.password = password.clone();
    state_sync.auth_resp = Some(data.clone());
    state_sync.store.with(|s| {
        if remember_me {
            s.set("username", &username);
            if let Err(e) = keyring::Entry::new(APP_NAME, &username)
                .and_then(|entry| entry.set_password(&password))
            {
                warn!("failed to save password: {}", e)
            }
        } else {
            s.del("username");
            _ = keyring::Entry::new(APP_NAME, &username).and_then(|entry| entry.delete_password());
        }
        s.set("remember_me", remember_me);
    });
    Ok(data)
}

async fn reauth(state: &mut tauri::State<'_, TauriState>) -> Result<(), String> {
    let req = {
        let mut state_sync = state.state_sync.lock().await;
        state_sync.cancel_auth_resp.cancel();
        state_sync.cancel_auth_resp = CancellationToken::new();
        server::login_request(
            &state.client,
            state_sync.cancel_auth_resp.clone(),
            &state_sync.current_endpoint,
            &state_sync.username,
            &state_sync.password,
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
) -> Result<server::Request<server::CharacterData>, String> {
    let mut state_sync = state.state_sync.lock().await;
    state_sync.cancel_create_character_resp.cancel();
    state_sync.cancel_create_character_resp = CancellationToken::new();
    let req = server::create_character_request(
        &state.client,
        state_sync.cancel_create_character_resp.clone(),
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
        "Failed to close window"
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
        "Failed to close window"
    })?;
    Ok(())
}

async fn get_delete_character_request(
    state: &mut tauri::State<'_, TauriState>,
    character_id: i32,
) -> Result<server::Request<server::EmptyResponse>, String> {
    let mut state_sync = state.state_sync.lock().await;
    state_sync.cancel_delete_character_resp.cancel();
    state_sync.cancel_delete_character_resp = CancellationToken::new();
    let req = server::delete_character_request(
        &state.client,
        state_sync.cancel_delete_character_resp.clone(),
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
) -> Result<server::Request<Value>, String> {
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
        .ok_or("Failed to create parent folder")?;
    File::options()
        .write(true)
        .create(true)
        .open(path)
        .ok()
        .and_then(|f| serde_json::to_writer_pretty(f, &data).ok())
        .ok_or("Failed to create save file")?;
    Ok(path.to_owned())
}

// TODO: re-enable auto-login logic?
// async fn handle_auto(
//     req: server::Request<AuthResponse>,
//     character_id: u32,
// ) -> Result<AuthResponse, String> {
//     let now = Instant::now();
//     let auth_resp = req.send().await.map_err(|e| e.into_frontend())?;
//     if !auth_resp.characters.iter().any(|c| c.id == character_id) {
//         return Err("The last played character does not exist anymore".into());
//     }
//     let took = Instant::now().duration_since(now);
//     tokio::time::sleep(Duration::from_secs(3).saturating_sub(took)).await;
//     Ok(auth_resp)
// }

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
    req: server::Request<Vec<Endpoint>>,
    state_sync_mutex: Arc<Mutex<TauriStateSync>>,
) {
    let resp = req.send().await;
    let mut remote_endpoints = config::get_fixed_endpoints();
    let fixed_len = remote_endpoints.len();
    let result = match resp {
        Ok(mut serverlist_endpoints) => {
            for endpoint in &mut serverlist_endpoints {
                endpoint.is_remote = true;
            }
            remote_endpoints.append(&mut serverlist_endpoints);
            Ok(())
        }
        Err(err) => Err(err.into_frontend()),
    };
    let mut state_sync = state_sync_mutex.lock().await;
    if state_sync.current_endpoint.is_remote
        && !remote_endpoints.contains(&state_sync.current_endpoint)
    {
        remote_endpoints.insert(fixed_len, state_sync.current_endpoint.clone());
    }
    state_sync.remote_endpoints = remote_endpoints;
    let payload = EndpointsPayload {
        remote_endpoints: Some(state_sync.remote_endpoints.clone()),
        ..Default::default()
    };
    window
        .emit("endpoints", payload)
        .unwrap_or_else(|e| error!("failed to send event: {}", e));
    if let Err(err) = result {
        window
            .emit(
                "error",
                format!("Failed to fetch data from serverlist url: {}", err).to_owned(),
            )
            .unwrap_or_else(|e| error!("failed to send event: {}", e));
    }
}

fn main() {
    // env_logger::Builder::new()
    //     .filter_level(log::LevelFilter::Info)
    //     .init();
    let (config, run) = {
        let fixed_endpoints = config::get_fixed_endpoints();
        let current_endpoint = fixed_endpoints[0].clone();
        let state_sync = Arc::new(Mutex::new(TauriStateSync {
            remote_endpoints: fixed_endpoints,
            current_endpoint,
            locale: "en".into(),
            serverlist_url: DEFAULT_SERVERLIST_URL.into(),
            ..Default::default()
        }));
        let mut app = tauri::Builder::default()
            .plugin(tauri_plugin_store::Builder::default().build())
            .plugin(
                tauri_plugin_log::Builder::default()
                    .targets([LogTarget::LogDir, LogTarget::Stdout, LogTarget::Webview])
                    .build(),
            )
            .manage(TauriState {
                client: reqwest::Client::new(),
                state_sync: state_sync.clone(),
            })
            .setup(|app| {
                let mut window = app.get_window("main").unwrap();
                window.hide().unwrap();
                let state: tauri::State<'_, TauriState> = app.state();
                let mut store = StoreBuilder::new(app.handle(), "config.json".parse()?).build();
                let mut state_sync = state.state_sync.blocking_lock();
                match &mut store.load() {
                    Ok(_) => {
                        store::get(&store, "style", &mut state_sync.style);
                        store::get(&store, "locale", &mut state_sync.locale);
                        store::get(&store, "endpoints", &mut state_sync.endpoints);
                        store::get(&store, "current_endpoint", &mut state_sync.current_endpoint);
                        store::get(&store, "username", &mut state_sync.username);
                        store::get(&store, "remember_me", &mut state_sync.remember_me);
                        store::get(&store, "game_folder", &mut state_sync.game_folder);
                        store::get(&store, "last_char_id", &mut state_sync.last_char_id);
                        store::get(&store, "serverlist_url", &mut state_sync.serverlist_url);
                        if !state_sync.username.is_empty() {
                            match keyring::Entry::new(APP_NAME, &state_sync.username)
                                .and_then(|entry| entry.get_password())
                            {
                                Ok(password) => state_sync.password = password,
                                Err(e) => warn!("failed to get user password: {}", e),
                            }
                        }
                        handle_style(&mut window, state_sync.style);
                    }
                    Err(e) => info!("unable to load config from disk: {}", e),
                }
                state_sync.store = StoreHelper::new(store);
                window.show().unwrap();
                let endpoints_req = server::endpoints_request(
                    &state.client,
                    state_sync.cancel_serverlist_resp.clone(),
                    &state_sync.serverlist_url,
                );
                let state_sync_mutex = state.state_sync.clone();
                tauri::async_runtime::spawn(async move {
                    handle_remote_endpoints(&window, endpoints_req, state_sync_mutex).await
                });
                Ok(())
            })
            .invoke_handler(tauri::generate_handler![
                initial_data,
                set_style,
                set_locale,
                set_endpoints,
                set_current_endpoint,
                set_game_folder,
                set_serverlist_url,
                login,
                register,
                create_character,
                select_character,
                delete_character,
                export_character,
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
            let mut config = MhfConfig {
                char_id,
                char_name: char.name.clone(),
                char_gr: char.gr,
                char_hr: char.hr,
                char_ids,
                char_new,
                user_token: auth_resp.user.token.clone(),
                user_name: state_sync.username.clone(),
                user_password: state_sync.password.clone(),
                user_rights: auth_resp.user.rights,
                server_host: state_sync.current_endpoint.host.clone(),
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
        mhf_iel::run(config).unwrap();
    }
    info!("app exit");
}
