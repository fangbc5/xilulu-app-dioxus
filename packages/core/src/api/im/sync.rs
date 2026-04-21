use serde::Deserialize;
use tracing::info;

use crate::api::client::ApiClient;
use crate::db::DbManager;

// ─────────────────────────────────────────────────────────────────────────────
// 服务端响应结构体（完全对齐 ms-im 实体字段名）
// ─────────────────────────────────────────────────────────────────────────────

/// 对应服务端 user_friend 实体
#[derive(Debug, Deserialize, Clone)]
pub struct ServerUserFriend {
    pub id: Option<i64>,
    pub uid: i64,
    pub friend_uid: i64,
    pub remark: Option<String>,
    pub status: i16,
    pub created_at: String,
    pub updated_at: String,
}

/// 对应服务端 contact 实体
#[derive(Debug, Deserialize, Clone)]
pub struct ServerContact {
    pub room_id: i64,
    pub read_time: Option<String>,
    pub active_time: Option<String>,
    pub last_msg_id: Option<i64>,
    pub read_msg_id: Option<i64>,
    pub clear_msg_id: i64,
    pub is_mute: i16,
    pub is_top: i16,
    pub is_deleted: i16,
    pub unread_count: i64,
    pub created_at: Option<String>,
    pub updated_at: String,
}

/// 对应服务端 room 实体
#[derive(Debug, Deserialize, Clone)]
pub struct ServerRoom {
    pub id: i64,
    #[serde(rename = "type")]
    pub r#type: i16,
    pub active_time: String,
    pub updated_at: String,
}

/// 对应服务端 room_friend 实体
#[derive(Debug, Deserialize, Clone)]
pub struct ServerRoomFriend {
    pub room_id: i64,
    pub uid1: i64,
    pub uid2: i64,
    pub created_at: String,
}

/// 对应服务端 room_group 实体
#[derive(Debug, Deserialize, Clone)]
pub struct ServerRoomGroup {
    pub id: i64,
    pub room_id: i64,
    pub name: String,
    pub avatar: Option<String>,
    pub notice: Option<String>,
    pub is_deleted: i16,
    pub created_by: i64,
    pub created_at: Option<String>,
    pub updated_at: String,
}

/// 对应服务端 group_member 实体
#[derive(Debug, Deserialize, Clone)]
pub struct ServerGroupMember {
    pub group_id: i64,
    pub uid: i64,
    pub role: i16,
    pub created_at: Option<String>,
    pub updated_at: String,
}

/// 对应服务端 UserBrief（来自 ms-identity BFF）
#[derive(Debug, Deserialize, Clone)]
pub struct ServerUserProfile {
    pub id: i64, // 对应本地 user_profile.uid
    pub nick_name: String,
    pub avatar: String,
}

/// Sync 接口完整响应体（字段名与服务端 SyncResponse 100% 对齐）
#[derive(Debug, Deserialize, Clone)]
pub struct SyncResponse {
    pub friends: Vec<ServerUserFriend>,
    pub contacts: Vec<ServerContact>,
    pub rooms: Vec<ServerRoom>,
    pub room_friends: Vec<ServerRoomFriend>,
    pub room_groups: Vec<ServerRoomGroup>,
    pub group_members: Vec<ServerGroupMember>,
    pub user_profiles: Vec<ServerUserProfile>,
}

/// 批量最新消息请求体
#[derive(Debug, serde::Serialize)]
pub struct BatchLatestMessageRequest {
    pub room_ids: Vec<i64>,
    pub limit: i64,
}

/// 对应服务端 message 实体
#[derive(Debug, Deserialize, Clone)]
pub struct ServerMessage {
    pub id: i64,
    pub room_id: i64,
    pub from_uid: i64,
    pub content: Option<String>,
    #[serde(rename = "type")]
    pub r#type: i16,
    pub reply_msg_id: Option<i64>,
    pub status: i16,
    pub extra: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// 增量同步核心逻辑
// ─────────────────────────────────────────────────────────────────────────────

/// 解析 RFC3339 时间字符串为毫秒时间戳
fn parse_ts(s: &str) -> i64 {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|d| d.timestamp_millis())
        .unwrap_or(0)
}

fn parse_ts_opt(s: Option<&str>) -> Option<i64> {
    s.and_then(|v| chrono::DateTime::parse_from_rfc3339(v).ok())
        .map(|d| d.timestamp_millis())
}

/// 执行增量同步
pub async fn execute_sync(api: &ApiClient, db: &DbManager, my_uid: i64) -> Result<(), String> {
    // 1. 读取最后一次的 Sync Timestamp
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT timestamp_ms FROM sync_cursor WHERE module = 'global'")
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

    println!(
        ">>> [SYNC] friends={}, contacts={}, room_groups={}, user_profiles={}",
        resp.friends.len(),
        resp.contacts.len(),
        resp.room_groups.len(),
        resp.user_profiles.len()
    );

    // 3. 若无任何变动则跳过落库
    let has_data = !resp.friends.is_empty()
        || !resp.contacts.is_empty()
        || !resp.room_groups.is_empty()
        || !resp.user_profiles.is_empty();

    if !has_data {
        println!(">>> [SYNC] 无增量数据，跳过落库");
        return Ok(());
    }

    let mut tx = db.pool.begin().await.map_err(|e| e.to_string())?;
    let now_ms = chrono::Utc::now().timestamp_millis();

    // ── 好友关系 → user_friend ──
    // 服务端已确保只返回 uid=me 的记录，直接按服务端字段写入，uid/friend_uid 都保留
    for friend in &resp.friends {
        let created_ms = parse_ts(&friend.created_at);
        let updated_ms = parse_ts(&friend.updated_at);
        sqlx::query(
            "INSERT INTO user_friend (uid, friend_uid, remark, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(uid, friend_uid) DO UPDATE SET
                remark=excluded.remark, status=excluded.status, updated_at=excluded.updated_at",
        )
        .bind(friend.uid)
        .bind(friend.friend_uid)
        .bind(&friend.remark)
        .bind(friend.status)
        .bind(created_ms)
        .bind(updated_ms)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }
    println!(">>> [SYNC] user_friend 写入 {} 条", resp.friends.len());

    // ── 构建 room_id → room_type 映射表（来自 rooms）──
    let mut rt_map = std::collections::HashMap::new();
    for r in &resp.rooms {
        rt_map.insert(r.id, r.r#type);
    }

    // ── 构建 room_id → friend_uid 映射表（来自 room_friends）──
    let mut rf_map = std::collections::HashMap::new();
    for rf in &resp.room_friends {
        let other_uid = if rf.uid1 == my_uid { rf.uid2 } else { rf.uid1 };
        rf_map.insert(rf.room_id, other_uid);
    }

    // ── 会话 → contact ──
    for c in &resp.contacts {
        let room_type = rt_map.get(&c.room_id).copied();
        let friend_uid = rf_map.get(&c.room_id).copied();
        let read_time_ms = parse_ts_opt(c.read_time.as_deref());
        let active_time_ms = parse_ts_opt(c.active_time.as_deref());
        let created_ms = parse_ts_opt(c.created_at.as_deref());
        let updated_ms = parse_ts(&c.updated_at);

        sqlx::query(
            "INSERT INTO contact
                (room_id, room_type, read_time, active_time, last_msg_id, read_msg_id,
                 clear_msg_id, is_mute, is_top, is_deleted, unread_count,
                 friend_uid, created_at, updated_at)
             VALUES
                (?, COALESCE(?, (SELECT room_type FROM contact WHERE room_id = ?), 1),
                 ?, ?, ?, ?, ?, ?, ?, ?, ?,
                 COALESCE(?, (SELECT friend_uid FROM contact WHERE room_id = ?)),
                 ?, ?)
             ON CONFLICT(room_id) DO UPDATE SET
                room_type    = COALESCE(excluded.room_type, contact.room_type),
                read_time    = COALESCE(excluded.read_time, contact.read_time),
                active_time  = COALESCE(excluded.active_time, contact.active_time),
                last_msg_id  = COALESCE(excluded.last_msg_id, contact.last_msg_id),
                read_msg_id  = excluded.read_msg_id,
                clear_msg_id = excluded.clear_msg_id,
                is_mute      = excluded.is_mute,
                is_top       = excluded.is_top,
                is_deleted   = excluded.is_deleted,
                unread_count = excluded.unread_count,
                friend_uid   = COALESCE(excluded.friend_uid, contact.friend_uid),
                updated_at   = excluded.updated_at",
        )
        .bind(c.room_id)
        .bind(room_type)
        .bind(c.room_id)
        .bind(read_time_ms)
        .bind(active_time_ms)
        .bind(c.last_msg_id)
        .bind(c.read_msg_id)
        .bind(c.clear_msg_id)
        .bind(c.is_mute)
        .bind(c.is_top)
        .bind(c.is_deleted)
        .bind(c.unread_count)
        .bind(friend_uid)
        .bind(c.room_id)
        .bind(created_ms)
        .bind(updated_ms)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }
    println!(">>> [SYNC] contact 写入 {} 条", resp.contacts.len());

    // ── 群聊 → room_group ──
    for rg in &resp.room_groups {
        let created_ms = parse_ts_opt(rg.created_at.as_deref()).unwrap_or(0);
        let updated_ms = parse_ts(&rg.updated_at);
        sqlx::query(
            "INSERT INTO room_group (id, room_id, name, avatar, notice, is_deleted, created_by, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                name=excluded.name, avatar=excluded.avatar, notice=excluded.notice,
                is_deleted=excluded.is_deleted, updated_at=excluded.updated_at"
        )
        .bind(rg.id)
        .bind(rg.room_id)
        .bind(&rg.name)
        .bind(&rg.avatar)
        .bind(&rg.notice)
        .bind(rg.is_deleted)
        .bind(rg.created_by)
        .bind(created_ms)
        .bind(updated_ms)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }
    println!(">>> [SYNC] room_group 写入 {} 条", resp.room_groups.len());

    // ── 群成员全量替换 → group_member ──
    let mut modified_groups: std::collections::HashSet<i64> = std::collections::HashSet::new();
    for gm in &resp.group_members {
        modified_groups.insert(gm.group_id);
    }
    for mg_id in &modified_groups {
        sqlx::query("DELETE FROM group_member WHERE group_id = ?")
            .bind(mg_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    for gm in &resp.group_members {
        let created_ms = parse_ts_opt(gm.created_at.as_deref()).unwrap_or(0);
        let updated_ms = parse_ts(&gm.updated_at);
        sqlx::query(
            "INSERT INTO group_member (group_id, uid, role, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(gm.group_id)
        .bind(gm.uid)
        .bind(gm.role)
        .bind(created_ms)
        .bind(updated_ms)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    // ── 用户资料 → user_profile ──
    // 服务端 UserBrief 字段：id（即 user.id），nick_name，avatar
    for up in &resp.user_profiles {
        sqlx::query(
            "INSERT INTO user_profile (uid, nick_name, avatar, updated_at)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(uid) DO UPDATE SET
                nick_name=excluded.nick_name, avatar=excluded.avatar, updated_at=excluded.updated_at"
        )
        .bind(up.id)        // UserBrief.id → user_profile.uid
        .bind(&up.nick_name)
        .bind(&up.avatar)
        .bind(now_ms)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }
    println!(
        ">>> [SYNC] user_profile 写入 {} 条",
        resp.user_profiles.len()
    );

    // ── 更新 Sync Cursor ──
    sqlx::query(
        "INSERT INTO sync_cursor (module, timestamp_ms) VALUES ('global', ?)
         ON CONFLICT(module) DO UPDATE SET timestamp_ms=excluded.timestamp_ms",
    )
    .bind(now_ms)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    info!("同步任务完成");
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// 历史消息大盘批量同步
// ─────────────────────────────────────────────────────────────────────────────

/// 执行并发最近消息切块同步
pub async fn execute_recent_messages_sync<F>(
    api: &ApiClient,
    db: &DbManager,
    sync_history: bool,
    on_latest_msg: F,
) -> Result<String, String>
where
    F: Fn(crate::api::im::models::XMessage) + Send + Sync + 'static,
{
    if !sync_history {
        info!("用户选择不同步历史消息，跳过最近消息大盘拉取");
        return Ok("Skip history option selected".to_string());
    }

    // 1. 捞出本地所有未删除的 room_ids
    let rooms: Vec<(i64,)> = sqlx::query_as("SELECT room_id FROM contact WHERE is_deleted = 0")
        .fetch_all(&db.pool)
        .await
        .map_err(|e| e.to_string())?;

    let all_room_ids: Vec<i64> = rooms.into_iter().map(|r| r.0).collect();
    if all_room_ids.is_empty() {
        return Ok("Rooms fetch from local contact table is EMPTY! all_room_ids = 0".to_string());
    }

    info!(
        "开始执行游标批量消息同步，涉及房间数: {}",
        all_room_ids.len()
    );

    let chunks: Vec<&[i64]> = all_room_ids.chunks(30).collect();
    let url = format!(
        "{}/api/v1/im/messages/batch_latest",
        crate::api::client::BASE_URL
    );

    let mut total_inserted = 0;

    for chunk in chunks {
        let req_body = BatchLatestMessageRequest {
            room_ids: chunk.to_vec(),
            limit: 50,
        };

        let req = api.client().post(&url).json(&req_body);
        let req = api.inject_auth(req).await;

        let raw_res = req.send().await.map_err(|e| e.to_string())?;
        let raw_str = raw_res.text().await.unwrap_or_default();
        let api_resp: crate::api::client::ApiResponse<
            std::collections::HashMap<String, Vec<ServerMessage>>,
        > = match serde_json::from_str(&raw_str) {
            Ok(r) => r,
            Err(e) => {
                return Err(format!(
                    "BATCH_LATEST RAW ERR. JSON: {} | Err: {}",
                    raw_str, e
                ));
            }
        };

        if !api_resp.success {
            return Err(format!(
                "BATCH LATEST API FAILED: {}",
                api_resp.msg.unwrap_or_default()
            ));
        }

        let resp = api_resp.data.unwrap_or_default();
        if resp.is_empty() {
            continue;
        }

        let mut tx = db.pool.begin().await.map_err(|e| e.to_string())?;
        let now_ms = chrono::Utc::now().timestamp_millis();
        let _ = now_ms;

        for (_room_id, msgs) in &resp {
            for m in msgs {
                let created_ms = parse_ts(&m.created_at);
                let updated_ms = m.updated_at.as_deref().map(parse_ts).unwrap_or(created_ms);

                sqlx::query(
                    "INSERT INTO message
                        (msg_id, room_id, from_uid, content, type, reply_msg_id, status, extra, local_status, created_at, updated_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?)
                     ON CONFLICT(msg_id) DO UPDATE SET
                        status=excluded.status,
                        content=excluded.content,
                        updated_at=excluded.updated_at"
                )
                .bind(&m.id.to_string())
                .bind(m.room_id)
                .bind(m.from_uid)
                .bind(&m.content)
                .bind(m.r#type)
                .bind(m.reply_msg_id)
                .bind(m.status)
                .bind(m.extra.as_ref().map(|v| v.to_string()))
                .bind(created_ms)
                .bind(updated_ms)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;

                // 同步更新 contact.last_msg_id（取更大值）
                sqlx::query(
                    "UPDATE contact SET last_msg_id = ? WHERE room_id = ?
                     AND (last_msg_id IS NULL OR last_msg_id < ?)",
                )
                .bind(m.id)
                .bind(m.room_id)
                .bind(m.id)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
            }

            total_inserted += msgs.len();
            // 将最新一条消息作为事件通知 Flutter 刷新 UI
            if let Some(latest) = msgs.first() {
                let created_ms = parse_ts(&latest.created_at);
                let xmsg = crate::api::im::models::XMessage {
                    msg_id: latest.id.to_string(),
                    room_id: latest.room_id,
                    from_uid: latest.from_uid,
                    content: latest.content.clone(),
                    msg_type: latest.r#type,
                    reply_msg_id: latest.reply_msg_id,
                    status: latest.status,
                    extra: latest.extra.as_ref().map(|v| v.to_string()),
                    local_status: 0,
                    created_at: created_ms,
                };
                on_latest_msg(xmsg);
            }
        }

        tx.commit().await.map_err(|e| e.to_string())?;
        info!("已同步 {} 个房间的消息数据", chunk.len());
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    info!("所有消息大盘同步完成");
    Ok(format!(
        "Successfully finished syncing recent messages. total_room_chunks: {}, total_inserted: {}",
        all_room_ids.len(),
        total_inserted
    ))
}
