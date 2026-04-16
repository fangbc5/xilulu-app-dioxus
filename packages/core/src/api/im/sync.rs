use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::{info, error};

use crate::db::DbManager;
use crate::api::client::ApiClient;
use super::models::XEvent;

/// 同步结构体：这些与 xilulu-server 的实体完全对应
#[derive(Debug, Deserialize, Clone)]
pub struct ServerUserFriend {
    pub uid: i64,
    pub friend_uid: i64,
    pub remark: Option<String>,
    pub status: i16,
    pub updated_at: String, // from DateTime<Utc>
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerContact {
    pub room_id: i64,
    pub room_type: i16,
    pub unread_count: i64,
    pub last_msg_id: Option<i64>,
    pub active_time: Option<String>,
    pub is_mute: i16,
    pub is_top: i16,
    pub is_deleted: i16,
    pub read_msg_id: Option<i64>,
    pub clear_msg_id: i64,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerRoom {
    pub id: i64,
    pub r#type: i16,
    pub active_time: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerRoomFriend {
    pub room_id: i64,
    pub uid1: i64,
    pub uid2: i64,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerRoomGroup {
    pub id: i64,
    pub room_id: i64,
    pub name: String,
    pub avatar: Option<String>,
    pub notice: Option<String>,
    pub is_deleted: i16,
    pub created_by: i64,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerGroupMember {
    pub group_id: i64,
    pub uid: i64,
    pub role: i16,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SyncResponse {
    pub friends: Vec<ServerUserFriend>,
    pub contacts: Vec<ServerContact>,
    pub rooms: Vec<ServerRoom>,
    pub room_friends: Vec<ServerRoomFriend>,
    pub room_groups: Vec<ServerRoomGroup>,
    pub group_members: Vec<ServerGroupMember>,
}

/// 执行增量同步
pub async fn execute_sync(api: &ApiClient, db: &DbManager, my_uid: i64) -> Result<(), String> {
    // 1. 读取最后一次的 Sync Timestamp
    let row: Option<(i64,)> = sqlx::query_as("SELECT timestamp_ms FROM sync_cursors WHERE module = 'global'")
        .fetch_optional(&db.pool)
        .await
        .map_err(|e| e.to_string())?;

    let since_ts = row.map(|r| r.0).unwrap_or(0);

    // 2. 发起 HTTP 增量拉取请求
    let ts_param = since_ts.to_string();
    let url = format!("{}/api/v1/im/sync", crate::api::client::BASE_URL);
    let req = api.client().get(&url).query(&[("since_ts", &ts_param)]);
    let req = api.inject_auth(req).await;
    let resp: SyncResponse = api.send_request(req).await.map_err(|e| e.to_string())?;

    // 3. 落库处理 (如果有变动)
    if resp.friends.is_empty() && resp.contacts.is_empty() && resp.room_groups.is_empty() {
        return Ok(());
    }

    let mut tx = db.pool.begin().await.map_err(|e| e.to_string())?;

    for friend in resp.friends {
        let other_uid = if friend.uid == my_uid { friend.friend_uid } else { friend.uid };
        let dt = chrono::DateTime::parse_from_rfc3339(&friend.updated_at)
            .map(|d| d.timestamp_millis())
            .unwrap_or(0);
        sqlx::query(
            "INSERT INTO friends (uid, remark, status, updated_at) VALUES (?, ?, ?, ?)
             ON CONFLICT(uid) DO UPDATE SET remark=excluded.remark, status=excluded.status, updated_at=excluded.updated_at"
        )
        .bind(other_uid)
        .bind(friend.remark)
        .bind(friend.status)
        .bind(dt)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    // 更新 contacts 和映射关系
    let mut rf_map = std::collections::HashMap::new();
    for rf in resp.room_friends {
        let other_uid = if rf.uid1 == my_uid { rf.uid2 } else { rf.uid1 };
        rf_map.insert(rf.room_id, other_uid);
    }

    for c in resp.contacts {
        // Find friend_uid for C2C chats. If we didn't receive room_friend in this payload, query the db to preserve it if it already exists, or we might miss it.
        // Or if it's new, we hopefully got it in room_friends.
        let f_uid = rf_map.get(&c.room_id).copied();

        sqlx::query(
            "INSERT INTO contacts (room_id, room_type, unread_count, is_mute, is_top, is_deleted, friend_uid, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, COALESCE(?, (SELECT friend_uid FROM contacts WHERE room_id = ?)), ?)
             ON CONFLICT(room_id) DO UPDATE SET 
                is_mute=excluded.is_mute, is_top=excluded.is_top, is_deleted=excluded.is_deleted, 
                friend_uid=COALESCE(excluded.friend_uid, contacts.friend_uid), updated_at=excluded.updated_at, unread_count=excluded.unread_count"
        )
        .bind(c.room_id)
        .bind(c.room_type)
        .bind(c.unread_count)
        .bind(c.is_mute)
        .bind(c.is_top)
        .bind(c.is_deleted)
        .bind(f_uid)
        .bind(c.room_id)
        .bind(chrono::Utc::now().timestamp_millis())
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    for rg in resp.room_groups {
        sqlx::query(
            "INSERT INTO groups (group_id, room_id, name, avatar, notice, is_deleted, owner_uid, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(group_id) DO UPDATE SET name=excluded.name, avatar=excluded.avatar, notice=excluded.notice, is_deleted=excluded.is_deleted, owner_uid=excluded.owner_uid, updated_at=excluded.updated_at"
        )
        .bind(rg.id)
        .bind(rg.room_id)
        .bind(rg.name)
        .bind(rg.avatar)
        .bind(rg.notice)
        .bind(rg.is_deleted)
        .bind(rg.created_by)
        .bind(chrono::Utc::now().timestamp_millis())
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    // 全量替换收到的有变动的群的群成员
    let mut modified_groups = std::collections::HashSet::new();
    for gm in &resp.group_members {
        modified_groups.insert(gm.group_id);
    }
    for mg_id in modified_groups {
        sqlx::query("DELETE FROM group_members WHERE group_id = ?")
            .bind(mg_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    for gm in resp.group_members {
        sqlx::query(
            "INSERT INTO group_members (group_id, uid, role, updated_at) VALUES (?, ?, ?, ?)"
        )
        .bind(gm.group_id)
        .bind(gm.uid)
        .bind(gm.role)
        .bind(chrono::Utc::now().timestamp_millis())
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    // 保存 Cursor
    let current_ms = chrono::Utc::now().timestamp_millis();
    sqlx::query("INSERT INTO sync_cursors (module, timestamp_ms) VALUES ('global', ?) ON CONFLICT(module) DO UPDATE SET timestamp_ms=excluded.timestamp_ms")
        .bind(current_ms)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    info!("同步任务完成");
    Ok(())
}
