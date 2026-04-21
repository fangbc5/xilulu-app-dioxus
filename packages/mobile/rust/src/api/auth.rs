use crate::api::{GLOBAL_API_CLIENT, GLOBAL_STORAGE};
use xilulu_core::api::auth::{
    login, login_or_register, logout, register, send_verify_code, LoginOrRegisterRequest,
    LoginRequest, RegisterRequest,
};
use xilulu_core::port::StorageProvider;

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
            // 同时保存 user_id，供 WS 同步任务读取 my_uid
            let _ = GLOBAL_STORAGE.set("user_id", &resp.user_info.id).await;
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
            let _ = GLOBAL_STORAGE
                .set("user_id", &resp.login_info.user_info.id)
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

pub async fn core_logout() -> Result<(), String> {
    match logout(&GLOBAL_API_CLIENT).await {
        Ok(_) => {
            let _ = GLOBAL_STORAGE.remove("access_token").await;
            let _ = GLOBAL_STORAGE.remove("refresh_token").await;
            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}
