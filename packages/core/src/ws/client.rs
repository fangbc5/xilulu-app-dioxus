use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use reqwest_websocket::{Message, RequestBuilderExt};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

use super::models::{
    WsBaseResp, WsStatus, HEARTBEAT_INTERVAL, INITIAL_RECONNECT_DELAY, MAX_RECONNECT_DELAY,
};

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
            // dynamically fetch latest token if available to auto-heal expired reconnect loops
            let mut current_token = fallback_token.clone();
            if let Some(st) = &storage {
                if let Ok(Some(fresh_tok)) = st.get("access_token").await {
                    if !fresh_tok.is_empty() {
                        current_token = fresh_tok;
                    }
                }
            }
            let url = format!("{}?token={}&clientId={}", base_url, current_token, client_id);

            {
                let mut s = status.lock().await;
                *s = WsStatus::Connecting;
            }

            info!("[WS] Connecting to {} (Attempt: {})", url, attempts);

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
