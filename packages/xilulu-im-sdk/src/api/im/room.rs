use crate::api::client::ApiClient;
use serde::{Deserialize, Serialize};
use crate::db::DbManager;

/// 群组信息
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GroupInfo {
    pub id: i64,
    pub name: String,
    pub owner_uid: i64,
    pub face_url: Option<String>,
    pub introduction: Option<String>,
    pub notification: Option<String>,
    pub member_count: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 群成员信息
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GroupMember {
    pub user_id: i64,
    pub group_id: i64,
    pub role: i32, // 0: 普通成员, 1: 管理员, 2: 群主
    pub nick_name: Option<String>,
    pub face_url: Option<String>,
    pub joined_at: i64,
}

/// 创建群组
/// 调用 POST /api/v1/im/rooms/groups
pub async fn create_group(
    api: &ApiClient,
    name: String,
    member_uids: Vec<i64>,
    introduction: Option<String>,
) -> Result<GroupInfo, String> {
    let url = format!("{}/api/v1/im/rooms/groups", crate::api::client::BASE_URL);

    #[derive(Serialize)]
    struct CreateGroupRequest {
        name: String,
        member_uids: Vec<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        introduction: Option<String>,
    }

    let payload = CreateGroupRequest {
        name,
        member_uids,
        introduction,
    };

    let builder = api.client().post(&url).json(&payload);
    let auth_builder = api.inject_auth(builder).await;

    api.send_request::<GroupInfo>(auth_builder)
        .await
        .map_err(|e| e.to_string())
}

/// 获取群组信息
/// 调用 GET /api/v1/im/rooms/groups/{group_id}
pub async fn get_group_info(api: &ApiClient, group_id: i64) -> Result<GroupInfo, String> {
    let url = format!(
        "{}/api/v1/im/rooms/groups/{}",
        crate::api::client::BASE_URL,
        group_id
    );
    let builder = api.client().get(&url);
    let auth_builder = api.inject_auth(builder).await;

    api.send_request::<GroupInfo>(auth_builder)
        .await
        .map_err(|e| e.to_string())
}

/// 从本地 SQLite 数据库获取群组信息（P4 强一致性本地缓存架构）
pub async fn get_group_info_local(db: &DbManager, group_id: i64) -> Result<GroupInfo, String> {
    let row: Option<(i64, String, Option<i64>, Option<String>, Option<String>, i64, i32)> = sqlx::query_as(
        r#"
        SELECT g.id, g.name, g.created_by, g.avatar, g.notice, g.updated_at,
               (SELECT COUNT(*) FROM group_member WHERE group_id = g.id) as member_count
        FROM room_group g
        WHERE g.id = ? AND g.is_deleted = 0
        "#
    )
    .bind(group_id as i64)
    .fetch_optional(&db.pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some((gid, name, created_by, avatar, notice, updated_at, member_count)) = row {
        Ok(GroupInfo {
            id: gid,
            name,
            owner_uid: created_by.unwrap_or(0),
            face_url: avatar,
            introduction: None,
            notification: notice,
            member_count,
            created_at: 0,
            updated_at,
        })
    } else {
        Err(format!("Group {} not found locally", group_id))
    }
}

/// 获取群成员列表
/// 调用 GET /api/v1/im/rooms/groups/{group_id}/members
pub async fn list_group_members(api: &ApiClient, group_id: i64) -> Result<Vec<GroupMember>, String> {
    let url = format!(
        "{}/api/v1/im/rooms/groups/{}/members",
        crate::api::client::BASE_URL,
        group_id
    );
    let builder = api.client().get(&url);
    let auth_builder = api.inject_auth(builder).await;

    api.send_request::<Vec<GroupMember>>(auth_builder)
        .await
        .map_err(|e| e.to_string())
}

/// 从本地 SQLite 获取群成员列表（P4 强一致性本地缓存架构）
pub async fn list_group_members_local(db: &DbManager, group_id: i64) -> Result<Vec<GroupMember>, String> {
    let rows: Vec<(i64, i64, i64, i64, Option<String>, Option<String>)> = sqlx::query_as(
        r#"
        SELECT gm.uid, gm.group_id, gm.role, gm.updated_at, u.nick_name, u.avatar
        FROM group_member gm
        LEFT JOIN user_profile u ON gm.uid = u.uid
        WHERE gm.group_id = ?
        "#
    )
    .bind(group_id as i64)
    .fetch_all(&db.pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut res = Vec::new();
    for (uid, gid, role, updated_at, nick_name, avatar) in rows {
        res.push(GroupMember {
            user_id: uid,
            group_id: gid,
            role: role as i32,
            nick_name,
            face_url: avatar,
            joined_at: updated_at,
        });
    }

    Ok(res)
}

/// 退出群组
/// 调用 POST /api/v1/im/rooms/groups/{group_id}/quit
pub async fn quit_group(api: &ApiClient, group_id: i64) -> Result<(), String> {
    let url = format!(
        "{}/api/v1/im/rooms/groups/{}/quit",
        crate::api::client::BASE_URL,
        group_id
    );
    let builder = api.client().post(&url);
    let auth_builder = api.inject_auth(builder).await;

    api.send_action(auth_builder)
        .await
        .map_err(|e| e.to_string())
}

/// 邀请成员加入群组
/// 调用 POST /api/v1/im/rooms/groups/{group_id}/members
pub async fn invite_members(
    api: &ApiClient,
    group_id: i64,
    member_uids: Vec<i64>,
) -> Result<(), String> {
    let url = format!(
        "{}/api/v1/im/rooms/groups/{}/members",
        crate::api::client::BASE_URL,
        group_id
    );

    #[derive(Serialize)]
    struct InviteMembersRequest {
        member_uids: Vec<i64>,
    }

    let payload = InviteMembersRequest { member_uids };
    let builder = api.client().post(&url).json(&payload);
    let auth_builder = api.inject_auth(builder).await;

    api.send_action(auth_builder)
        .await
        .map_err(|e| e.to_string())
}

/// 踢出群成员
/// 调用 DELETE /api/v1/im/rooms/groups/{group_id}/members/{user_id}
pub async fn kick_member(api: &ApiClient, group_id: i64, user_id: i64) -> Result<(), String> {
    let url = format!(
        "{}/api/v1/im/rooms/groups/{}/members/{}",
        crate::api::client::BASE_URL,
        group_id,
        user_id
    );
    let builder = api.client().delete(&url);
    let auth_builder = api.inject_auth(builder).await;

    api.send_action(auth_builder)
        .await
        .map_err(|e| e.to_string())
}

/// 更新群组信息
/// 调用 PUT /api/v1/im/rooms/groups/{group_id}
pub async fn update_group_info(
    api: &ApiClient,
    group_id: i64,
    name: Option<String>,
    face_url: Option<String>,
    introduction: Option<String>,
    notification: Option<String>,
) -> Result<GroupInfo, String> {
    let url = format!(
        "{}/api/v1/im/rooms/groups/{}",
        crate::api::client::BASE_URL,
        group_id
    );

    #[derive(Serialize)]
    struct UpdateGroupRequest {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        face_url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        introduction: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        notification: Option<String>,
    }

    let payload = UpdateGroupRequest {
        name,
        face_url,
        introduction,
        notification,
    };

    let builder = api.client().put(&url).json(&payload);
    let auth_builder = api.inject_auth(builder).await;

    api.send_request::<GroupInfo>(auth_builder)
        .await
        .map_err(|e| e.to_string())
}

/// 解散群组
/// 调用 DELETE /api/v1/im/rooms/groups/{group_id}
pub async fn dismiss_group(api: &ApiClient, group_id: i64) -> Result<(), String> {
    let url = format!(
        "{}/api/v1/im/rooms/groups/{}",
        crate::api::client::BASE_URL,
        group_id
    );
    let builder = api.client().delete(&url);
    let auth_builder = api.inject_auth(builder).await;

    api.send_action(auth_builder)
        .await
        .map_err(|e| e.to_string())
}
