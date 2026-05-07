//! API 服务层
//!
//! 所有与 ms-team 后端的 HTTP 通信都通过这里。
//! 使用 gloo-net 发起 WASM 兼容的 HTTP 请求。

pub mod auth;
pub mod client;
pub mod storage;
pub mod organization;
pub mod department;
pub mod employee;
pub mod position;
pub mod contacts;

pub use client::ApiClient;
