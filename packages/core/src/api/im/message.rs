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
    my_uid: i64,
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

    // 更新对应的会话记录，包含最后一条消息 ID，以及累加未读数
    // 如果是自己在多设备端发的消息，未读数不应增加
    let increment = if xmsg.from_uid == my_uid { 0 } else { 1 };
    let msg_id_i64: i64 = xmsg.msg_id.parse().unwrap_or(0);

    let _ = sqlx::query(
        r#"
        UPDATE contact 
        SET last_msg_id = ?, 
            unread_count = unread_count + ?, 
            updated_at = ? 
        WHERE room_id = ?
        "#
    )
    .bind(msg_id_i64)
    .bind(increment)
    .bind(now_ms)
    .bind(xmsg.room_id)
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
        // 游标应该是最后一条正常远端消息的 msg_id，而不是 created_at
        let mut cursor = None;
        for m in messages.iter().rev() {
            if let Ok(id) = m.msg_id.parse::<i64>() {
                if id > 0 {
                    cursor = Some(id);
                    break;
                }
            }
        }
        
        let need_count = limit - messages.len() as i64;

        match pull_remote_messages(api, db, room_id, cursor, need_count).await {
            Ok(remote_msgs) => {
                // 去重保护，防止本地已存在相同的消息重新出现两次
                for rm in remote_msgs {
                    if !messages.iter().any(|m| m.msg_id == rm.msg_id) {
                        messages.push(rm);
                    }
                }
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

/// 消费离线重发队列（WS 重连后调用）
///
/// 从 `sync_queue` 表中取出到期的待重发消息，逐条发送到服务端。
/// 成功后删除队列项并更新本地消息状态；失败后递增 `retry_count`，
/// 超过 3 次重试上限的消息将被丢弃（本地消息保持 `local_status=2` 失败态）。
pub async fn flush_sync_queue(
    api: &ApiClient,
    db: &crate::db::DbManager,
) -> Result<(), String> {
    let now_ms = chrono::Utc::now().timestamp_millis();

    let rows: Vec<(i64, String, i64)> = sqlx::query_as(
        "SELECT id, payload, retry_count FROM sync_queue WHERE next_retry_at <= ? ORDER BY id LIMIT 50",
    )
    .bind(now_ms)
    .fetch_all(&db.pool)
    .await
    .map_err(|e| e.to_string())?;

    if rows.is_empty() {
        return Ok(());
    }

    tracing::info!("开始消费离线重发队列，待处理 {} 条", rows.len());

    for (id, payload, retry_count) in rows {
        // 超过重试上限，丢弃
        if retry_count >= 3 {
            tracing::warn!("重发队列消息超过重试上限，丢弃: queue_id={}", id);
            sqlx::query("DELETE FROM sync_queue WHERE id = ?")
                .bind(id)
                .execute(&db.pool)
                .await
                .ok();
            continue;
        }

        let msg_payload: serde_json::Value =
            serde_json::from_str(&payload).unwrap_or_default();

        let url = format!("{}/api/v1/im/messages", crate::api::client::BASE_URL);
        let builder = api.client().post(&url).json(&msg_payload);
        let auth_builder = api.inject_auth(builder).await;

        match api
            .send_request::<serde_json::Value>(auth_builder)
            .await
        {
            Ok(resp) => {
                // 发送成功 → 删除队列项，更新本地消息状态
                sqlx::query("DELETE FROM sync_queue WHERE id = ?")
                    .bind(id)
                    .execute(&db.pool)
                    .await
                    .ok();

                // 将本地消息从 pending/failed 更新为 success
                if let Some(uuid) = msg_payload.get("uuid").and_then(|v| v.as_str()) {
                    if let Some(remote_id) = resp.get("id").and_then(|v| v.as_i64()) {
                        sqlx::query(
                            "UPDATE message SET msg_id = ?, local_status = 0 WHERE msg_id = ?",
                        )
                        .bind(remote_id.to_string())
                        .bind(uuid)
                        .execute(&db.pool)
                        .await
                        .ok();
                    }
                }
                tracing::info!("重发队列消息成功: queue_id={}", id);
            }
            Err(_) => {
                // 仍然失败 → 递增 retry_count，延迟下次重试
                let next_retry = now_ms + (5000 * (retry_count + 1));
                sqlx::query(
                    "UPDATE sync_queue SET retry_count = retry_count + 1, next_retry_at = ? WHERE id = ?",
                )
                .bind(next_retry)
                .bind(id)
                .execute(&db.pool)
                .await
                .ok();
                tracing::warn!(
                    "重发队列消息仍失败: queue_id={}, retry={}",
                    id,
                    retry_count + 1
                );
            }
        }
    }

    Ok(())
}
