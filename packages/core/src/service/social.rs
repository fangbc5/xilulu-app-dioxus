use crate::model::social::Contact;
use crate::api::client::ApiClient;
use tokio::sync::watch;

pub struct SocialService {
    _api: ApiClient,
    contacts_sender: watch::Sender<Vec<Contact>>,
    pub contacts_receiver: watch::Receiver<Vec<Contact>>,
}

impl SocialService {
    pub fn new(api: ApiClient) -> Self {
        let (tx, rx) = watch::channel(Vec::new());
        Self {
            _api: api,
            contacts_sender: tx,
            contacts_receiver: rx,
        }
    }

    pub async fn load_contacts(&self) -> Result<(), String> {
        let mock_friends = vec![
            Contact { id: "1".to_string(), group: "星标朋友".to_string(), name: "老舅".to_string(), avatar: Some("https://i.pravatar.cc/150?u=12".to_string()) },
            Contact { id: "2".to_string(), group: "星标朋友".to_string(), name: "Mather".to_string(), avatar: Some("https://i.pravatar.cc/150?u=mather".to_string()) },
            Contact { id: "3".to_string(), group: "星标朋友".to_string(), name: "亲爱的姐姐".to_string(), avatar: Some("https://i.pravatar.cc/150?u=sister".to_string()) },
            Contact { id: "4".to_string(), group: "Z".to_string(), name: "周峰".to_string(), avatar: Some("https://i.pravatar.cc/150?u=zhoufeng".to_string()) },
            Contact { id: "5".to_string(), group: "Z".to_string(), name: "周子龙".to_string(), avatar: Some("https://i.pravatar.cc/150?u=zilong".to_string()) },
            Contact { id: "6".to_string(), group: "Z".to_string(), name: "朱方方".to_string(), avatar: Some("https://i.pravatar.cc/150?u=zhu".to_string()) },
            Contact { id: "7".to_string(), group: "Z".to_string(), name: "壮哥".to_string(), avatar: Some("https://i.pravatar.cc/150?u=zhuang".to_string()) },
            Contact { id: "8".to_string(), group: "Z".to_string(), name: "自如管家王义 (全北京带看)".to_string(), avatar: Some("https://i.pravatar.cc/150?u=ziru".to_string()) },
            Contact { id: "9".to_string(), group: "Z".to_string(), name: "梓文橘子数科".to_string(), avatar: Some("https://i.pravatar.cc/150?u=ziwen".to_string()) },
            Contact { id: "10".to_string(), group: "Z".to_string(), name: "ZXB~".to_string(), avatar: Some("https://i.pravatar.cc/150?u=zxb".to_string()) },
            Contact { id: "11".to_string(), group: "#".to_string(), name: "101健身".to_string(), avatar: Some("https://i.pravatar.cc/150?u=101".to_string()) },
            Contact { id: "12".to_string(), group: "#".to_string(), name: "🍓".to_string(), avatar: Some("https://i.pravatar.cc/150?u=berry".to_string()) },
            Contact { id: "13".to_string(), group: "#".to_string(), name: "[月亮] 城市与海 .".to_string(), avatar: Some("https://i.pravatar.cc/150?u=moon".to_string()) },
        ];
        
        let _ = self.contacts_sender.send(mock_friends);
        Ok(())
    }
}
