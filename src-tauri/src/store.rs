use log::warn;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{from_value, to_value};
use tauri::Wry;
use tauri_plugin_store::Store;

#[derive(Debug, Default)]
pub struct StoreHelper {
    store: Option<Store<Wry>>,
}

impl StoreHelper {
    pub fn new(store: Store<Wry>) -> Self {
        Self { store: Some(store) }
    }

    pub fn with<F: FnOnce(&mut StoreTransaction)>(&mut self, f: F) {
        let store = self.store.as_mut().unwrap();
        let mut t = StoreTransaction { store };
        f(&mut t);
        if let Err(e) = store.save() {
            warn!("unable to save store: {}", e);
        }
    }
}

pub struct StoreTransaction<'a> {
    store: &'a mut Store<Wry>,
}

impl<'a> StoreTransaction<'a> {
    pub fn set<T: Serialize>(&mut self, key: &'static str, value: T) {
        let value = match to_value(value) {
            Ok(v) => v,
            Err(e) => {
                warn!("unable to serialize store value on key '{}': {}", key, e);
                return;
            }
        };
        if let Err(e) = self.store.insert(key.into(), value) {
            warn!("unable to insert value into store: {}", e);
        }
    }
}

pub fn get<T: DeserializeOwned>(store: &Store<Wry>, key: &str, target: &mut T) {
    let Some(value) = store.get(key) else {
        return;
    };
    let value = match from_value(value.clone()) {
        Ok(v) => v,
        Err(e) => {
            warn!("unable to load value: {}", e);
            return;
        }
    };
    *target = value;
}
