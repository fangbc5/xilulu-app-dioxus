//! localStorage 持久化工具
//!
//! 保存认证信息到浏览器 localStorage，支持页面刷新后恢复登录态。

use crate::services::auth::UserInfo;

/// localStorage 键名
const LS_ACCESS_TOKEN: &str = "xilulu_access_token";
const LS_REFRESH_TOKEN: &str = "xilulu_refresh_token";
const LS_USER_INFO: &str = "xilulu_user_info";

/// 从 localStorage 读取字符串
pub fn ls_get(key: &str) -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .and_then(|s| s.get_item(key).ok())
        .flatten()
        .filter(|v| !v.is_empty())
}

/// 写入 localStorage
fn ls_set(key: &str, value: &str) {
    let _ = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .map(|s| s.set_item(key, value));
}

/// 清除认证相关的 localStorage
pub fn clear_auth() {
    let storage = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten();
    if let Some(s) = storage {
        let _ = s.remove_item(LS_ACCESS_TOKEN);
        let _ = s.remove_item(LS_REFRESH_TOKEN);
        let _ = s.remove_item(LS_USER_INFO);
    }
}

/// 保存认证信息到 localStorage
pub fn save_auth(access_token: &str, refresh_token: &str, user_info: &UserInfo) {
    ls_set(LS_ACCESS_TOKEN, access_token);
    ls_set(LS_REFRESH_TOKEN, refresh_token);
    if let Ok(json) = serde_json::to_string(user_info) {
        ls_set(LS_USER_INFO, &json);
    }
}

/// 从 localStorage 恢复认证状态，返回 (access_token, refresh_token, user_info)
pub fn restore_auth() -> (String, String, Option<UserInfo>) {
    let access = ls_get(LS_ACCESS_TOKEN).unwrap_or_default();
    let refresh = ls_get(LS_REFRESH_TOKEN).unwrap_or_default();
    let user: Option<UserInfo> = ls_get(LS_USER_INFO)
        .and_then(|json| serde_json::from_str(&json).ok());
    (access, refresh, user)
}