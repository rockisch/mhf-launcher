use std::collections::HashMap;

use crate::endpoint::Endpoint;
use log::warn;
use serde::{Deserialize, Serialize};

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
    fn get_target(&self, endpoint: &'_ Endpoint) -> String {
        format!("{}:{}", endpoint.name, endpoint.is_remote)
    }

    pub fn get(&self, endpoint: &'_ Endpoint) -> (UserData, String) {
        let target = self.get_target(endpoint);
        let data = &self.data[endpoint.is_remote as usize];
        let userdata = data
            .get(&endpoint.name)
            .cloned()
            .unwrap_or_else(|| UserData {
                username: "".into(),
                remember_me: true,
            });
        let password = if !userdata.username.is_empty() {
            keyring::Entry::new_with_target(&target, APP_NAME, &userdata.username)
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
        let target = self.get_target(endpoint);
        let data = &mut self.data[endpoint.is_remote as usize];
        let entry = keyring::Entry::new_with_target(&target, APP_NAME, &userdata.username);
        if userdata.remember_me {
            entry
                .and_then(|entry| entry.set_password(&password))
                .unwrap_or_else(|e| warn!("failed to save password: {}", e));
            data.insert(endpoint.name.to_owned(), userdata);
        } else {
            entry
                .and_then(|entry| entry.delete_password())
                .unwrap_or_else(|e| warn!("failed to save password: {}", e));
            data.remove(&endpoint.name);
        }
    }
}
