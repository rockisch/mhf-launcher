use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub game_folder: Option<PathBuf>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialOrd, Ord)]
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

impl PartialEq for Endpoint {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.is_remote == other.is_remote
    }
}

impl Endpoint {
    pub fn url(&self, path: &str) -> String {
        let port = self.launcher_port.unwrap_or(8080);
        if self.host.contains("://") {
            format!("{}:{}{}", self.host, port, path)
        } else {
            format!("http://{}:{}{}", self.host, port, path)
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
                return Err("Server name must not be empty");
            } else if endpoint.host.is_empty() {
                return Err("Server host must not be empty");
            } else if self.iter().filter(|e| e.name == endpoint.name).count() > 1 {
                return Err("Server names must be unique");
            }
            if let Some(game_folder) = endpoint.game_folder.as_ref() {
                if !game_folder.exists() {
                    println!("{:?}", game_folder);
                    return Err("The specified game folder does not exist");
                }
            }
        }
        Ok(())
    }

    fn extend_valid(&mut self, other: Self) {
        self.reserve(other.len());
        for endpoint in other {
            if !endpoint.name.is_empty() && !endpoint.host.is_empty() && !self.contains(&endpoint) {
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
