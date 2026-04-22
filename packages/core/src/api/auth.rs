use serde::{Deserialize, Serialize};
use crate::api::client::{ApiClient, BASE_URL, ApiError};

#[derive(Serialize)]
pub struct LoginRequest<'a> {
    pub username: Option<&'a str>,
    pub password: Option<&'a str>,
    pub mobile: Option<&'a str>,
    pub email: Option<&'a str>,
    pub code: Option<&'a str>,
    pub captcha_id: Option<&'a str>,
    pub captcha: Option<&'a str>,
    pub region: Option<&'a str>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UserInfo {
    pub id: String,
    pub nickname: String,
    pub avatar: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TenantInfo {
    pub id: String,
    pub name: String,
    pub is_owner: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub refresh_expires_in: i64,
    pub user_info: UserInfo,
    pub tenant_list: Option<Vec<TenantInfo>>,
}

#[derive(Serialize)]
pub struct RegisterRequest<'a> {
    pub username: Option<&'a str>,
    pub password: Option<&'a str>,
    pub mobile: Option<&'a str>,
    pub email: Option<&'a str>,
    pub code: Option<&'a str>,
    pub captcha_id: Option<&'a str>,
    pub captcha: Option<&'a str>,
    pub nick_name: Option<&'a str>,
    pub avatar: Option<&'a str>,
    pub region: Option<&'a str>,
}

#[derive(Serialize)]
pub struct LoginOrRegisterRequest<'a> {
    pub mobile: Option<&'a str>,
    pub email: Option<&'a str>,
    pub code: &'a str,
    pub region: Option<&'a str>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LoginOrRegisterResponse {
    pub is_new_user: bool,
    pub login_info: LoginResponse,
}

#[derive(Serialize)]
pub struct SendVerifyCodeRequest<'a> {
    pub account: &'a str,
}

#[derive(Deserialize, Debug)]
pub struct ImageCaptchaResponse {
    pub captcha_id: String,
    pub image_base64: String,
}

pub async fn login(api: &ApiClient, req: LoginRequest<'_>) -> Result<LoginResponse, ApiError> {
    let url = format!("{}/api/v1/auth/login", BASE_URL);
    let builder = api.client().post(&url).json(&req);
    api.send_request(builder).await
}

pub async fn register(api: &ApiClient, req: RegisterRequest<'_>) -> Result<(), ApiError> {
    let url = format!("{}/api/v1/auth/register", BASE_URL);
    let builder = api.client().post(&url).json(&req);
    api.send_action(builder).await
}

pub async fn fetch_captcha(api: &ApiClient) -> Result<ImageCaptchaResponse, ApiError> {
    let url = format!("{}/api/v1/auth/captcha", BASE_URL);
    let builder = api.client().get(&url);
    api.send_request(builder).await
}

pub async fn login_or_register(api: &ApiClient, req: LoginOrRegisterRequest<'_>) -> Result<LoginOrRegisterResponse, ApiError> {
    let url = format!("{}/api/v1/auth/login-or-register", BASE_URL);
    let builder = api.client().post(&url).json(&req);
    api.send_request(builder).await
}

pub async fn send_verify_code(api: &ApiClient, account: &str) -> Result<(), ApiError> {
    let url = format!("{}/api/v1/auth/send-code", BASE_URL);
    let builder = api.client().post(&url).json(&SendVerifyCodeRequest { account });
    api.send_action(builder).await
}

pub async fn logout(api: &ApiClient) -> Result<(), ApiError> {
    let url = format!("{}/api/v1/auth/logout", BASE_URL);
    let builder = api.client().post(&url);
    let auth_builder = api.inject_auth(builder).await;
    api.send_action(auth_builder).await
}
