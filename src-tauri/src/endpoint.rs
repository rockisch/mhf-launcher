use std::{collections::HashMap, path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub game_folder: Option<PathBuf>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct Endpoint {
    pub url: String,
    pub name: String,
    pub launcher_port: Option<u16>,
    pub game_port: Option<u16>,
    pub game_folder: Option<PathBuf>,
    pub version: mhf_iel::MhfVersion,
    #[serde(default)]
    pub is_remote: bool,
}

impl PartialEq for Endpoint {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.is_remote == other.is_remote
    }
}

impl Endpoint {
    pub fn host(&self) -> String {
        reqwest::Url::from_str(&self.url)
            .ok()
            .and_then(|u| u.host().map(|h| h.to_string()))
            .unwrap_or(self.url.to_owned())
    }

    pub fn get_url(&self, path: &str) -> String {
        let port = self.launcher_port.unwrap_or(8080);
        if self.url.contains("://") {
            format!("{}:{}{}", self.url, port, path)
        } else {
            format!("http://{}:{}{}", self.url, port, path)
        }
    }
}

pub trait EndpointVecExt {
    fn check_valid(&self) -> Result<(), &'static str>;
    fn extend_valid(&mut self, other: Self);
    fn apply_config(&mut self, configs: &HashMap<String, EndpointConfig>);
    fn update_config(&self, configs: &mut HashMap<String, EndpointConfig>);
}

impl EndpointVecExt for Vec<Endpoint> {
    fn check_valid(&self) -> Result<(), &'static str> {
        for endpoint in self {
            if endpoint.name.is_empty() {
                return Err("endpoint-name-empty");
            } else if endpoint.url.is_empty() {
                return Err("endpoint-host-empty");
            } else if self.iter().filter(|e| e.name == endpoint.name).count() > 1 {
                return Err("endpoint-unique");
            }
            if let Some(game_folder) = endpoint.game_folder.as_ref() {
                if !game_folder.exists() {
                    return Err("path-exists-error");
                }
            }
        }
        Ok(())
    }

    fn extend_valid(&mut self, other: Self) {
        self.reserve(other.len());
        for endpoint in other {
            if !endpoint.name.is_empty() && !endpoint.url.is_empty() && !self.contains(&endpoint) {
                self.push(endpoint)
            }
        }
    }

    fn apply_config(&mut self, configs: &HashMap<String, EndpointConfig>) {
        for endpoint in self {
            if let Some(config) = configs.get(&endpoint.name) {
                endpoint.game_folder = config.game_folder.clone();
            }
        }
    }

    fn update_config(&self, configs: &mut HashMap<String, EndpointConfig>) {
        for endpoint in self {
            if endpoint.game_folder.is_some() {
                configs.insert(
                    endpoint.name.clone(),
                    EndpointConfig {
                        game_folder: endpoint.game_folder.clone(),
                    },
                );
            } else {
                configs.remove(&endpoint.name);
            }
        }
    }
}
