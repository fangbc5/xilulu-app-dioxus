use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use serde_json::json;

use crate::ws::client::WsClient;
use crate::ws::models::{
    CallMsgType, CallRequestData, CallResponseData, IncomingCallData, CallAcceptedData, CallRoomIdData, WsBaseResp
};
use super::state::{CallState, CallStatus, CallMode};

/// Event sent to the UI/Flutter layer when RTC operations need physical handling.
#[derive(Debug, Clone)]
pub enum RtcEvent {
    IncomingCall { caller_uid: i64, is_video: bool },
    CallAccepted { token: String, url: String },
    CallRejected,
    CallTimeout,
    Dropped,
    Cancelled,
}

pub struct CallManager {
    state: Arc<Mutex<CallState>>,
    ws_client: Arc<WsClient>,
    event_tx: broadcast::Sender<RtcEvent>,
}

impl CallManager {
    pub fn new(ws_client: Arc<WsClient>) -> Self {
        let (event_tx, _) = broadcast::channel(50);
        let state = Arc::new(Mutex::new(CallState::default()));

        let manager = Self {
            state: state.clone(),
            ws_client: ws_client.clone(),
            event_tx: event_tx.clone(),
        };

        // Spawn a background task to bridge Ws events into CallState transitions
        let mut ws_rx = ws_client.subscribe();
        let state_clone = state.clone();
        let tx_clone = event_tx.clone();
        let ws_clone = ws_client.clone();
        
        tokio::spawn(async move {
            while let Ok(msg) = ws_rx.recv().await {
                Self::handle_ws_msg(msg, state_clone.clone(), tx_clone.clone(), ws_clone.clone()).await;
            }
        });

        manager
    }

    pub fn subscribe(&self) -> broadcast::Receiver<RtcEvent> {
        self.event_tx.subscribe()
    }

    pub async fn get_state(&self) -> CallState {
        self.state.lock().await.clone()
    }

    pub async fn initiate_call(&self, target_uid: i64, room_id: i64, mode: CallMode) -> Result<(), String> {
        let mut s = self.state.lock().await;
        if s.status != CallStatus::Idle {
            return Err("Call already in progress".to_string());
        }

        s.status = CallStatus::Outgoing;
        s.mode = mode;
        s.is_video = mode == CallMode::Video;
        s.room_id = room_id;
        s.remote_uid = target_uid;

        let req = CallRequestData {
            target_uid,
            room_id,
            is_video: s.is_video,
        };

        self.ws_client
            .send_json(json!({
                "type": CallMsgType::VideoCallRequest as i32,
                "data": req,
            }))
            .await?;

        Ok(())
    }

    pub async fn accept_call(&self) -> Result<(), String> {
        let mut s = self.state.lock().await;
        if s.status != CallStatus::Incoming {
            return Err("No incoming call to accept".to_string());
        }

        s.status = CallStatus::Connecting;
        let resp = CallResponseData {
            caller_uid: s.remote_uid,
            room_id: s.room_id,
            accepted: 1,
        };

        self.ws_client
            .send_json(json!({
                "type": CallMsgType::CallAccepted as i32,
                "data": resp,
            }))
            .await?;
        Ok(())
    }

    pub async fn reject_call(&self) -> Result<(), String> {
        let mut s = self.state.lock().await;
        if s.status != CallStatus::Incoming {
            return Err("No incoming call to reject".to_string());
        }

        let resp = CallResponseData {
            caller_uid: s.remote_uid,
            room_id: s.room_id,
            accepted: 0,
        };

        self.ws_client
            .send_json(json!({
                "type": CallMsgType::CallRejected as i32,
                "data": resp,
            }))
            .await?;
        
        s.reset();
        Ok(())
    }

    pub async fn cancel_call(&self) -> Result<(), String> {
        let mut s = self.state.lock().await;
        if s.status != CallStatus::Outgoing {
            return Err("No outgoing call to cancel".to_string());
        }

        let data = CallRoomIdData { room_id: s.room_id };
        self.ws_client
            .send_json(json!({
                "type": CallMsgType::Cancel as i32,
                "data": data,
            }))
            .await?;
        
        s.reset();
        Ok(())
    }

    pub async fn hangup(&self) -> Result<(), String> {
        let mut s = self.state.lock().await;
        let data = CallRoomIdData { room_id: s.room_id };
        
        self.ws_client
            .send_json(json!({
                "type": CallMsgType::Dropped as i32,
                "data": data,
            }))
            .await?;
        
        s.reset();
        Ok(())
    }

    async fn handle_ws_msg(
        msg: WsBaseResp,
        state: Arc<Mutex<CallState>>,
        event_tx: broadcast::Sender<RtcEvent>,
        ws_client: Arc<WsClient>
    ) {
        let Ok(msg_type) = CallMsgType::try_from(msg.msg_type) else {
            return; // Not a call related msg
        };

        let mut s = state.lock().await;

        match msg_type {
            CallMsgType::VideoCallRequest => {
                if let Ok(data) = serde_json::from_value::<IncomingCallData>(msg.data) {
                    if s.status != CallStatus::Idle {
                        // Busy, auto reject
                        let resp = CallResponseData {
                            caller_uid: data.caller_uid,
                            room_id: data.room_id,
                            accepted: 0,
                        };
                        let _ = ws_client.send_json(json!({
                            "type": CallMsgType::CallRejected as i32,
                            "data": resp,
                        })).await;
                        return;
                    }
                    
                    s.status = CallStatus::Incoming;
                    s.room_id = data.room_id;
                    s.remote_uid = data.caller_uid;
                    s.is_video = data.is_video;
                    s.mode = if data.is_video { CallMode::Video } else { CallMode::Audio };

                    let _ = event_tx.send(RtcEvent::IncomingCall {
                        caller_uid: data.caller_uid,
                        is_video: data.is_video,
                    });
                }
            }
            CallMsgType::CallAccepted => {
                if let Ok(data) = serde_json::from_value::<CallAcceptedData>(msg.data) {
                    s.status = CallStatus::Connecting;
                    s.livekit_token = Some(data.token.clone());
                    s.livekit_url = Some(data.livekit_url.clone());
                    let _ = event_tx.send(RtcEvent::CallAccepted {
                        token: data.token,
                        url: data.livekit_url,
                    });
                }
            }
            CallMsgType::CallRejected => {
                s.reset();
                let _ = event_tx.send(RtcEvent::CallRejected);
            }
            CallMsgType::Timeout => {
                s.reset();
                let _ = event_tx.send(RtcEvent::CallTimeout);
            }
            CallMsgType::Dropped => {
                s.reset();
                let _ = event_tx.send(RtcEvent::Dropped);
            }
            CallMsgType::Cancel => {
                s.reset();
                let _ = event_tx.send(RtcEvent::Cancelled);
            }
            CallMsgType::Heartbeat => {}
        }
    }
}
