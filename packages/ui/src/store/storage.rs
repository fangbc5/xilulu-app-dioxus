#[cfg(target_arch = "wasm32")]
pub mod local {
    use gloo_storage::{LocalStorage, Storage};

    pub fn get(key: &str) -> Option<String> {
        LocalStorage::get(key).ok()
    }

    pub fn set(key: &str, value: &str) {
        let _ = LocalStorage::set(key, value);
    }
    
    pub fn remove(key: &str) {
        LocalStorage::delete(key);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub mod local {
    use directories::ProjectDirs;
    use sled::Db;
    use lazy_static::lazy_static;
    use std::sync::Mutex;
    
    lazy_static! {
        static ref DB: Mutex<Option<Db>> = Mutex::new(init_db());
    }
    
    fn init_db() -> Option<Db> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "xilulu", "app") {
            let db_path = proj_dirs.data_dir().join("storage_db");
            tracing::debug!(">>> Init DB at: {:?}", db_path);
            // Ensure the directory exists
            if let Some(parent) = db_path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    tracing::error!(">>> Failed to create DB dir: {:?}", e);
                }
            }
            match sled::open(&db_path) {
                Ok(db) => {
                    tracing::debug!(">>> Sled DB opened successfully!");
                    return Some(db);
                },
                Err(e) => {
                    tracing::error!(">>> Failed to open Sled DB: {:?}", e);
                }
            }
        } else {
            tracing::error!(">>> ProjectDirs::from returned None! Storage will not be persistent.");
        }
        None
    }


    pub fn get(key: &str) -> Option<String> {
        if let Ok(guard) = DB.lock() {
            if let Some(db) = &*guard {
                if let Ok(Some(ivec)) = db.get(key) {
                    return String::from_utf8(ivec.to_vec()).ok();
                }
            }
        }
        None
    }

    pub fn set(key: &str, value: &str) {
        if let Ok(guard) = DB.lock() {
            if let Some(db) = &*guard {
                let _ = db.insert(key, value.as_bytes());
                let _ = db.flush();
            }
        }
    }
    
    pub fn remove(key: &str) {
        if let Ok(guard) = DB.lock() {
            if let Some(db) = &*guard {
                let _ = db.remove(key);
                let _ = db.flush();
            }
        }
    }
}
