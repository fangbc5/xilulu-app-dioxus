pub mod auth;
pub mod common;
pub mod im;
pub mod oss;

pub use auth::*;
pub use common::*;
pub use im::*;
pub use oss::*;

// Export models so stale frb_generated.rs can find XEvent until regenerated

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use xilulu_core::api::client::ApiClient;
use xilulu_core::port::StorageProvider;

/// 移动端 Token 内存存储适配器。
/// 当 access_token 被写入（非空）时，自动向内部广播通道发出信号，
/// 供 im.rs 中的监听任务将 TOKEN_REFRESHED 事件推送给 Flutter。
pub struct MobileStorageAdapter {
    store: Arc<RwLock<HashMap<String, String>>>,
    token_update_tx: broadcast::Sender<()>,
}

impl MobileStorageAdapter {
    pub fn new() -> Self {
        // 广播通道容量 16，足够并发刷新场景
        let (token_update_tx, _) = broadcast::channel(16);
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            token_update_tx,
        }
    }

    /// 订阅 token 更新信号。每次 access_token 非空写入时触发一次。
    /// pub(crate) 避免被 flutter_rust_bridge_codegen 误暴露到 FFI 边界。
    pub(crate) fn subscribe_token_updates(&self) -> broadcast::Receiver<()> {
        self.token_update_tx.subscribe()
    }
}

#[async_trait::async_trait]
impl StorageProvider for MobileStorageAdapter {
    async fn set(&self, key: &str, val: &str) -> Result<(), String> {
        self.store
            .write()
            .await
            .insert(key.to_string(), val.to_string());
        // access_token 非空写入时广播通知（登录成功 + Token 刷新都会触发）
        if key == "access_token" && !val.is_empty() {
            let _ = self.token_update_tx.send(());
        }
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
    pub static ref GLOBAL_STORAGE: Arc<MobileStorageAdapter> = Arc::new(MobileStorageAdapter::new());

    pub static ref GLOBAL_API_CLIENT: ApiClient = {
        ApiClient::new(GLOBAL_STORAGE.clone())
    };
}
