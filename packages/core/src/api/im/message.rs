use super::models::XMessage;
use crate::api::client::ApiClient;
use serde::Deserialize;
use serde_json::json;

/// SDK Internal message delivery entry
/// Fallbacks to saving into sync_queue when network error occurs directly.
pub async fn send_text_message(
    api: &ApiClient,
    db: &crate::db::DbManager,
    room_id: i64,
    content: &str,
    sender_uid: i64,
) -> Result<XMessage, String> {
    let msg_id = uuid::Uuid::new_v4().to_string();
    let current_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // 1. Immediately log to local DB as Pending.
    let xmsg = XMessage {
        msg_id: msg_id.clone(),
        room_id,
        sender_uid,
        msg_type: 0,
        content: content.to_string(),
        local_status: 1, // Pending
        created_at: current_ts,
    };

    sqlx::query(
        r#"
        INSERT INTO messages (msg_id, room_id, sender_uid, msg_type, content, local_status, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&xmsg.msg_id)
    .bind(xmsg.room_id)
    .bind(xmsg.sender_uid)
    .bind(xmsg.msg_type)
    .bind(&xmsg.content)
    .bind(xmsg.local_status)
    .bind(xmsg.created_at)
    .execute(&db.pool)
    .await
    .map_err(|e| e.to_string())?;

    // Prepare remote sync payload conforming to ms-im SendMessageRequest
    let payload = serde_json::json!({
        "room_id": room_id,
        "type": 1, // 1 for text in the backend schema
        "content": content,
        "uuid": msg_id, // Identifies uniqueness
    });
    let url = format!("{}/api/v1/im/messages", crate::api::client::BASE_URL);
    let builder = api.client().post(&url).json(&payload);
    let auth_builder = api.inject_auth(builder).await;

    match api.send_request::<serde_json::Value>(auth_builder).await {
        Ok(_) => {
            // Update local to Success
            sqlx::query("UPDATE messages SET local_status = 0 WHERE msg_id = ?")
                .bind(&msg_id)
                .execute(&db.pool)
                .await
                .map_err(|e| e.to_string())?;

            let mut final_msg = xmsg.clone();
            final_msg.local_status = 0;
            Ok(final_msg)
        }
        Err(e) => {
            tracing::error!("Network failed when sending message: {}", e);
            println!("Network failed when sending message: {}", e);
            // 3. Fallback: Network failed. Push to sync_queue and mark as Failed locally.
            sqlx::query("UPDATE messages SET local_status = 2 WHERE msg_id = ?")
                .bind(&msg_id)
                .execute(&db.pool)
                .await
                .map_err(|e| e.to_string())?;

            let queue_payload = serde_json::to_string(&payload).unwrap();
            sqlx::query("INSERT INTO sync_queue (payload, next_retry_at) VALUES (?, ?)")
                .bind(queue_payload)
                .bind(current_ts + 5) // Retry in 5 seconds by the daemon
                .execute(&db.pool)
                .await
                .map_err(|e| e.to_string())?;

            let mut final_msg = xmsg.clone();
            final_msg.local_status = 2; // Failed but queued
            
            // To safely surface the error to Flutter without causing SSE decode crashes:
            final_msg.content = format!("Network Error: {}", e);
            
            Ok(final_msg)
        }
    }
}

pub async fn save_incoming_message(
    db: &crate::db::DbManager,
    xmsg: &crate::api::im::models::XMessage,
) {
    let _ = sqlx::query(
        r#"
        INSERT INTO messages (msg_id, room_id, sender_uid, msg_type, content, local_status, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(msg_id) DO NOTHING
        "#
    )
    .bind(&xmsg.msg_id)
    .bind(xmsg.room_id)
    .bind(xmsg.sender_uid)
    .bind(xmsg.msg_type)
    .bind(&xmsg.content)
    .bind(xmsg.local_status)
    .bind(xmsg.created_at)
    .execute(&db.pool)
    .await;
}

/// 从本地 SQLite 查询指定房间的历史消息
/// 如果本地消息不足，会尝试从远端拉取
pub async fn get_history_messages(
    api: &ApiClient,
    db: &crate::db::DbManager,
    room_id: i64,
    limit: i64,
) -> Result<Vec<XMessage>, String> {
    // 先从本地查询
    let rows = sqlx::query_as::<_, (String, i64, i64, i32, String, i32, i64)>(
        r#"
        SELECT msg_id, room_id, sender_uid, msg_type, content, local_status, created_at
        FROM messages
        WHERE room_id = ?
        ORDER BY created_at DESC
        LIMIT ?
        "#
    )
    .bind(room_id)
    .bind(limit)
    .fetch_all(&db.pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut messages: Vec<XMessage> = rows
        .into_iter()
        .map(|(msg_id, room_id, sender_uid, msg_type, content, local_status, created_at)| {
            XMessage {
                msg_id,
                room_id,
                sender_uid,
                msg_type,
                content,
                local_status,
                created_at,
            }
        })
        .collect();

    // 如果本地消息数量不足，尝试从远端拉取
    if messages.len() < limit as usize {
        let cursor = messages.last().map(|m| m.created_at);
        let need_count = limit - messages.len() as i64;

        match pull_remote_messages(api, db, room_id, cursor, need_count).await {
            Ok(remote_msgs) => {
                messages.extend(remote_msgs);
            }
            Err(e) => {
                tracing::warn!("Failed to pull remote messages: {}", e);
                // 远端拉取失败不影响返回本地消息
            }
        }
    }

    Ok(messages)
}

/// 后端返回的消息列表项
#[derive(Debug, Deserialize)]
struct RemoteMessageResponse {
    msg_id: String,
    room_id: i64,
    sender_uid: i64,
    #[serde(rename = "type")]
    msg_type: i32,
    content: String,
    created_at: i64,
}

/// 从远端拉取历史消息并存入本地 SQLite
/// 调用 GET /api/v1/im/messages?room_id={room_id}&cursor={cursor}&limit={limit}
pub async fn pull_remote_messages(
    api: &ApiClient,
    db: &crate::db::DbManager,
    room_id: i64,
    cursor: Option<i64>,
    limit: i64,
) -> Result<Vec<XMessage>, String> {
    let mut url = format!(
        "{}/api/v1/im/messages?room_id={}&limit={}",
        crate::api::client::BASE_URL,
        room_id,
        limit
    );

    if let Some(c) = cursor {
        url.push_str(&format!("&cursor={}", c));
    }

    let builder = api.client().get(&url);
    let auth_builder = api.inject_auth(builder).await;

    let remote_msgs = api
        .send_request::<Vec<RemoteMessageResponse>>(auth_builder)
        .await
        .map_err(|e| e.to_string())?;

    let mut messages = Vec::new();

    for rm in remote_msgs {
        let xmsg = XMessage {
            msg_id: rm.msg_id.clone(),
            room_id: rm.room_id,
            sender_uid: rm.sender_uid,
            msg_type: rm.msg_type,
            content: rm.content,
            local_status: 0, // 远端消息默认成功状态
            created_at: rm.created_at,
        };

        // 存入本地 SQLite
        let _ = sqlx::query(
            r#"
            INSERT INTO messages (msg_id, room_id, sender_uid, msg_type, content, local_status, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(msg_id) DO NOTHING
            "#
        )
        .bind(&xmsg.msg_id)
        .bind(xmsg.room_id)
        .bind(xmsg.sender_uid)
        .bind(xmsg.msg_type)
        .bind(&xmsg.content)
        .bind(xmsg.local_status)
        .bind(xmsg.created_at)
        .execute(&db.pool)
        .await;

        messages.push(xmsg);
    }

    Ok(messages)
}

