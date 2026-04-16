use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChatRoom {
    pub id: String, // Keeping it String for generic UUID/ID compatibility
    pub name: String,
    pub avatar: Option<String>,
    pub last_msg: String,
    pub last_time: String,
    pub is_top: bool,
    pub unread: i32,
    pub is_mute: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub room_id: String,
    pub sender_id: String,
    pub content: String,
    pub timestamp: i64,
}
