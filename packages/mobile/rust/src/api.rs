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
    region: Option<String>,
) -> Result<String, String> {
    let is_email = account.contains('@');
    let is_mobile = account.chars().all(|c| c.is_ascii_digit()) && account.len() >= 5;
    
    let mut mobile = None;
    let mut email = None;
    let mut username = None;
    
    if is_email {
        email = Some(account.as_str());
    } else if is_mobile {
        mobile = Some(account.as_str());
    } else {
        username = Some(account.as_str());
    }

    let req = LoginRequest {
        mobile,
        password: password.as_deref(),
        username,
        email,
        code: None,
        captcha_id: None,
        captcha: None,
        region: region.as_deref(),
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
    region: Option<String>,
) -> Result<String, String> {
    let req = LoginOrRegisterRequest {
        mobile: Some(&mobile),
        email: None,
        code: &code,
        region: region.as_deref(),
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
    account: String,
    password: Option<String>,
    nickname: Option<String>,
    avatar: Option<String>,
    region: Option<String>,
) -> Result<(), String> {
    let is_email = account.contains('@');
    let is_mobile = account.chars().all(|c| c.is_ascii_digit()) && account.len() >= 5;
    
    let mut mobile = None;
    let mut email = None;
    let mut username = None;
    
    if is_email {
        email = Some(account.as_str());
    } else if is_mobile {
        mobile = Some(account.as_str());
    } else {
        username = Some(account.as_str());
    }

    let req = RegisterRequest {
        mobile,
        password: password.as_deref(),
        nick_name: nickname.as_deref(),
        username,
        email,
        code: None,
        captcha_id: None,
        captcha: None,
        avatar: avatar.as_deref(),
        region: region.as_deref(),
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

pub async fn core_upload_file(
    file_bytes: Vec<u8>,
    filename: String,
    scene: String,
) -> Result<String, String> {
    let content_type = if filename.ends_with(".png") {
        Some("image/png")
    } else if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
        Some("image/jpeg")
    } else if filename.ends_with(".webp") {
        Some("image/webp")
    } else {
        None
    };

    use xilulu_core::api::oss::upload_file;
    match upload_file(
        &GLOBAL_API_CLIENT,
        file_bytes,
        &filename,
        &scene,
        content_type,
    )
    .await
    {
        Ok(res) => serde_json::to_string(&res.meta).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}
