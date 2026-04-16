use super::models::{XContact, XMessage};
use crate::api::client::ApiClient;
use serde::{Deserialize, Serialize};
use crate::db::DbManager;

/// 后端返回的会话列表项
#[derive(Debug, Deserialize)]
struct ContactResponse {
    room_id: i64,
    room_type: i32,
    unread_count: i32,
    updated_at: i64,
    last_message: Option<LastMessageResponse>,
}

#[derive(Debug, Deserialize)]
struct LastMessageResponse {
    msg_id: String,
    sender_uid: i64,
    msg_type: i32,
    content: String,
    created_at: i64,
}

/// 获取会话列表
/// 调用 GET /api/v1/im/contacts
pub async fn list_contacts(api: &ApiClient) -> Result<Vec<XContact>, String> {
    let url = format!("{}/api/v1/im/contacts", crate::api::client::BASE_URL);
    let builder = api.client().get(&url);
    let auth_builder = api.inject_auth(builder).await;

    let contacts_resp = api
        .send_request::<Vec<ContactResponse>>(auth_builder)
        .await
        .map_err(|e| e.to_string())?;

    let contacts: Vec<XContact> = contacts_resp
        .into_iter()
        .map(|c| XContact {
            room_id: c.room_id,
            room_type: c.room_type,
            unread_count: c.unread_count,
            updated_at: c.updated_at,
            last_message: c.last_message.map(|lm| XMessage {
                msg_id: lm.msg_id,
                room_id: c.room_id,
                sender_uid: lm.sender_uid,
                msg_type: lm.msg_type,
                content: lm.content,
                local_status: 0, // 远端消息默认成功状态
                created_at: lm.created_at,
            }),
            show_name: None,
            face_url: None,
        })
        .collect();

    Ok(contacts)
}

/// 从本地 SQLite 数据库读取会话列表（P4 强一致性本地缓存架构）
pub async fn list_contacts_local(db: &DbManager) -> Result<Vec<XContact>, String> {
    let rows: Vec<(
        i64, i64, i64, i64,
        Option<String>, Option<i64>, Option<i64>, Option<String>, Option<i64>, Option<i64>,
        Option<String>, Option<String>
    )> = sqlx::query_as(
        r#"
        SELECT c.room_id, c.room_type, c.unread_count, c.updated_at,
               m.msg_id, m.sender_uid, m.msg_type, m.content, m.local_status, m.created_at,
               COALESCE(u.nick_name, g.name) as show_name,
               COALESCE(u.avatar, g.avatar) as face_url
        FROM contacts c
        LEFT JOIN messages m ON c.last_msg_id = m.msg_id
        LEFT JOIN user_profiles u ON c.room_type = 1 AND c.friend_uid = u.uid
        LEFT JOIN groups g ON c.room_type = 2 AND c.room_id = g.room_id
        WHERE c.is_deleted = 0
        ORDER BY c.updated_at DESC
        "#
    )
    .fetch_all(&db.pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut res = Vec::new();
    for (
        room_id, room_type, unread_count, updated_at,
        msg_id, sender_uid, msg_type, content, local_status, created_at,
        show_name, face_url
    ) in rows {
        let last_message = if let Some(msg_id) = msg_id {
            Some(XMessage {
                msg_id,
                room_id,
                sender_uid: sender_uid.unwrap_or(0),
                msg_type: msg_type.unwrap_or(0) as i32,
                content: content.unwrap_or_default(),
                local_status: local_status.unwrap_or(0) as i32,
                created_at: created_at.unwrap_or(0),
            })
        } else {
            None
        };

        res.push(XContact {
            room_id,
            room_type: room_type as i32,
            unread_count: unread_count as i32,
            updated_at,
            last_message,
            show_name,
            face_url,
        });
    }

    Ok(res)
}

/// 删除会话
/// 调用 DELETE /api/v1/im/contacts/{room_id}
pub async fn delete_contact(api: &ApiClient, room_id: i64) -> Result<(), String> {
    let url = format!(
        "{}/api/v1/im/contacts/{}",
        crate::api::client::BASE_URL,
        room_id
    );
    let builder = api.client().delete(&url);
    let auth_builder = api.inject_auth(builder).await;

    api.send_action(auth_builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// 标记会话已读
/// 调用 POST /api/v1/im/contacts/read
pub async fn mark_read(api: &ApiClient, room_id: i64) -> Result<(), String> {
    let url = format!("{}/api/v1/im/contacts/read", crate::api::client::BASE_URL);

    #[derive(Serialize)]
    struct MarkReadRequest {
        room_id: i64,
    }

    let payload = MarkReadRequest { room_id };
    let builder = api.client().post(&url).json(&payload);
    let auth_builder = api.inject_auth(builder).await;

    api.send_action(auth_builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
