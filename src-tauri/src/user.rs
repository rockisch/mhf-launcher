use std::collections::HashMap;

use log::warn;
use serde::{Deserialize, Serialize};
use tauri::Wry;
use tauri_plugin_store::Store;

use crate::endpoint::Endpoint;

const APP_NAME: &str = "mhf-launcher";

#[derive(Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserData {
    pub username: String,
    pub remember_me: bool,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserManager {
    data: [HashMap<String, UserData>; 2],
}

impl UserManager {
    pub fn get(&self, endpoint: &'_ Endpoint) -> (UserData, String) {
        let data = &self.data[endpoint.is_remote as usize];
        let userdata = data.get(&endpoint.name).cloned().unwrap_or_default();
        let password = if !userdata.username.is_empty() {
            keyring::Entry::new(APP_NAME, &userdata.username)
                .and_then(|entry| entry.get_password())
                .unwrap_or_else(|e| {
                    warn!("failed to get user password: {}", e);
                    "".to_owned()
                })
        } else {
            "".to_owned()
        };
        (userdata, password)
    }

    pub fn set(&mut self, endpoint: &'_ Endpoint, userdata: UserData, password: String) {
        let data = &mut self.data[endpoint.is_remote as usize];
        if userdata.remember_me {
            keyring::Entry::new(APP_NAME, &userdata.username)
                .and_then(|entry| entry.set_password(&password))
                .unwrap_or_else(|e| warn!("failed to save password: {}", e));
            data.insert(endpoint.name.to_owned(), userdata);
        } else {
            _ = keyring::Entry::new(APP_NAME, &userdata.username)
                .and_then(|entry| entry.delete_password());
            data.remove(&endpoint.name);
        }
    }
}
