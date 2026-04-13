use crate::api::client::ApiClient;
use crate::model::chat::ChatRoom;
use tokio::sync::watch;

pub struct ChatService {
    _api: ApiClient,
    rooms_sender: watch::Sender<Vec<ChatRoom>>,
    pub rooms_receiver: watch::Receiver<Vec<ChatRoom>>,
}

impl ChatService {
    pub fn new(api: ApiClient) -> Self {
        // Initialize with empty or cached rooms, here we use empty as it will be loaded
        let (tx, rx) = watch::channel(Vec::new());
        Self {
            _api: api,
            rooms_sender: tx,
            rooms_receiver: rx,
        }
    }

    pub async fn load_rooms(&self) -> Result<(), String> {
        // TODO: Replace with real HTTP request via self.api
        // For now, load mock data natively here to uncouple the UI.
        let mock_data = vec![
            ChatRoom {
                id: "1".to_string(),
                name: "财神爷的独生女".to_string(),
                avatar: Some("https://i.pravatar.cc/150?u=1".to_string()),
                last_msg: "劈头盖脸哈哈".to_string(),
                last_time: "10:35".to_string(),
                is_top: true,
                unread: 0,
                is_mute: false,
            },
            ChatRoom {
                id: "2".to_string(),
                name: "文件传输助手".to_string(),
                avatar: None,
                last_msg: "[图片]".to_string(),
                last_time: "3月3日".to_string(),
                is_top: true,
                unread: 0,
                is_mute: false,
            },
            ChatRoom {
                id: "3".to_string(),
                name: "公众号".to_string(),
                avatar: None,
                last_msg: "架构汪: 2026年，离职潮彻底消失了。。。".to_string(),
                last_time: "15:06".to_string(),
                is_top: false,
                unread: 1,
                is_mute: true,
            },
            ChatRoom {
                id: "4".to_string(),
                name: "美团拼好饭".to_string(),
                avatar: None,
                last_msg: "[群公告] 拼好饭-饭饭: [小程序] 大牌奶茶...".to_string(),
                last_time: "15:03".to_string(),
                is_top: false,
                unread: 2,
                is_mute: true,
            },
            ChatRoom {
                id: "5".to_string(),
                name: "前端跳槽互助群".to_string(),
                avatar: None,
                last_msg: "李哥: 今天大厂都在裁员吗？".to_string(),
                last_time: "13:15".to_string(),
                is_top: false,
                unread: 12,
                is_mute: true,
            },
            ChatRoom {
                id: "6".to_string(),
                name: "老板".to_string(),
                avatar: Some("https://i.pravatar.cc/150?u=boss".to_string()),
                last_msg: "今年的年终奖已经审批过了，过个好年".to_string(),
                last_time: "2025年1月".to_string(),
                is_top: false,
                unread: 0,
                is_mute: false,
            },
        ];

        let _ = self.rooms_sender.send(mock_data);
        Ok(())
    }

    pub fn unread_total(&self) -> u32 {
        self.rooms_receiver.borrow().iter().map(|r| r.unread).sum()
    }
}
