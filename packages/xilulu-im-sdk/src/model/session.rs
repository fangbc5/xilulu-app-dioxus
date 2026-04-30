use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SessionUserInfo {
    pub id: String,
    pub nickname: String,
    pub avatar: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Session {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub user: Option<SessionUserInfo>,
}

impl Session {
    pub fn is_authenticated(&self) -> bool {
        self.access_token.is_some()
    }
}
