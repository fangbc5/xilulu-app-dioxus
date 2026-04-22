use crate::frb_generated::StreamSink;
use std::sync::Arc;

use crate::api::GLOBAL_API_CLIENT;
use xilulu_core::api::im::message::send_text_message;
pub use xilulu_core::api::im::models::{XContact, XEvent, XMessage};
use xilulu_core::port::StorageProvider;
use xilulu_core::ws::client::WsClient;

lazy_static::lazy_static! {
    /// SDK Singleton Global Database
    pub static ref GLOBAL_DB: tokio::sync::RwLock<Option<xilulu_core::db::DbManager>> = tokio::sync::RwLock::new(None);

    /// Flutter Event Sink — hot restart 后 core_subscribe_im_events 会替换它
    pub static ref FLUTTER_STREAM: std::sync::Mutex<Option<StreamSink<String>>> = std::sync::Mutex::new(None);

    /// Global WS Client — 用 RwLock 替代 OnceCell，允许 hot restart 后重新创建
    pub static ref GLOBAL_WS_CLIENT: tokio::sync::RwLock<Option<Arc<WsClient>>> = tokio::sync::RwLock::new(None);
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // 默认通过 flutter_rust_bridge 控制台输出和 panic 拦截
    flutter_rust_bridge::setup_default_user_utils();
    // 初始化 Tracing 并定向到标准输出流，Flutter Run 会在控制台捕获这些输出
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();
}

pub async fn core_init_sdk(db_path: String) -> Result<(), String> {
    let db_url = format!("sqlite://{}", db_path);
    let manager = xilulu_core::db::DbManager::new(&db_url)
        .await
        .map_err(|e| e.to_string())?;

    let mut db_guard = GLOBAL_DB.write().await;
    *db_guard = Some(manager);
    Ok(())
}

/// 将 Flutter 侧（SharedPreferences）持久化的 Token 注入到 Rust GLOBAL_STORAGE。
/// 应在 main.dart 的 coreInitSdk 之后立即调用，以恢复上次登录状态。
pub async fn core_update_tokens(
    access_token: String,
    refresh_token: String,
    access_expires_at: Option<i64>,
    refresh_expires_at: Option<i64>,
) {
    use xilulu_core::port::StorageProvider;
    let _ = crate::api::GLOBAL_STORAGE
        .set("access_token", &access_token)
        .await;
    let _ = crate::api::GLOBAL_STORAGE
        .set("refresh_token", &refresh_token)
        .await;

    // 同步过期时间
    if let Some(access_exp) = access_expires_at {
        let _ = crate::api::GLOBAL_STORAGE
            .set("access_expires_at", &access_exp.to_string())
            .await;
    }
    if let Some(refresh_exp) = refresh_expires_at {
        let _ = crate::api::GLOBAL_STORAGE
            .set("refresh_expires_at", &refresh_exp.to_string())
            .await;
    }
}

/// 启动 WebSocket 连接和 IM 后台任务。
///
/// Token 已通过 core_update_tokens 注入到 GLOBAL_STORAGE，此处不再需要传入。
/// - `url`: WebSocket 服务端地址，如 "ws://127.0.0.1:8080/ws"
/// - `client_id`: 设备唯一标识（UUID v4）
/// - `sync_chat_history`: 是否同步近期聊天记录
pub async fn core_start_ws(
    url: String,
    client_id: String,
    sync_chat_history: bool,
) -> Result<(), String> {
    // 1. 断开并销毁旧的 WS 连接（hot restart 场景）
    {
        let mut guard = GLOBAL_WS_CLIENT.write().await;
        if let Some(old_ws) = guard.take() {
            old_ws.disconnect().await;
        }
    }

    // 2. 从 GLOBAL_STORAGE 读取最新 access_token
    let access_token = crate::api::GLOBAL_STORAGE
        .get("access_token")
        .await
        .ok()
        .flatten()
        .unwrap_or_default();

    if access_token.is_empty() {
        eprintln!("[WS] access_token 为空，跳过 WS 连接。请先调用 core_update_tokens。");
        return Err("access_token is empty".to_string());
    }

    // 3. 建立 WS 连接
    let ws = Arc::new(WsClient::new(
        url,
        access_token,
        client_id.clone(),
        Some(crate::api::GLOBAL_STORAGE.clone()),
    ));

    {
        let mut guard = GLOBAL_WS_CLIENT.write().await;
        *guard = Some(ws.clone());
    }

    // my_uid 用于增量同步，从 GLOBAL_STORAGE 读取（登录time写入）
    let my_uid = crate::api::GLOBAL_STORAGE
        .get("user_id")
        .await
        .ok()
        .flatten()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    // 4. 启动 TOKEN_REFRESHED 转发任务
    //    监听 GLOBAL_STORAGE 的 token 更新广播，转推给 Flutter，
    //    Flutter 收到后将新 token 写回 SharedPreferences 持久化
    let mut token_rx = crate::api::GLOBAL_STORAGE.subscribe_token_updates();
    tokio::spawn(async move {
        while token_rx.recv().await.is_ok() {
            let access = crate::api::GLOBAL_STORAGE
                .get("access_token")
                .await
                .ok()
                .flatten()
                .unwrap_or_default();
            let refresh = crate::api::GLOBAL_STORAGE
                .get("refresh_token")
                .await
                .ok()
                .flatten()
                .unwrap_or_default();
            let expires_at_str = crate::api::GLOBAL_STORAGE
                .get("access_expires_at")
                .await
                .ok()
                .flatten()
                .unwrap_or_default();

            if !access.is_empty() {
                // 计算剩余有效期（秒）
                let now = chrono::Utc::now().timestamp();
                let expires_at = expires_at_str.parse::<i64>().unwrap_or(now + 900);
                let expires_in = (expires_at - now).max(0);

                if let Ok(guard) = FLUTTER_STREAM.lock() {
                    if let Some(sink) = guard.as_ref() {
                        let json = format!(
                            "{{\"TOKEN_REFRESHED\": {{\"access\": \"{}\", \"refresh\": \"{}\", \"expires_in\": {}}}}}",
                            access, refresh, expires_in
                        );
                        let _ = sink.add(json);
                    }
                }
            }
        }
    });

    // 5. 启动增量同步任务
    tokio::spawn(async move {
        if let Some(db) = &*GLOBAL_DB.read().await {
            match xilulu_core::api::im::sync::execute_sync(
                &crate::api::GLOBAL_API_CLIENT,
                db,
                my_uid,
            )
            .await
            {
                Ok(_) => {
                    if let Ok(guard) = FLUTTER_STREAM.lock() {
                        if let Some(sink) = guard.as_ref() {
                            let _ = sink.add(
                                serde_json::to_string(&XEvent::OnConversationListUpdated).unwrap(),
                            );
                        }
                    }

                    // 拉取近期消息
                    match xilulu_core::api::im::sync::execute_recent_messages_sync(
                        &crate::api::GLOBAL_API_CLIENT,
                        db,
                        sync_chat_history,
                        |latest_msg| {
                            if let Ok(guard) = FLUTTER_STREAM.lock() {
                                if let Some(sink) = guard.as_ref() {
                                    if let Ok(json) = serde_json::to_string(
                                        &XEvent::OnChatLatestMessageUpdated(latest_msg),
                                    ) {
                                        let _ = sink.add(json);
                                    }
                                }
                            }
                        },
                    )
                    .await
                    {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("最近历史消息同步失败: {}", e);
                        }
                    }
                }
                Err(e) => {
                    let err_str = e.replace('"', "'");
                    let is_auth_failure = err_str.contains("UNAUTHORIZED");
                    if let Ok(guard) = FLUTTER_STREAM.lock() {
                        if let Some(sink) = guard.as_ref() {
                            let _ = sink
                                .add(format!("{{\"DEBUG_SYNC\": \"Sync failed: {}\"}}", err_str));
                            if is_auth_failure {
                                // Token 不可恢复，通知 Flutter 跳转登录页
                                let _ = sink.add("{\"AUTH_EXPIRED\": true}".to_string());
                            }
                        }
                    }
                }
            }
        } else {
            if let Ok(guard) = FLUTTER_STREAM.lock() {
                if let Some(sink) = guard.as_ref() {
                    let _ = sink.add("{\"DEBUG_SYNC\": \"GLOBAL_DB is None!\"}".to_string());
                }
            }
        }
    });

    // 6. 订阅 WS 事件，处理实时消息
    let mut rx = ws.subscribe();
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // 文本消息
            if msg.msg_type == 1001 {
                #[derive(serde::Deserialize)]
                struct BackendMsg {
                    id: Option<i64>,
                    room_id: Option<i64>,
                    from_uid: Option<i64>,
                    content: Option<String>,
                    #[serde(rename = "type")]
                    msg_type: Option<i16>,
                    reply_msg_id: Option<i64>,
                    status: Option<i16>,
                    extra: Option<serde_json::Value>,
                    created_at: Option<i64>,
                }

                if let Ok(bm) = serde_json::from_value::<BackendMsg>(msg.data) {
                    let xmsg = XMessage {
                        msg_id: bm.id.unwrap_or(0).to_string(),
                        room_id: bm.room_id.unwrap_or(0),
                        from_uid: bm.from_uid.unwrap_or(0),
                        content: bm.content,
                        msg_type: bm.msg_type.unwrap_or(0),
                        reply_msg_id: bm.reply_msg_id,
                        status: bm.status.unwrap_or(0),
                        extra: bm.extra.map(|v| v.to_string()),
                        local_status: 0,
                        created_at: bm.created_at.unwrap_or(0),
                    };

                    if let Some(db) = &*GLOBAL_DB.read().await {
                        xilulu_core::api::im::message::save_incoming_message(db, &xmsg).await;
                    }

                    if let Ok(guard) = FLUTTER_STREAM.lock() {
                        if let Some(sink) = guard.as_ref() {
                            if let Ok(json_str) =
                                serde_json::to_string(&XEvent::OnNewMessageReceived(xmsg))
                            {
                                let _ = sink.add(json_str);
                            }
                        }
                    }
                }
            }
            // 业务变更信令，触发增量同步
            else if msg.msg_type == 1003 || msg.msg_type == 1004 || msg.msg_type == 1005 {
                if let Some(db) = &*GLOBAL_DB.read().await {
                    if let Err(e) = xilulu_core::api::im::sync::execute_sync(
                        &crate::api::GLOBAL_API_CLIENT,
                        db,
                        my_uid,
                    )
                    .await
                    {
                        tracing::error!("事件触发同步失败: {}", e);
                    }

                    if let Ok(guard) = FLUTTER_STREAM.lock() {
                        if let Some(sink) = guard.as_ref() {
                            if let Ok(json_str) =
                                serde_json::to_string(&XEvent::OnConversationListUpdated)
                            {
                                let _ = sink.add(json_str);
                            }
                        }
                    }
                }
            }
            // WS 重连成功 → 立即触发增量同步 + 消费离线消息重发队列
            else if msg.msg_type == xilulu_core::ws::models::INTERNAL_WS_RECONNECTED {
                tracing::info!("[WS] 重连成功，触发增量同步与消息重发...");
                if let Some(db) = &*GLOBAL_DB.read().await {
                    // 通知 UI: 收取中...
                    if let Ok(guard) = FLUTTER_STREAM.lock() {
                        if let Some(sink) = guard.as_ref() {
                            let _ = sink.add(
                                serde_json::to_string(&XEvent::OnSyncStarted).unwrap(),
                            );
                        }
                    }

                    // 1. 增量同步，补齐断线期间丢失的数据
                    if let Err(e) = xilulu_core::api::im::sync::execute_sync(
                        &crate::api::GLOBAL_API_CLIENT,
                        db,
                        my_uid,
                    )
                    .await
                    {
                        tracing::error!("重连同步失败: {}", e);
                    }

                    // 2. 消费离线消息重发队列
                    if let Err(e) = xilulu_core::api::im::message::flush_sync_queue(
                        &crate::api::GLOBAL_API_CLIENT,
                        db,
                    )
                    .await
                    {
                        tracing::error!("重发队列消费失败: {}", e);
                    }

                    // 3. 通知 Flutter UI 全面刷新 + 收取完成
                    if let Ok(guard) = FLUTTER_STREAM.lock() {
                        if let Some(sink) = guard.as_ref() {
                            let _ = sink.add(
                                serde_json::to_string(&XEvent::OnConversationListUpdated).unwrap(),
                            );
                            let _ = sink.add(
                                serde_json::to_string(&XEvent::OnSyncCompleted).unwrap(),
                            );
                        }
                    }
                }
            }
            // WS 连接状态变更 → 转发给 Flutter UI 显示连接指示条
            else if msg.msg_type == xilulu_core::ws::models::INTERNAL_WS_STATUS_CHANGED {
                if let Some(status_str) = msg.data.get("status").and_then(|v| v.as_str()) {
                    if let Ok(guard) = FLUTTER_STREAM.lock() {
                        if let Some(sink) = guard.as_ref() {
                            let _ = sink.add(
                                serde_json::to_string(
                                    &XEvent::OnConnectionStatusChanged(status_str.to_string()),
                                )
                                .unwrap(),
                            );
                        }
                    }
                }
            }
        }
    });

    // 7. 定时增量同步兜底（每 3 分钟），即使 WS 正常也做一次轻量同步确保数据不过时
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(180));
        // 跳过首次立即触发（首次同步已由步骤 5 执行）
        interval.tick().await;
        loop {
            interval.tick().await;
            if let Some(db) = &*GLOBAL_DB.read().await {
                tracing::info!("[SYNC] 定时兜底同步触发");
                let _ = xilulu_core::api::im::sync::execute_sync(
                    &crate::api::GLOBAL_API_CLIENT,
                    db,
                    my_uid,
                )
                .await;
                // 同步完成后通知 Flutter 刷新
                if let Ok(guard) = FLUTTER_STREAM.lock() {
                    if let Some(sink) = guard.as_ref() {
                        let _ = sink.add(
                            serde_json::to_string(&XEvent::OnConversationListUpdated).unwrap(),
                        );
                    }
                }
            }
        }
    });

    Ok(())
}

pub fn core_subscribe_im_events(sink: StreamSink<String>) {
    if let Ok(mut guard) = FLUTTER_STREAM.lock() {
        *guard = Some(sink);
    }
}

/// Flutter 侧 App 回到前台时调用，触发增量同步确保数据最新
pub async fn core_on_app_foreground() -> Result<(), String> {
    let my_uid = crate::api::GLOBAL_STORAGE
        .get("user_id")
        .await
        .ok()
        .flatten()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    if my_uid == 0 {
        return Ok(()); // 未登录，不同步
    }

    if let Some(db) = &*GLOBAL_DB.read().await {
        tracing::info!("[SYNC] App 回到前台，触发增量同步");

        // 通知 UI: 收取中...
        if let Ok(guard) = FLUTTER_STREAM.lock() {
            if let Some(sink) = guard.as_ref() {
                let _ = sink.add(
                    serde_json::to_string(&XEvent::OnSyncStarted).unwrap(),
                );
            }
        }

        if let Err(e) = xilulu_core::api::im::sync::execute_sync(
            &crate::api::GLOBAL_API_CLIENT,
            db,
            my_uid,
        )
        .await
        {
            tracing::error!("前台恢复同步失败: {}", e);
        }

        // 通知 Flutter 刷新 + 收取完成
        if let Ok(guard) = FLUTTER_STREAM.lock() {
            if let Some(sink) = guard.as_ref() {
                let _ = sink.add(
                    serde_json::to_string(&XEvent::OnConversationListUpdated).unwrap(),
                );
                let _ = sink.add(
                    serde_json::to_string(&XEvent::OnSyncCompleted).unwrap(),
                );
            }
        }
    }

    Ok(())
}

pub async fn core_send_text_message(
    room_id: i64,
    content: String,
    from_uid: i64,
) -> Result<String, String> {
    let guard = GLOBAL_DB.read().await;
    let db = guard
        .as_ref()
        .ok_or("Database not initialized! Call core_init_sdk first.")?;

    match send_text_message(&GLOBAL_API_CLIENT, db, room_id, &content, from_uid).await {
        Ok(msg) => serde_json::to_string(&msg).map_err(|e| e.to_string()),
        Err(e) => {
            // 安全兼容性局面 JSON 错误占位
            Ok(format!("{{\"msg_id\":\"\",\"room_id\":{},\"from_uid\":{},\"type\":1,\"content\":\"NATIVE ERROR: {}\",\"status\":0,\"local_status\":2,\"created_at\":0}}",
                room_id, from_uid, e.to_string().replace("\"", "'")))
        }
    }
}

/// 从本地 SQLite 获取历史消息，返回 JSON 数组字符串
pub async fn core_get_history_messages(room_id: i64, limit: i64) -> Result<String, String> {
    let guard = GLOBAL_DB.read().await;
    let db = guard
        .as_ref()
        .ok_or("Database not initialized! Call core_init_sdk first.")?;

    let messages =
        xilulu_core::api::im::message::get_history_messages(&GLOBAL_API_CLIENT, db, room_id, limit)
            .await?;
    serde_json::to_string(&messages).map_err(|e| e.to_string())
}

/// 获取会话列表，返回 JSON 数组字符串
pub async fn core_get_contacts() -> Result<String, String> {
    let guard = GLOBAL_DB.read().await;
    let db = guard
        .as_ref()
        .ok_or("Database not initialized! Call core_init_sdk first.")?;
    let contacts = xilulu_core::api::im::contact::list_contacts_local(db).await?;
    serde_json::to_string(&contacts).map_err(|e| e.to_string())
}

/// 删除会话
pub async fn core_delete_contact(room_id: i64) -> Result<(), String> {
    xilulu_core::api::im::contact::delete_contact(&GLOBAL_API_CLIENT, room_id).await
}

/// 标记会话已读
pub async fn core_mark_read(room_id: i64) -> Result<(), String> {
    xilulu_core::api::im::contact::mark_read(&GLOBAL_API_CLIENT, room_id).await
}

/// 从远端拉取历史消息，返回 JSON 数组字符串
pub async fn core_pull_remote_messages(
    room_id: i64,
    cursor: Option<i64>,
    limit: i64,
) -> Result<String, String> {
    let guard = GLOBAL_DB.read().await;
    let db = guard
        .as_ref()
        .ok_or("Database not initialized! Call core_init_sdk first.")?;

    let messages = xilulu_core::api::im::message::pull_remote_messages(
        &GLOBAL_API_CLIENT,
        db,
        room_id,
        cursor,
        limit,
    )
    .await?;
    serde_json::to_string(&messages).map_err(|e| e.to_string())
}

/// 获取好友列表，返回 JSON 数组字符串（走本地 SQLite）
pub async fn core_list_friends() -> Result<String, String> {
    let guard = GLOBAL_DB.read().await;
    let db = guard
        .as_ref()
        .ok_or("Database not initialized! Call core_init_sdk first.")?;
    let friends = xilulu_core::api::im::friend::list_friends_local(db).await?;
    serde_json::to_string(&friends).map_err(|e| e.to_string())
}

/// 搜索用户，返回 JSON 数组字符串
pub async fn core_search_user(keyword: String) -> Result<String, String> {
    let users = xilulu_core::api::im::friend::search_user(&GLOBAL_API_CLIENT, &keyword).await?;
    serde_json::to_string(&users).map_err(|e| e.to_string())
}

/// 添加好友（发送好友申请）
pub async fn core_add_friend(target_uid: i64, message: Option<String>) -> Result<(), String> {
    xilulu_core::api::im::friend::add_friend(&GLOBAL_API_CLIENT, target_uid, message).await
}

/// 删除好友
pub async fn core_delete_friend(target_uid: i64) -> Result<(), String> {
    xilulu_core::api::im::friend::delete_friend(&GLOBAL_API_CLIENT, target_uid).await
}

/// 获取好友申请列表，返回 JSON 数组字符串
pub async fn core_list_friend_applies() -> Result<String, String> {
    let applies = xilulu_core::api::im::friend::list_friend_applies(&GLOBAL_API_CLIENT).await?;
    serde_json::to_string(&applies).map_err(|e| e.to_string())
}

/// 同意好友申请
pub async fn core_approve_friend_apply(apply_id: i64) -> Result<(), String> {
    xilulu_core::api::im::friend::approve_apply(&GLOBAL_API_CLIENT, apply_id).await
}

/// 拒绝好友申请
pub async fn core_reject_friend_apply(apply_id: i64) -> Result<(), String> {
    xilulu_core::api::im::friend::reject_apply(&GLOBAL_API_CLIENT, apply_id).await
}

/// 获取用户信息，返回 JSON 字符串
pub async fn core_get_user_info(user_id: i64) -> Result<String, String> {
    let user_info = xilulu_core::api::identity::get_user_info(&GLOBAL_API_CLIENT, user_id).await?;
    serde_json::to_string(&user_info).map_err(|e| e.to_string())
}

/// 更新用户信息，返回更新后的用户信息 JSON 字符串
pub async fn core_update_user_info(
    user_id: i64,
    nick_name: Option<String>,
    face_url: Option<String>,
    self_signature: Option<String>,
    gender: Option<i32>,
) -> Result<String, String> {
    let user_info = xilulu_core::api::identity::update_user_info(
        &GLOBAL_API_CLIENT,
        user_id,
        nick_name,
        face_url,
        self_signature,
        gender,
    )
    .await?;
    serde_json::to_string(&user_info).map_err(|e| e.to_string())
}

/// 批量获取用户信息，返回 JSON 数组字符串
pub async fn core_get_users_info(user_ids: Vec<i64>) -> Result<String, String> {
    let users_info =
        xilulu_core::api::identity::get_users_info(&GLOBAL_API_CLIENT, user_ids).await?;
    serde_json::to_string(&users_info).map_err(|e| e.to_string())
}

/// 创建群组，返回群组信息 JSON 字符串
pub async fn core_create_group(
    name: String,
    member_uids: Vec<i64>,
    introduction: Option<String>,
) -> Result<String, String> {
    let group_info = xilulu_core::api::im::room::create_group(
        &GLOBAL_API_CLIENT,
        name,
        member_uids,
        introduction,
    )
    .await?;
    serde_json::to_string(&group_info).map_err(|e| e.to_string())
}

/// 获取群组信息，返回 JSON 字符串（走本地 SQLite）
pub async fn core_get_group_info(group_id: i64) -> Result<String, String> {
    let guard = GLOBAL_DB.read().await;
    let db = guard
        .as_ref()
        .ok_or("Database not initialized! Call core_init_sdk first.")?;
    let group_info = xilulu_core::api::im::room::get_group_info_local(db, group_id).await?;
    serde_json::to_string(&group_info).map_err(|e| e.to_string())
}

/// 获取群成员列表，返回 JSON 数组字符串（走本地 SQLite）
pub async fn core_list_group_members(group_id: i64) -> Result<String, String> {
    let guard = GLOBAL_DB.read().await;
    let db = guard
        .as_ref()
        .ok_or("Database not initialized! Call core_init_sdk first.")?;
    let members = xilulu_core::api::im::room::list_group_members_local(db, group_id).await?;
    serde_json::to_string(&members).map_err(|e| e.to_string())
}

/// 退出群组
pub async fn core_quit_group(group_id: i64) -> Result<(), String> {
    xilulu_core::api::im::room::quit_group(&GLOBAL_API_CLIENT, group_id).await
}

/// 邀请成员加入群组
pub async fn core_invite_members(group_id: i64, member_uids: Vec<i64>) -> Result<(), String> {
    xilulu_core::api::im::room::invite_members(&GLOBAL_API_CLIENT, group_id, member_uids).await
}

/// 踢出群成员
pub async fn core_kick_member(group_id: i64, user_id: i64) -> Result<(), String> {
    xilulu_core::api::im::room::kick_member(&GLOBAL_API_CLIENT, group_id, user_id).await
}

/// 更新群组信息，返回更新后的群组信息 JSON 字符串
pub async fn core_update_group_info(
    group_id: i64,
    name: Option<String>,
    face_url: Option<String>,
    introduction: Option<String>,
    notification: Option<String>,
) -> Result<String, String> {
    let group_info = xilulu_core::api::im::room::update_group_info(
        &GLOBAL_API_CLIENT,
        group_id,
        name,
        face_url,
        introduction,
        notification,
    )
    .await?;
    serde_json::to_string(&group_info).map_err(|e| e.to_string())
}

/// 解散群组
pub async fn core_dismiss_group(group_id: i64) -> Result<(), String> {
    xilulu_core::api::im::room::dismiss_group(&GLOBAL_API_CLIENT, group_id).await
}
