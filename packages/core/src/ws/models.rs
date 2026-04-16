use std::time::Duration;
use serde::{Deserialize, Serialize};

/// 消息类型的信令映射枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum CallMsgType {
    Heartbeat = 2,
    VideoCallRequest = 5,
    Timeout = 17,
    CallAccepted = 20,
    CallRejected = 21,
    Cancel = 22,
    Dropped = 23,
}

impl TryFrom<i32> for CallMsgType {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            2 => Ok(Self::Heartbeat),
            5 => Ok(Self::VideoCallRequest),
            17 => Ok(Self::Timeout),
            20 => Ok(Self::CallAccepted),
            21 => Ok(Self::CallRejected),
            22 => Ok(Self::Cancel),
            23 => Ok(Self::Dropped),
            _ => Err(format!("Unknown CallMsgType: {}", value)),
        }
    }
}

/// WebSocket 基础报文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsBaseResp {
    #[serde(rename = "type")]
    pub msg_type: i32,
    pub data: serde_json::Value,
}

/// 请求通话数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRequestData {
    pub target_uid: i64,
    pub room_id: i64,
    pub is_video: bool,
}

/// 响应通话数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallResponseData {
    pub caller_uid: i64,
    pub room_id: i64,
    pub accepted: i32, // 1 for yes, 0 for no
}

/// 接收到的来电数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingCallData {
    pub caller_uid: i64,
    pub room_id: i64,
    pub is_video: bool,
}

/// 通话已接通数据 (携带 livekit 信息)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallAcceptedData {
    pub token: String,
    pub livekit_url: String,
}

/// 用于取消或丢弃的通用 Room ID 结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRoomIdData {
    pub room_id: i64,
}

/// WebSocket 客户端连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WsStatus {
    Connecting,
    Connected,
    Disconnected,
    Reconnecting,
}

/// 控制常量
pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);
pub const MAX_RECONNECT_DELAY: Duration = Duration::from_secs(30);
pub const INITIAL_RECONNECT_DELAY: Duration = Duration::from_millis(1000);
