pub mod auth;
pub mod common;
pub mod im;
pub mod oss;

pub use auth::*;
pub use common::*;
pub use im::*;
pub use oss::*;

// Export models so stale frb_generated.rs can find XEvent until regenerated
pub use xilulu_core::api::im::models::*;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use xilulu_core::api::client::ApiClient;
use xilulu_core::port::StorageProvider;

pub struct MobileStorageAdapter {
    store: Arc<RwLock<HashMap<String, String>>>,
}

#[async_trait::async_trait]
impl StorageProvider for MobileStorageAdapter {
    async fn set(&self, key: &str, val: &str) -> Result<(), String> {
        self.store
            .write()
            .await
            .insert(key.to_string(), val.to_string());
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<String>, String> {
        Ok(self.store.read().await.get(key).cloned())
    }

    async fn remove(&self, key: &str) -> Result<(), String> {
        self.store.write().await.remove(key);
        Ok(())
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_STORAGE: Arc<MobileStorageAdapter> = Arc::new(MobileStorageAdapter {
        store: Arc::new(RwLock::new(HashMap::new()))
    });

    pub static ref GLOBAL_API_CLIENT: ApiClient = {
        ApiClient::new(GLOBAL_STORAGE.clone())
    };
}
