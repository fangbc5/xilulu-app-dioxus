use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use reqwest_websocket::{Message, RequestBuilderExt};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

use super::models::{
    WsBaseResp, WsStatus, HEARTBEAT_INTERVAL, INITIAL_RECONNECT_DELAY, MAX_RECONNECT_DELAY,
    INTERNAL_WS_RECONNECTED, INTERNAL_WS_STATUS_CHANGED,
};

use crate::api::client::{ApiResponse, BASE_URL};
use crate::port::StorageProvider;

pub struct WsClient {
    status: Arc<Mutex<WsStatus>>,
    event_tx: broadcast::Sender<WsBaseResp>,
    cmd_tx: tokio::sync::mpsc::Sender<WsCmd>,
    storage: Option<Arc<dyn StorageProvider>>,
}

#[derive(Debug)]
pub enum WsCmd {
    Send(Message),
    Disconnect,
}

#[derive(Deserialize, Debug)]
struct WsRefreshResp {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
}

impl WsClient {
    /// Create a new WebSocket client and start its background task loop.
    pub fn new(
        url: String, 
        fallback_token: String, 
        client_id: String, 
        storage: Option<Arc<dyn StorageProvider>>
    ) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(100);
        let status = Arc::new(Mutex::new(WsStatus::Disconnected));

        let client = Self {
            status: status.clone(),
            event_tx: event_tx.clone(),
            cmd_tx,
            storage: storage.clone(),
        };

        tokio::spawn(Self::connection_loop(url, fallback_token, client_id, storage, status, event_tx, cmd_rx));

        client
    }

    /// Subscribe to WebSocket messages.
    pub fn subscribe(&self) -> broadcast::Receiver<WsBaseResp> {
        self.event_tx.subscribe()
    }

    /// Send a JSON string to the server via the WebSocket.
    pub async fn send_json(&self, data: serde_json::Value) -> Result<(), String> {
        let msg_str = serde_json::to_string(&data).map_err(|e| e.to_string())?;
        self.cmd_tx
            .send(WsCmd::Send(Message::Text(msg_str)))
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn disconnect(&self) {
        let _ = self.cmd_tx.send(WsCmd::Disconnect).await;
    }

    pub async fn status(&self) -> WsStatus {
        let s = self.status.lock().await;
        *s
    }

    /// WS 重连前主动尝试用 refresh_token 刷新 access_token，
    /// 避免拿过期 token 去连 WS 造成死循环。
    async fn try_refresh_token_for_ws(
        client: &Client,
        storage: &Arc<dyn StorageProvider>,
    ) -> Option<String> {
        let r_token = storage.get("refresh_token").await.ok().flatten()?;
        if r_token.is_empty() {
            return None;
        }

        let refresh_url = format!("{}/api/v1/auth/refresh-token", BASE_URL);
        let body = json!({ "refresh_token": r_token });

        let resp = client.post(&refresh_url).json(&body).send().await.ok()?;
        if !resp.status().is_success() {
            warn!("[WS] Token refresh returned {}", resp.status());
            return None;
        }

        let raw = resp.text().await.ok()?;
        let api_resp = serde_json::from_str::<ApiResponse<WsRefreshResp>>(&raw).ok()?;
        if !api_resp.success {
            return None;
        }
        let data = api_resp.data?;

        // 写回 storage，让 HTTP 层也能受益
        let now = chrono::Utc::now().timestamp();
        let _ = storage.set("access_token", &data.access_token).await;
        let _ = storage.set("refresh_token", &data.refresh_token).await;
        let _ = storage.set("access_expires_at", &(now + data.expires_in).to_string()).await;
        // 注意：refresh_expires_at 不更新，因为 refresh_token 未变化

        info!("[WS] Token refreshed proactively before reconnect.");
        Some(data.access_token)
    }

    /// Background loop robustly managing connection, exponential backoff, and task spawning
    async fn connection_loop(
        base_url: String,
        fallback_token: String,
        client_id: String,
        storage: Option<Arc<dyn StorageProvider>>,
        status: Arc<Mutex<WsStatus>>,
        event_tx: broadcast::Sender<WsBaseResp>,
        mut cmd_rx: tokio::sync::mpsc::Receiver<WsCmd>,
    ) {
        let client = Client::new();
        let mut retry_delay = INITIAL_RECONNECT_DELAY;
        let mut attempts = 0;

        loop {
            // 1. 先从 storage 读取最新 access_token
            let mut current_token = fallback_token.clone();
            if let Some(st) = &storage {
                if let Ok(Some(fresh_tok)) = st.get("access_token").await {
                    if !fresh_tok.is_empty() {
                        current_token = fresh_tok;
                    }
                }
            }

            // 2. 如果已经重连过 1 次以上，检查 refresh_token 是否过期
            if attempts > 0 {
                if let Some(st) = &storage {
                    // 先检查 refresh_token 是否过期
                    if let Ok(Some(refresh_expires_str)) = st.get("refresh_expires_at").await {
                        if let Ok(refresh_expires_at) = refresh_expires_str.parse::<i64>() {
                            let now = chrono::Utc::now().timestamp();
                            if now >= refresh_expires_at {
                                error!("[WS] Refresh token expired, stopping reconnection");
                                // 发送 AUTH_EXPIRED 事件通知 Flutter 跳转登录页
                                let _ = event_tx.send(WsBaseResp {
                                    msg_type: -1,
                                    data: serde_json::json!({"AUTH_EXPIRED": {}}),
                                });
                                return; // 停止重连循环
                            }
                        }
                    }

                    // refresh_token 未过期，尝试刷新 access_token
                    if let Some(new_tok) = Self::try_refresh_token_for_ws(&client, st).await {
                        current_token = new_tok;
                    }
                }
            }

            let url = format!("{}?token={}&clientId={}", base_url, current_token, client_id);

            {
                let mut s = status.lock().await;
                *s = WsStatus::Connecting;
            }
            let _ = event_tx.send(WsBaseResp {
                msg_type: INTERNAL_WS_STATUS_CHANGED,
                data: serde_json::json!({"status": "connecting"}),
            });

            info!("[WS] Connecting (Attempt: {})", attempts);

            let req = match client.get(&url).upgrade().send().await {
                Ok(r) => r,
                Err(e) => {
                    error!("[WS] Failed to upgrade request: {}", e);
                    Self::delay_reconnect(&mut retry_delay, &mut attempts).await;
                    continue;
                }
            };

            let ws = match req.into_websocket().await {
                Ok(w) => w,
                Err(e) => {
                    error!("[WS] Failed to build websocket: {}", e);
                    Self::delay_reconnect(&mut retry_delay, &mut attempts).await;
                    continue;
                }
            };

            info!("[WS] Connected successfully!");
            {
                let mut s = status.lock().await;
                *s = WsStatus::Connected;
            }
            let _ = event_tx.send(WsBaseResp {
                msg_type: INTERNAL_WS_STATUS_CHANGED,
                data: serde_json::json!({"status": "connected"}),
            });

            // 重连成功 → 通知上层触发增量同步，补齐断线期间丢失的数据
            if attempts > 0 {
                info!("[WS] Reconnected after {} attempts, notifying sync...", attempts);
                let _ = event_tx.send(WsBaseResp {
                    msg_type: INTERNAL_WS_RECONNECTED,
                    data: serde_json::json!({}),
                });
            }

            retry_delay = INITIAL_RECONNECT_DELAY;
            attempts = 0;

            let (mut ws_tx, mut ws_rx) = ws.split();

            // Spawn Heartbeat ping
            let (hb_tx, mut hb_rx) = tokio::sync::mpsc::channel(1);
            tokio::spawn(async move {
                // Delay the first tick to avoid sending data before gateway fully proxies stream
                let mut ticker = tokio::time::interval_at(
                    tokio::time::Instant::now() + HEARTBEAT_INTERVAL,
                    HEARTBEAT_INTERVAL,
                );
                loop {
                    tokio::select! {
                        _ = ticker.tick() => {
                            let ping = json!({ "type": 2, "data": {} });
                            if let Ok(msg) = serde_json::to_string(&ping) {
                                if hb_tx.send(Message::Text(msg)).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                }
            });

            loop {
                tokio::select! {
                    msg = ws_rx.next() => {
                        match msg {
                            Some(Ok(Message::Text(txt))) => {
                                if let Ok(parsed) = serde_json::from_str::<WsBaseResp>(&txt) {
                                    let _ = event_tx.send(parsed);
                                } else {
                                    warn!("[WS] Failed to parse message: {}", txt);
                                }
                            }
                            Some(Ok(Message::Close { .. })) => {
                                info!("[WS] Server closed connection.");
                                break;
                            }
                            Some(Err(e)) => {
                                error!("[WS] Error reading from stream: {}", e);
                                break;
                            }
                            None => {
                                info!("[WS] Stream closed cleanly.");
                                break;
                            }
                            _ => {} // Ignore binary and other types for now
                        }
                    }
                    cmd = cmd_rx.recv() => {
                        match cmd {
                            Some(WsCmd::Send(msg)) => {
                                if let Err(e) = ws_tx.send(msg).await {
                                    error!("[WS] Error sending message: {}", e);
                                    break;
                                }
                            }
                            Some(WsCmd::Disconnect) => {
                                info!("[WS] Disconnecting due to manual command.");
                                let mut s = status.lock().await;
                                *s = WsStatus::Disconnected;
                                return;
                            }
                            None => {
                                // cmd channel closed, exit entirely
                                return;
                            }
                        }
                    }
                    hb_msg = hb_rx.recv() => {
                        if let Some(msg) = hb_msg {
                            if let Err(e) = ws_tx.send(msg).await {
                                error!("[WS] Failed to send heartbeat: {}", e);
                                break;
                            }
                        }
                    }
                }
            }

            {
                let mut s = status.lock().await;
                *s = WsStatus::Reconnecting;
            }
            let _ = event_tx.send(WsBaseResp {
                msg_type: INTERNAL_WS_STATUS_CHANGED,
                data: serde_json::json!({"status": "reconnecting"}),
            });
            Self::delay_reconnect(&mut retry_delay, &mut attempts).await;
        }
    }

    async fn delay_reconnect(delay: &mut Duration, attempts: &mut u32) {
        *attempts += 1;
        info!(
            "[WS] Waiting {}ms before reconnecting...",
            delay.as_millis()
        );
        sleep(*delay).await;

        // Exponential backoff
        *delay = std::cmp::min(*delay * 2, MAX_RECONNECT_DELAY);
    }
}

