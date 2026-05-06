//! HTTP 客户端封装
//!
//! 基于 gloo-net 的 WASM 兼容 HTTP 客户端。

use gloo_net::http::{Method, Request};
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
    base_url: String,
}

impl ApiClient {
    /// 创建 API 客户端
    ///
    /// base_url: ms-team 服务地址，例如 "http://localhost:30101"
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    /// 创建默认客户端（连接本地 ms-team）
    pub fn default() -> Self {
        Self::new("http://localhost:30101")
    }

    /// GET 请求
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        self.request(Method::GET, path, None as Option<&serde_json::Value>).await
    }

    /// POST 请求
    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        self.request(Method::POST, path, Some(body)).await
    }

    /// PUT 请求
    pub async fn put<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        self.request(Method::PUT, path, Some(body)).await
    }

    /// DELETE 请求
    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        self.request(Method::DELETE, path, None as Option<&serde_json::Value>).await
    }

    /// 内部请求方法
    async fn request<B: Serialize, T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T, ApiError> {
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{}{}", self.base_url.trim_end_matches('/'), path)
        };

        let mut req = Request::new(&url).method(method);

        // 设置 JSON content-type
        req = req.header("Content-Type", "application/json");
        req = req.header("Accept", "application/json");

        // 添加 body
        if let Some(body) = body {
            let json = serde_json::to_string(body)?;
            req = req.body(json);
        }

        let resp = req.send().await.map_err(|e| ApiError::Network(e.to_string()))?;

        let status = resp.status();

        if status == 401 {
            return Err(ApiError::Unauthorized);
        }

        let text = resp.text().await.map_err(|e| ApiError::Network(e.to_string()))?;

        if status.is_server_error() || status.is_client_error() {
            // 尝试解析错误信息
            if let Ok(err_resp) = serde_json::from_str::<serde_json::Value>(&text) {
                let message = err_resp
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error")
                    .to_string();
                return Err(ApiError::Server {
                    status: status.as_u16(),
                    message,
                });
            }
            return Err(ApiError::Server {
                status: status.as_u16(),
                message: text,
            });
        }

        serde_json::from_str(&text).map_err(Into::into)
    }
}
