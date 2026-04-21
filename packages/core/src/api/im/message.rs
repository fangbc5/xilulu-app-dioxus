use super::models::XMessage;
use crate::api::client::ApiClient;
use serde::Deserialize;

/// 发送文本消息（先本地落库为 Pending，再发远端，失败后推入 sync_queue）
pub async fn send_text_message(
    api: &ApiClient,
    db: &crate::db::DbManager,
    room_id: i64,
    content: &str,
    from_uid: i64,
) -> Result<XMessage, String> {
    let msg_id = uuid::Uuid::new_v4().to_string();
    let current_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    // 1. 立刻将消息以 Pending 状态写入本地 SQLite
    let xmsg = XMessage {
        msg_id: msg_id.clone(),
        room_id,
        from_uid,
        content: Some(content.to_string()),
        msg_type: 1, // 1 = 文本
        reply_msg_id: None,
        status: 0, // 0 = 正常
        extra: None,
        local_status: 1, // 发送中
        created_at: current_ts,
    };

    sqlx::query(
        r#"
        INSERT INTO message (msg_id, room_id, from_uid, content, type, reply_msg_id, status, local_status, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&xmsg.msg_id)
    .bind(xmsg.room_id)
    .bind(xmsg.from_uid)
    .bind(&xmsg.content)
    .bind(xmsg.msg_type)
    .bind(xmsg.reply_msg_id)
    .bind(xmsg.status)
    .bind(xmsg.local_status)
    .bind(xmsg.created_at)
    .bind(xmsg.created_at) // updated_at 初始与 created_at 相同
    .execute(&db.pool)
    .await
    .map_err(|e| e.to_string())?;

    // 2. 向服务端发起 REST 请求
    let payload = serde_json::json!({
        "room_id": room_id,
        "type": 1,       // 服务端 type：1=文本
        "content": content,
        "uuid": msg_id,  // 幂等标识
    });
    let url = format!("{}/api/v1/im/messages", crate::api::client::BASE_URL);
    let builder = api.client().post(&url).json(&payload);
    let auth_builder = api.inject_auth(builder).await;

    match api.send_request::<serde_json::Value>(auth_builder).await {
        Ok(resp) => {
            if let Some(remote_id) = resp.get("id").and_then(|v| v.as_i64()) {
                let remote_id_str = remote_id.to_string();
                // 3a. 发送成功 → 替换 msg_id 为真实服务端 ID，设为 local_status=0
                sqlx::query("UPDATE message SET msg_id = ?, local_status = 0, updated_at = ? WHERE msg_id = ?")
                    .bind(&remote_id_str)
                    .bind(current_ts)
                    .bind(&msg_id)
                    .execute(&db.pool)
                    .await
                    .map_err(|e| e.to_string())?;

                let mut final_msg = xmsg.clone();
                final_msg.msg_id = remote_id_str;
                final_msg.local_status = 0;
                Ok(final_msg)
            } else {
                // 降级：未返回 id，仅更新状态
                sqlx::query("UPDATE message SET local_status = 0, updated_at = ? WHERE msg_id = ?")
                    .bind(current_ts)
                    .bind(&msg_id)
                    .execute(&db.pool)
                    .await
                    .map_err(|e| e.to_string())?;

                let mut final_msg = xmsg.clone();
                final_msg.local_status = 0;
                Ok(final_msg)
            }
        }
        Err(e) => {
            tracing::error!("发送消息网络失败: {}", e);
            println!("发送消息网络失败: {}", e);

            // 3b. 发送失败 → 更新为 local_status=2（失败），推入重发队列
            sqlx::query("UPDATE message SET local_status = 2, updated_at = ? WHERE msg_id = ?")
                .bind(current_ts)
                .bind(&msg_id)
                .execute(&db.pool)
                .await
                .map_err(|e| e.to_string())?;

            let queue_payload = serde_json::to_string(&payload).unwrap();
            let _ = sqlx::query("INSERT INTO sync_queue (payload, next_retry_at) VALUES (?, ?)")
                .bind(queue_payload)
                .bind(current_ts + 5000) // 5 秒后重试
                .execute(&db.pool)
                .await;

            let mut final_msg = xmsg.clone();
            final_msg.local_status = 2;
            final_msg.content = Some(format!("Network Error: {}", e));
            Ok(final_msg)
        }
    }
}

/// 保存从 WebSocket 收到的消息到本地 SQLite
pub async fn save_incoming_message(
    db: &crate::db::DbManager,
    xmsg: &crate::api::im::models::XMessage,
) {
    let now_ms = chrono::Utc::now().timestamp_millis();
    let _ = sqlx::query(
        r#"
        INSERT INTO message (msg_id, room_id, from_uid, content, type, reply_msg_id, status, local_status, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, ?)
        ON CONFLICT(msg_id) DO NOTHING
        "#
    )
    .bind(&xmsg.msg_id)
    .bind(xmsg.room_id)
    .bind(xmsg.from_uid)
    .bind(&xmsg.content)
    .bind(xmsg.msg_type)
    .bind(xmsg.reply_msg_id)
    .bind(xmsg.status)
    .bind(xmsg.created_at)
    .bind(now_ms)
    .execute(&db.pool)
    .await;
}

/// 从本地 SQLite 查询指定房间的历史消息，不足时从远端补充
pub async fn get_history_messages(
    api: &ApiClient,
    db: &crate::db::DbManager,
    room_id: i64,
    limit: i64,
) -> Result<Vec<XMessage>, String> {
    // 先从本地查
    let rows = sqlx::query_as::<_, (String, i64, i64, Option<String>, i64, Option<i64>, i64, Option<String>, i64, i64)>(
        r#"
        SELECT msg_id, room_id, from_uid, content, type, reply_msg_id, status, extra, local_status, created_at
        FROM message
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
        .map(
            |(
                msg_id,
                room_id,
                from_uid,
                content,
                msg_type,
                reply_msg_id,
                status,
                extra,
                local_status,
                created_at,
            )| {
                XMessage {
                    msg_id,
                    room_id,
                    from_uid,
                    content,
                    msg_type: msg_type as i16,
                    reply_msg_id,
                    status: status as i16,
                    extra,
                    local_status: local_status as i32,
                    created_at,
                }
            },
        )
        .collect();

    // 本地不足时从远端补充
    if messages.len() < limit as usize {
        let cursor = messages.last().map(|m| m.created_at);
        let need_count = limit - messages.len() as i64;

        match pull_remote_messages(api, db, room_id, cursor, need_count).await {
            Ok(remote_msgs) => {
                messages.extend(remote_msgs);
            }
            Err(e) => {
                tracing::warn!("拉取远端历史消息失败: {}", e);
            }
        }
    }

    Ok(messages)
}

/// 远端消息响应结构（与服务端 message 实体字段对齐）
#[derive(Debug, Deserialize)]
struct RemoteMessageResponse {
    id: i64,
    room_id: i64,
    from_uid: i64,
    content: Option<String>,
    #[serde(rename = "type")]
    msg_type: i16,
    reply_msg_id: Option<i64>,
    status: i16,
    extra: Option<serde_json::Value>,
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

    let resp_wrapper = api
        .send_request::<serde_json::Value>(auth_builder)
        .await
        .map_err(|e| e.to_string())?;

    let list_val = resp_wrapper
        .get("list")
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));

    let remote_msgs = serde_json::from_value::<Vec<RemoteMessageResponse>>(list_val)
        .map_err(|e| e.to_string())?;

    let mut messages = Vec::new();
    let now_ms = chrono::Utc::now().timestamp_millis();

    for rm in remote_msgs {
        let xmsg = XMessage {
            msg_id: rm.id.to_string(),
            room_id: rm.room_id,
            from_uid: rm.from_uid,
            content: rm.content,
            msg_type: rm.msg_type,
            reply_msg_id: rm.reply_msg_id,
            status: rm.status,
            extra: rm.extra.map(|v| v.to_string()),
            local_status: 0, // 远端消息 = 已完成
            created_at: rm.created_at,
        };

        // 写入本地
        let _ = sqlx::query(
            r#"
            INSERT INTO message (msg_id, room_id, from_uid, content, type, reply_msg_id, status, local_status, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, ?)
            ON CONFLICT(msg_id) DO NOTHING
            "#
        )
        .bind(&xmsg.msg_id)
        .bind(xmsg.room_id)
        .bind(xmsg.from_uid)
        .bind(&xmsg.content)
        .bind(xmsg.msg_type)
        .bind(xmsg.reply_msg_id)
        .bind(xmsg.status)
        .bind(xmsg.created_at)
        .bind(now_ms)
        .execute(&db.pool)
        .await;

        messages.push(xmsg);
    }

    Ok(messages)
}
