use std::collections::HashMap;
use serde::Deserialize;

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

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub code: i32,
    pub msg: Option<String>,
    pub data: Option<T>,
    pub timestamp: i64,
}

fn main() {
    let json_str = r#"{"success":true,"code":200,"msg":"ok","data":{"2":[{"id":16,"room_id":2,"from_uid":54,"content":"你好","type":1,"reply_msg_id":null,"status":0,"extra":null,"created_at":"2026-03-31T08:11:49Z","updated_at":"2026-03-31T08:11:49Z"},{"id":42,"room_id":2,"from_uid":54,"content":"<p>巨物👾</p>","type":1,"reply_msg_id":null,"status":0,"extra":null,"created_at":"2026-04-08T08:10:54Z","updated_at":"2026-04-08T08:10:54Z"}],"1":[{"id":1,"room_id":1,"from_uid":52,"content":"1","type":1,"reply_msg_id":null,"status":0,"extra":null,"created_at":"2026-03-26T13:06:31Z","updated_at":"2026-03-26T13:06:31Z"}]},"timestamp":1776346977490}"#;
    let map: Result<ApiResponse<HashMap<String, Vec<ServerMessage>>>, _> = serde_json::from_str(json_str);
    match map {
        Ok(m) => println!("OK! {:?}", m.data.unwrap().keys()),
        Err(e) => println!("ERROR! {}", e),
    }
}
