use serde::{Deserialize, Serialize};

/// 对应曾经的 V2TimMessage，现自研核心实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XMessage {
    pub msg_id: String,
    pub room_id: u64,
    pub sender_uid: u64,
    pub msg_type: i32,  // 0: Text, 1: Image, 2: Voice, 3: Video, 等等
    pub content: String, // 具体 payload
    pub local_status: i32, // 0: 成功, 1: 发送中, 2: 失败, 3: 已撤回
    pub created_at: u64,
}

/// 对应曾经的 V2TimConversation，自研核心会话清单实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XContact {
    pub room_id: u64,
    pub room_type: i32, // 1: C2C, 2: Group
    pub unread_count: i32,
    pub updated_at: u64,
    pub last_message: Option<XMessage>,
}

/// 接收到的新消息广播包裹
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum XEvent {
    OnNewMessageReceived(XMessage),
    OnMessagesReadReceipt(u64), // room id
    OnConversationListUpdated,
    OnTotalUnreadCountChanged(i32),
}
