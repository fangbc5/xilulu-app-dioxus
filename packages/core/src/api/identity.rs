use serde::{Deserialize, Serialize};
use crate::api::client::{ApiClient, BASE_URL, ApiError};

#[derive(Serialize)]
pub struct UpdateProfileRequest<'a> {
    pub nickname: Option<&'a str>,
    pub gender: Option<&'a str>,
    pub avatar_url: Option<&'a str>,
}

pub async fn update_profile(api: &ApiClient, user_id: &str, req: UpdateProfileRequest<'_>) -> Result<(), ApiError> {
    let url = format!("{}/api/v1/identity/users/{}", BASE_URL, user_id);
    let builder = api.client().put(&url).json(&req);
    let auth_builder = api.inject_auth(builder).await;
    api.send_action(auth_builder).await
}

/// 用户资料信息
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub nick_name: Option<String>,
    pub face_url: Option<String>,
    pub self_signature: Option<String>,
    pub gender: Option<i32>,
    pub birthday: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

/// 获取用户信息
/// 调用 GET /api/v1/identity/users/{user_id}
pub async fn get_user_info(api: &ApiClient, user_id: i64) -> Result<UserInfo, String> {
    let url = format!("{}/api/v1/identity/users/{}", BASE_URL, user_id);
    let builder = api.client().get(&url);
    let auth_builder = api.inject_auth(builder).await;

    api.send_request::<UserInfo>(auth_builder)
        .await
        .map_err(|e| e.to_string())
}

/// 更新用户信息
/// 调用 PUT /api/v1/identity/users/{user_id}
pub async fn update_user_info(
    api: &ApiClient,
    user_id: i64,
    nick_name: Option<String>,
    face_url: Option<String>,
    self_signature: Option<String>,
    gender: Option<i32>,
) -> Result<UserInfo, String> {
    let url = format!("{}/api/v1/identity/users/{}", BASE_URL, user_id);

    #[derive(Serialize)]
    struct UpdateUserRequest {
        #[serde(skip_serializing_if = "Option::is_none")]
        nick_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        face_url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        self_signature: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        gender: Option<i32>,
    }

    let payload = UpdateUserRequest {
        nick_name,
        face_url,
        self_signature,
        gender,
    };

    let builder = api.client().put(&url).json(&payload);
    let auth_builder = api.inject_auth(builder).await;

    api.send_request::<UserInfo>(auth_builder)
        .await
        .map_err(|e| e.to_string())
}

/// 批量获取用户信息
/// 调用 POST /api/v1/identity/users/batch
pub async fn get_users_info(api: &ApiClient, user_ids: Vec<i64>) -> Result<Vec<UserInfo>, String> {
    let url = format!("{}/api/v1/identity/users/batch", BASE_URL);

    #[derive(Serialize)]
    struct BatchRequest {
        user_ids: Vec<i64>,
    }

    let payload = BatchRequest { user_ids };
    let builder = api.client().post(&url).json(&payload);
    let auth_builder = api.inject_auth(builder).await;

    api.send_request::<Vec<UserInfo>>(auth_builder)
        .await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ResourceInfo {
    pub id: Option<i32>,
    pub application_id: i32,
    pub code: String,
    pub name: String,
    pub parent_id: i32,
    pub resource_type: Option<String>,
    pub path: Option<String>,
    pub component: Option<String>,
    pub icon: Option<String>,
    #[serde(rename = "describe_")]
    pub describe: Option<String>,
    pub state: Option<bool>,
}

pub async fn get_user_menus(api: &ApiClient, app_id: i32, tenant_id: Option<i32>) -> Result<Vec<ResourceInfo>, ApiError> {
    let mut url = format!("{}/api/v1/identity/resources/menus?application_id={}", BASE_URL, app_id);
    if let Some(t_id) = tenant_id {
        url = format!("{}&tenant_id={}", url, t_id);
    }
    let builder = api.client().get(&url);
    let auth_builder = api.inject_auth(builder).await;
    api.send_request(auth_builder).await
}
