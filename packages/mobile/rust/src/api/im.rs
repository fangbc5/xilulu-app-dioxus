use crate::frb_generated::StreamSink;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api::GLOBAL_API_CLIENT;
use xilulu_core::api::im::message::send_text_message;
pub use xilulu_core::api::im::models::{XEvent, XMessage};
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
            let _ = crate::api::GLOBAL_STORAGE.set("refresh_token", ref_tok).await;
        }
    } else {
        // Fallback for single standard token if not JSON format
        let _ = crate::api::GLOBAL_STORAGE.set("access_token", &token).await;
    }

    let ws = Arc::new(WsClient::new(url, actual_access_token, client_id));
    GLOBAL_WS_CLIENT.set(ws.clone()).unwrap_or(());

    // Spawn a listener that pipes raw WS events to Flutter & DB
    let mut rx = ws.subscribe();

    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // Assume WS server sends `{ type: 10, ... }` for New Text Messages
            if msg.msg_type == 10 {
                if let Ok(xmsg) = serde_json::from_value::<XMessage>(msg.data) {
                    // 1. Save to DB
                    if let Some(db) = GLOBAL_DB.get() {
                        xilulu_core::api::im::message::save_incoming_message(db, &xmsg).await;
                    }

                    // 2. Transmit to Flutter
                    if let Ok(guard) = FLUTTER_STREAM.lock() {
                        if let Some(sink) = guard.as_ref() {
                            if let Ok(json_str) = serde_json::to_string(&XEvent::OnNewMessageReceived(xmsg)) {
                                let _ = sink.add(json_str);
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
    room_id: u64,
    content: String,
    sender_uid: u64,
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
pub async fn core_get_history_messages(
    room_id: u64,
    limit: i64,
) -> Result<String, String> {
    let db = GLOBAL_DB
        .get()
        .ok_or("Database not initialized! Call core_init_sdk first.")?;

    let messages = xilulu_core::api::im::message::get_history_messages(db, room_id, limit).await?;
    serde_json::to_string(&messages).map_err(|e| e.to_string())
}

