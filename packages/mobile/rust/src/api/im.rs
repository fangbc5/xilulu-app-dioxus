use crate::frb_generated::StreamSink;
use std::sync::Arc;

use crate::api::GLOBAL_API_CLIENT;
use xilulu_core::api::im::message::send_text_message;
pub use xilulu_core::api::im::models::{XContact, XEvent, XMessage};
use xilulu_core::ws::client::WsClient;

lazy_static::lazy_static! {
    /// SDK Singleton Global Database
    pub static ref GLOBAL_DB: tokio::sync::OnceCell<xilulu_core::db::DbManager> = tokio::sync::OnceCell::new();

    /// Flutter Event Sink
    pub static ref FLUTTER_STREAM: std::sync::Mutex<Option<StreamSink<String>>> = std::sync::Mutex::new(None);

    /// Global WS Client singleton
    pub static ref GLOBAL_WS_CLIENT: tokio::sync::OnceCell<Arc<WsClient>> = tokio::sync::OnceCell::new();
}

pub async fn core_init_sdk(db_path: String) -> Result<(), String> {
    // Initialize the SQLite local persistent storage
    let db_url = format!("sqlite://{}", db_path);
    let manager = xilulu_core::db::DbManager::new(&db_url)
        .await
        .map_err(|e| e.to_string())?;
    GLOBAL_DB.set(manager).unwrap_or(());
    Ok(())
}

pub async fn core_start_ws(url: String, token: String, client_id: String) -> Result<(), String> {
    use xilulu_core::port::StorageProvider;
    let mut actual_access_token = token.clone();

    // 如果是从 Flutter Dart 层利用 JSON 夹带了 access 和 refresh 两个 Token
    // 则在这里强行提取，完美避免 FFI Signature 修改导致的编解码崩溃问题
    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&token) {
        if let Some(acc) = data.get("access").and_then(|v| v.as_str()) {
            actual_access_token = acc.to_string();
            let _ = crate::api::GLOBAL_STORAGE.set("access_token", acc).await;
        }
        if let Some(ref_tok) = data.get("refresh").and_then(|v| v.as_str()) {
            let _ = crate::api::GLOBAL_STORAGE
                .set("refresh_token", ref_tok)
                .await;
        }
    } else {
        // Fallback for single standard token if not JSON format
        let _ = crate::api::GLOBAL_STORAGE.set("access_token", &token).await;
    }

    let ws = Arc::new(WsClient::new(url, actual_access_token, client_id.clone()));
    GLOBAL_WS_CLIENT.set(ws.clone()).unwrap_or(());

    let my_uid = client_id.parse::<i64>().unwrap_or(0);

    // Initial background sync upon start
    tokio::spawn(async move {
        if let Some(db) = GLOBAL_DB.get() {
            if let Err(e) =
                xilulu_core::api::im::sync::execute_sync(&crate::api::GLOBAL_API_CLIENT, db, my_uid)
                    .await
            {
                eprintln!("Failed initial sync: {}", e);
            }
        }
    });

    // Spawn a listener that pipes raw WS events to Flutter & DB
    let mut rx = ws.subscribe();

    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // 文本消息
            if msg.msg_type == 10 {
                if let Ok(xmsg) = serde_json::from_value::<XMessage>(msg.data) {
                    // 1. Save to DB
                    if let Some(db) = GLOBAL_DB.get() {
                        xilulu_core::api::im::message::save_incoming_message(db, &xmsg).await;
                    }

                    // 2. Transmit to Flutter
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
                println!(
                    "Received business change event {}, triggering sync...",
                    msg.msg_type
                );
                if let Some(db) = GLOBAL_DB.get() {
                    if let Err(e) = xilulu_core::api::im::sync::execute_sync(
                        &crate::api::GLOBAL_API_CLIENT,
                        db,
                        my_uid,
                    )
                    .await
                    {
                        eprintln!("Failed event-triggered sync: {}", e);
                    }

                    // Notify Flutter that data has been updated, so UI can re-read DB
                    // (You can add specific events in XEvent like OnDataChanged)
                    if let Ok(guard) = FLUTTER_STREAM.lock() {
                        if let Some(sink) = guard.as_ref() {
                            if let Ok(json_str) =
                                serde_json::to_string(&XEvent::OnConversationListUpdated)
                            {
                                let _ = sink.add(json_str); // Trigger UI rebuild
                            }
                        }
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

pub async fn core_send_text_message(
    room_id: i64,
    content: String,
    sender_uid: i64,
) -> Result<String, String> {
    let db = GLOBAL_DB
        .get()
        .ok_or("Database not initialized! Call core_init_sdk first.")?;

    // Core sends, and inherently saves to internal db, pushing offline to queue
    match send_text_message(&GLOBAL_API_CLIENT, db, room_id, &content, sender_uid).await {
        Ok(msg) => serde_json::to_string(&msg).map_err(|e| e.to_string()),
        Err(e) => {
            // Safe fallback to just return a JSON with error inside content
            Ok(format!("{{\"msg_id\":\"\",\"room_id\":{},\"sender_uid\":{},\"msg_type\":0,\"content\":\"NATIVE ERROR: {}\",\"local_status\":2,\"created_at\":0}}", room_id, sender_uid, e.to_string().replace("\"", "'")))
        }
    }
}

/// 从本地 SQLite 获取历史消息，返回 JSON 数组字符串
pub async fn core_get_history_messages(room_id: i64, limit: i64) -> Result<String, String> {
    let db = GLOBAL_DB
        .get()
        .ok_or("Database not initialized! Call core_init_sdk first.")?;

    let messages =
        xilulu_core::api::im::message::get_history_messages(&GLOBAL_API_CLIENT, db, room_id, limit)
            .await?;
    serde_json::to_string(&messages).map_err(|e| e.to_string())
}

/// 获取会话列表，返回 JSON 数组字符串
pub async fn core_get_contacts() -> Result<String, String> {
    let db = GLOBAL_DB
        .get()
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
    let db = GLOBAL_DB
        .get()
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
    let db = GLOBAL_DB
        .get()
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
    let db = GLOBAL_DB
        .get()
        .ok_or("Database not initialized! Call core_init_sdk first.")?;
    let group_info = xilulu_core::api::im::room::get_group_info_local(db, group_id).await?;
    serde_json::to_string(&group_info).map_err(|e| e.to_string())
}

/// 获取群成员列表，返回 JSON 数组字符串（走本地 SQLite）
pub async fn core_list_group_members(group_id: i64) -> Result<String, String> {
    let db = GLOBAL_DB
        .get()
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
