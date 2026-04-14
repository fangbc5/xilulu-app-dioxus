use flutter_rust_bridge::frb;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use xilulu_core::api::auth::{
    login, login_or_register, register, send_verify_code, LoginOrRegisterRequest, LoginRequest,
    RegisterRequest,
};
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

#[frb(sync)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}

pub async fn ping_core() -> String {
    "Pong from Xilulu Core!".to_string()
}

pub async fn core_login_with_pwd(
    account: String,
    password: Option<String>,
) -> Result<String, String> {
    let req = LoginRequest {
        mobile: Some(&account),
        password: password.as_deref(),
        username: None,
        email: None,
        code: None,
        captcha_id: None,
        captcha: None,
    };
    match login(&GLOBAL_API_CLIENT, req).await {
        Ok(resp) => {
            let _ = GLOBAL_STORAGE.set("access_token", &resp.access_token).await;
            let _ = GLOBAL_STORAGE
                .set("refresh_token", &resp.refresh_token)
                .await;
            serde_json::to_string(&resp).map_err(|e| e.to_string())
        }
        Err(e) => Err(e.to_string()),
    }
}

pub async fn core_login_or_register_by_code(
    mobile: String,
    code: String,
) -> Result<String, String> {
    let req = LoginOrRegisterRequest {
        mobile: Some(&mobile),
        email: None,
        code: &code,
    };
    match login_or_register(&GLOBAL_API_CLIENT, req).await {
        Ok(resp) => {
            let _ = GLOBAL_STORAGE
                .set("access_token", &resp.login_info.access_token)
                .await;
            let _ = GLOBAL_STORAGE
                .set("refresh_token", &resp.login_info.refresh_token)
                .await;
            serde_json::to_string(&resp).map_err(|e| e.to_string())
        }
        Err(e) => Err(e.to_string()),
    }
}

pub async fn core_register(
    mobile: String,
    password: Option<String>,
    nickname: Option<String>,
) -> Result<(), String> {
    let req = RegisterRequest {
        mobile: Some(&mobile),
        password: password.as_deref(),
        nick_name: nickname.as_deref(),
        username: None,
        email: None,
        code: None,
        captcha_id: None,
        captcha: None,
    };
    register(&GLOBAL_API_CLIENT, req)
        .await
        .map_err(|e| e.to_string())
}

pub async fn core_send_verify_code(mobile: String) -> Result<(), String> {
    send_verify_code(&GLOBAL_API_CLIENT, &mobile)
        .await
        .map_err(|e| e.to_string())
}
