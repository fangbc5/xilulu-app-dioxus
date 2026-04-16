use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CallStatus {
    Idle,
    Outgoing,
    Incoming,
    Connecting,
    Connected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CallMode {
    Audio,
    Video,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallState {
    pub status: CallStatus,
    pub mode: CallMode,
    pub room_id: i64,
    pub remote_uid: i64,
    pub is_video: bool,
    pub is_muted: bool,
    pub is_camera_off: bool,
    pub is_speaker_on: bool,
    pub call_start_time: i64,
    pub livekit_token: Option<String>,
    pub livekit_url: Option<String>,
}

impl Default for CallState {
    fn default() -> Self {
        Self {
            status: CallStatus::Idle,
            mode: CallMode::Audio,
            room_id: 0,
            remote_uid: 0,
            is_video: false,
            is_muted: false,
            is_camera_off: false,
            is_speaker_on: true,
            call_start_time: 0,
            livekit_token: None,
            livekit_url: None,
        }
    }
}

impl CallState {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
