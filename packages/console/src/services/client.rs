//! HTTP 客户端封装
//!
//! 基于 gloo-net 的 WASM 兼容 HTTP 客户端。

use std::cell::RefCell;
use std::rc::Rc;

use gloo_net::http::Request;
use serde::{de::DeserializeOwned, Serialize};

/// API 错误
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("网络请求失败: {0}")]
    Network(String),

    #[error("JSON 解析失败: {0}")]
    Json(String),

    #[error("服务端错误 ({status}): {message}")]
    Server { status: u16, message: String },

    #[error("未授权，请重新登录")]
    Unauthorized,
}

impl From<gloo_net::Error> for ApiError {
    fn from(e: gloo_net::Error) -> Self {
        ApiError::Network(e.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        ApiError::Json(e.to_string())
    }
}

/// API 客户端
#[derive(Clone)]
pub struct ApiClient {
    /// ms-team 服务地址
    base_url: String,
    /// ms-auth 服务地址（默认与 base_url 相同）
    auth_url: String,
    /// 访问令牌
    token: Rc<RefCell<String>>,
}

impl ApiClient {
    /// 创建 API 客户端
    ///
    /// base_url: ms-team 服务地址，例如 "http://localhost:30101"
    pub fn new(base_url: impl Into<String>) -> Self {
        let url = base_url.into();
        Self {
            auth_url: url.clone(),
            base_url: url,
            token: Rc::new(RefCell::new(String::new())),
        }
    }

    /// 设置 auth 服务地址
    pub fn with_auth_url(mut self, auth_url: impl Into<String>) -> Self {
        self.auth_url = auth_url.into();
        self
    }

    /// 设置 token
    pub fn set_token(&self, token: &str) {
        *self.token.borrow_mut() = token.to_string();
    }

    /// 获取当前 token
    pub fn get_token(&self) -> String {
        self.token.borrow().clone()
    }

    /// 创建默认客户端（连接本地 ms-team）
    pub fn default() -> Self {
        Self::new("http://localhost:30101")
    }

    /// 判断路径是否走 auth 服务
    fn is_auth_path(path: &str) -> bool {
        path.starts_with("/api/v1/auth")
    }

    fn resolve_url(&self, path: &str) -> String {
        if path.starts_with("http") {
            return path.to_string();
        }
        let base = if Self::is_auth_path(path) {
            &self.auth_url
        } else {
            &self.base_url
        };
        format!("{}{}", base.trim_end_matches('/'), path)
    }

    /// GET 请求（自动附带 token）
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let url = self.resolve_url(path);
        let token = self.get_token();
        let mut req = Request::get(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");
        if !token.is_empty() {
            req = req.header("Authorization", &format!("Bearer {token}"));
        }
        let resp = req
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        self.handle_response(resp).await
    }

    /// POST 请求（自动附带 token）
    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let url = self.resolve_url(path);
        let token = self.get_token();
        let json = serde_json::to_string(body)?;
        let mut req = Request::post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");
        if !token.is_empty() && !Self::is_auth_path(path) {
            req = req.header("Authorization", &format!("Bearer {token}"));
        }
        let resp = req
            .body(json)
            .map_err(|e| ApiError::Network(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        self.handle_response(resp).await
    }

    /// PUT 请求（自动附带 token）
    pub async fn put<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let url = self.resolve_url(path);
        let token = self.get_token();
        let json = serde_json::to_string(body)?;
        let mut req = Request::put(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");
        if !token.is_empty() {
            req = req.header("Authorization", &format!("Bearer {token}"));
        }
        let resp = req
            .body(json)
            .map_err(|e| ApiError::Network(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        self.handle_response(resp).await
    }

    /// DELETE 请求（自动附带 token）
    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let url = self.resolve_url(path);
        let token = self.get_token();
        let mut req = Request::delete(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");
        if !token.is_empty() {
            req = req.header("Authorization", &format!("Bearer {token}"));
        }
        let resp = req
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        self.handle_response(resp).await
    }

    /// 处理响应
    async fn handle_response<T: DeserializeOwned>(
        &self,
        resp: gloo_net::http::Response,
    ) -> Result<T, ApiError> {
        let status = resp.status();
        if status == 401 {
            return Err(ApiError::Unauthorized);
        }
        let text = resp.text().await.map_err(|e| ApiError::Network(e.to_string()))?;
        if status >= 400 {
            if let Ok(err_resp) = serde_json::from_str::<serde_json::Value>(&text) {
                let message = err_resp
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error")
                    .to_string();
                return Err(ApiError::Server { status, message });
            }
            return Err(ApiError::Server { status, message: text });
        }
        serde_json::from_str(&text).map_err(Into::into)
    }
}
