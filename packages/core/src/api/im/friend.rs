use crate::api::client::ApiClient;
use serde::{Deserialize, Serialize};
use crate::db::DbManager;

/// 后端返回的好友信息
#[derive(Debug, Deserialize, Serialize)]
pub struct FriendInfo {
    pub user_id: i64,
    pub nick_name: Option<String>,
    pub face_url: Option<String>,
    pub friend_remark: Option<String>,
    pub created_at: Option<i64>,
}

/// 后端返回的用户搜索结果
#[derive(Debug, Deserialize, Serialize)]
pub struct UserSearchResult {
    pub user_id: i64,
    pub nick_name: Option<String>,
    pub face_url: Option<String>,
}

/// 后端返回的好友申请
#[derive(Debug, Deserialize, Serialize)]
pub struct FriendApplication {
    pub id: i64,
    pub from_uid: i64,
    pub to_uid: i64,
    pub message: Option<String>,
    pub status: i32, // 0: 待处理, 1: 已同意, 2: 已拒绝
    pub created_at: i64,
    pub from_user: Option<UserSearchResult>,
}

/// 获取好友列表（仍然提供在线拉取，但通常使用 local 方法）
/// 调用 GET /api/v1/im/friends
pub async fn list_friends(api: &ApiClient) -> Result<Vec<FriendInfo>, String> {
    let url = format!("{}/api/v1/im/friends", crate::api::client::BASE_URL);
    let builder = api.client().get(&url);
    let auth_builder = api.inject_auth(builder).await;

    api.send_request::<Vec<FriendInfo>>(auth_builder)
        .await
        .map_err(|e| e.to_string())
}

/// 从本地 SQLite 数据库读取好友列表（P4 强一致性本地缓存架构）
pub async fn list_friends_local(db: &DbManager) -> Result<Vec<FriendInfo>, String> {
    // 关联好友表和用户画像表，只返回状态为正常的 (status=1)
    let rows: Vec<(i64, Option<String>, i64, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT f.uid, f.remark, f.updated_at, u.nick_name, u.avatar
         FROM friends f
         LEFT JOIN user_profiles u ON f.uid = u.uid
         WHERE f.status = 1"
    )
    .fetch_all(&db.pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut res = Vec::new();
    for (uid, remark, updated_at, nick_name, avatar) in rows {
        res.push(FriendInfo {
            user_id: uid,
            nick_name,
            face_url: avatar,
            friend_remark: remark,
            created_at: Some(updated_at),
        });
    }

    Ok(res)
}

/// 搜索用户
/// 调用 GET /api/v1/im/friends/search-user?keyword={keyword}
pub async fn search_user(api: &ApiClient, keyword: &str) -> Result<Vec<UserSearchResult>, String> {
    let url = format!(
        "{}/api/v1/im/friends/search-user?keyword={}",
        crate::api::client::BASE_URL,
        urlencoding::encode(keyword)
    );
    let builder = api.client().get(&url);
    let auth_builder = api.inject_auth(builder).await;

    api.send_request::<Vec<UserSearchResult>>(auth_builder)
        .await
        .map_err(|e| e.to_string())
}

/// 添加好友（发送好友申请）
/// 调用 POST /api/v1/im/friends/applies
pub async fn add_friend(api: &ApiClient, target_uid: i64, message: Option<String>) -> Result<(), String> {
    let url = format!("{}/api/v1/im/friends/applies", crate::api::client::BASE_URL);

    #[derive(Serialize)]
    struct AddFriendRequest {
        to_uid: i64,
        message: Option<String>,
    }

    let payload = AddFriendRequest {
        to_uid: target_uid,
        message,
    };

    let builder = api.client().post(&url).json(&payload);
    let auth_builder = api.inject_auth(builder).await;

    api.send_action(auth_builder)
        .await
        .map_err(|e| e.to_string())
}

/// 删除好友
/// 调用 DELETE /api/v1/im/friends/{target_uid}
pub async fn delete_friend(api: &ApiClient, target_uid: i64) -> Result<(), String> {
    let url = format!(
        "{}/api/v1/im/friends/{}",
        crate::api::client::BASE_URL,
        target_uid
    );
    let builder = api.client().delete(&url);
    let auth_builder = api.inject_auth(builder).await;

    api.send_action(auth_builder)
        .await
        .map_err(|e| e.to_string())
}

/// 获取好友申请列表
/// 调用 GET /api/v1/im/friends/applies
pub async fn list_friend_applies(api: &ApiClient) -> Result<Vec<FriendApplication>, String> {
    let url = format!("{}/api/v1/im/friends/applies", crate::api::client::BASE_URL);
    let builder = api.client().get(&url);
    let auth_builder = api.inject_auth(builder).await;

    api.send_request::<Vec<FriendApplication>>(auth_builder)
        .await
        .map_err(|e| e.to_string())
}

/// 同意好友申请
/// 调用 POST /api/v1/im/friends/applies/{apply_id}/approve
pub async fn approve_apply(api: &ApiClient, apply_id: i64) -> Result<(), String> {
    let url = format!(
        "{}/api/v1/im/friends/applies/{}/approve",
        crate::api::client::BASE_URL,
        apply_id
    );
    let builder = api.client().post(&url);
    let auth_builder = api.inject_auth(builder).await;

    api.send_action(auth_builder)
        .await
        .map_err(|e| e.to_string())
}

/// 拒绝好友申请
/// 调用 POST /api/v1/im/friends/applies/{apply_id}/reject
pub async fn reject_apply(api: &ApiClient, apply_id: i64) -> Result<(), String> {
    let url = format!(
        "{}/api/v1/im/friends/applies/{}/reject",
        crate::api::client::BASE_URL,
        apply_id
    );
    let builder = api.client().post(&url);
    let auth_builder = api.inject_auth(builder).await;

    api.send_action(auth_builder)
        .await
        .map_err(|e| e.to_string())
}
