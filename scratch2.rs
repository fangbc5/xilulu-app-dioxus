use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ServerMessage {
    pub id: i64,
    pub room_id: i64,
    pub from_uid: i64,
    pub content: Option<String>,
    #[serde(rename = "type")]
    pub r#type: i16,
    pub reply_msg_id: Option<i64>,
    pub status: i16,
    pub extra: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

fn main() {
    let json_str = r#"{"2":[{"id":16,"room_id":2,"from_uid":54,"content":"你好","type":1,"reply_msg_id":null,"status":0,"extra":null,"created_at":"2026-03-31T08:11:49Z","updated_at":"2026-03-31T08:11:49Z"}],"1":[{"id":1,"room_id":1,"from_uid":52,"content":"1","type":1,"reply_msg_id":null,"status":0,"extra":null,"created_at":"2026-03-26T13:06:31Z","updated_at":"2026-03-26T13:06:31Z"}]}"#;
    let map: Result<HashMap<String, Vec<ServerMessage>>, _> = serde_json::from_str(json_str);
    println!("{:?}", map);
}
