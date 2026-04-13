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
