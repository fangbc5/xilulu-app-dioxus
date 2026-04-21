use serde::{Deserialize, Serialize};

/// 消息实体 —— 与服务端 `message` 表字段语义 100% 对齐
/// 客户端专有字段: `local_status`（本地发送状态，不落服务端）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XMessage {
    /// 对应服务端 message.id（整型，从服务端同步时填充；本地发送时先用 UUID 临时占位，发送成功后替换为服务端返回的 id）
    pub msg_id: String,
    /// 对应服务端 message.room_id
    pub room_id: i64,
    /// 对应服务端 message.from_uid（原字段名 sender_uid 已废弃）
    pub from_uid: i64,
    /// 对应服务端 message.content（可为 NULL，例如撤回消息）
    pub content: Option<String>,
    /// 对应服务端 message.type：1文本 2图片 3文件 4语音 5视频 6撤回 7系统
    #[serde(rename = "type")]
    pub msg_type: i16,
    /// 对应服务端 message.reply_msg_id（整型）
    pub reply_msg_id: Option<i64>,
    /// 对应服务端 message.status：0正常 1撤回
    pub status: i16,
    /// 对应服务端 message.extra（JSON 扩展字段）
    pub extra: Option<String>,
    /// 仅客户端：本地发送状态。0已完成 1发送中 2失败（不落后端）
    pub local_status: i32,
    /// 对应服务端 message.created_at（毫秒时间戳）
    pub created_at: i64,
}

/// 会话实体 —— 本地聚合视图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XContact {
    /// 对应服务端 contact.room_id
    pub room_id: i64,
    /// 对应服务端 room.type：1单聊 2群聊（本地从 room 表冗余存储在 contacts）
    pub room_type: i32,
    /// 对应服务端 contact.unread_count
    pub unread_count: i32,
    /// 对应服务端 contact.updated_at（毫秒时间戳）
    pub updated_at: i64,
    /// 本地聚合：最新一条消息（通过 last_msg_id JOIN messages 查到）
    pub last_message: Option<XMessage>,
    /// 本地聚合：显示名（单聊=对方昵称，群聊=群名，通过 JOIN 获得）
    pub show_name: Option<String>,
    /// 本地聚合：头像 URL
    pub face_url: Option<String>,
}

/// 消息事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum XEvent {
    OnNewMessageReceived(XMessage),
    OnMessagesReadReceipt(i64), // room id
    OnConversationListUpdated,
    OnTotalUnreadCountChanged(i32),
    OnChatLatestMessageUpdated(XMessage),
}
