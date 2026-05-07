//! HTTP 客户端封装
//!
//! 基于 gloo-net 的 WASM 兼容 HTTP 客户端。
//! 支持 access_token / refresh_token 双令牌管理，
//! 401 自动刷新重试。

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
    /// 访问令牌（用于 Authorization 请求头）
    access_token: Rc<RefCell<String>>,
    /// 刷新令牌（用于 access_token 过期时刷新）
    refresh_token: Rc<RefCell<String>>,
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
            access_token: Rc::new(RefCell::new(String::new())),
            refresh_token: Rc::new(RefCell::new(String::new())),
        }
    }

    /// 设置 auth 服务地址
    pub fn with_auth_url(mut self, auth_url: impl Into<String>) -> Self {
        self.auth_url = auth_url.into();
        self
    }

    /// 设置 access_token
    pub fn set_token(&self, token: &str) {
        log::debug!("🔑 ApiClient.set_token: {}...{} chars", &token[..8.min(token.len())], token.len());
        *self.access_token.borrow_mut() = token.to_string();
    }

    /// 获取当前 access_token
    pub fn get_token(&self) -> String {
        self.access_token.borrow().clone()
    }

    /// 设置 refresh_token
    pub fn set_refresh_token(&self, token: &str) {
        *self.refresh_token.borrow_mut() = token.to_string();
    }

    /// 获取当前 refresh_token
    pub fn get_refresh_token(&self) -> String {
        self.refresh_token.borrow().clone()
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

    // ================================================================
    // Token 自动刷新
    // ================================================================

    /// 用 refresh_token 刷新 access_token。
    /// 成功返回 true 并更新内部 access_token；失败返回 false。
    async fn try_refresh_token(&self) -> bool {
        let rt = self.get_refresh_token();
        if rt.is_empty() {
            log::warn!("🔑 try_refresh_token: refresh_token 为空，无法刷新");
            return false;
        }

        log::info!("🔑 access_token 过期，正在用 refresh_token 刷新...");

        let url = self.resolve_url("/api/v1/auth/refresh-token");

        #[derive(Debug, Serialize)]
        struct Body {
            refresh_token: String,
        }
        #[derive(Debug, serde::Deserialize)]
        struct Resp {
            data: Option<RefreshData>,
        }
        #[derive(Debug, serde::Deserialize)]
        struct RefreshData {
            access_token: String,
            refresh_token: Option<String>,
        }

        let body = serde_json::to_string(&Body { refresh_token: rt }).unwrap();
        let resp = match Request::post(&url)
            .header("Content-Type", "application/json")
            .body(body)
        {
            Ok(req) => match req.send().await {
                Ok(r) => r,
                Err(e) => {
                    log::error!("🔑 refresh token 请求发送失败: {e}");
                    return false;
                }
            },
            Err(e) => {
                log::error!("🔑 refresh token 请求构建失败: {e}");
                return false;
            }
        };

        let status = resp.status();
        let text = match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                log::error!("🔑 refresh token 读取响应失败: {e}");
                return false;
            }
        };

        if status != 200 {
            log::warn!("🔑 refresh token 失败 (status={status}): {text}");
            return false;
        }

        match serde_json::from_str::<Resp>(&text) {
            Ok(resp) => {
                if let Some(data) = resp.data {
                    log::info!(
                        "🔑 token 刷新成功，新 access_token: {}...",
                        &data.access_token[..8.min(data.access_token.len())]
                    );
                    self.set_token(&data.access_token);
                    // 服务端可能返回新的 refresh_token，也可能沿用旧的
                    if let Some(new_rt) = data.refresh_token {
                        self.set_refresh_token(&new_rt);
                    }
                    true
                } else {
                    log::warn!("🔑 refresh 响应缺少 data 字段");
                    false
                }
            }
            Err(e) => {
                log::error!("🔑 refresh 响应 JSON 解析失败: {e}");
                false
            }
        }
    }

    // ================================================================
    // HTTP 方法（带 401 自动刷新重试）
    // ================================================================

    /// GET 请求（自动附带 token，401 自动刷新重试）
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let result = self.get_inner::<T>(path).await;
        self.retry_on_401(path, result, || self.get_inner::<T>(path))
            .await
    }

    /// POST 请求（自动附带 token，401 自动刷新重试）
    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let json = serde_json::to_string(body)?;
        let result = self.post_inner::<T>(path, &json).await;
        self.retry_on_401(path, result, || self.post_inner::<T>(path, &json))
            .await
    }

    /// PUT 请求（自动附带 token，401 自动刷新重试）
    pub async fn put<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let json = serde_json::to_string(body)?;
        let result = self.put_inner::<T>(path, &json).await;
        self.retry_on_401(path, result, || self.put_inner::<T>(path, &json))
            .await
    }

    /// DELETE 请求（自动附带 token，401 自动刷新重试）
    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let result = self.delete_inner::<T>(path).await;
        self.retry_on_401(path, result, || self.delete_inner::<T>(path))
            .await
    }

    /// 401 自动刷新重试：收到 401 时先尝试刷新 token，成功则重试一次原请求
    async fn retry_on_401<T: DeserializeOwned, F, Fut>(
        &self,
        path: &str,
        first_result: Result<T, ApiError>,
        retry_fn: F,
    ) -> Result<T, ApiError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, ApiError>>,
    {
        match first_result {
            Err(ApiError::Unauthorized) => {
                log::warn!("🔑 请求 {path} 返回 401，尝试刷新 token...");
                if self.try_refresh_token().await {
                    log::info!("🔑 token 刷新成功，重试请求 {path}");
                    retry_fn().await
                } else {
                    log::warn!("🔑 token 刷新失败，返回 Unauthorized");
                    Err(ApiError::Unauthorized)
                }
            }
            other => other,
        }
    }

    // ================================================================
    // 内部请求方法（无重试）
    // ================================================================

    async fn get_inner<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
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

    async fn post_inner<T: DeserializeOwned>(
        &self,
        path: &str,
        json: &str,
    ) -> Result<T, ApiError> {
        let url = self.resolve_url(path);
        let token = self.get_token();
        let mut req = Request::post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");
        if !token.is_empty() && !Self::is_auth_path(path) {
            req = req.header("Authorization", &format!("Bearer {token}"));
        }
        let resp = req
            .body(json.to_string())
            .map_err(|e| ApiError::Network(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        self.handle_response(resp).await
    }

    async fn put_inner<T: DeserializeOwned>(
        &self,
        path: &str,
        json: &str,
    ) -> Result<T, ApiError> {
        let url = self.resolve_url(path);
        let token = self.get_token();
        let mut req = Request::put(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");
        if !token.is_empty() {
            req = req.header("Authorization", &format!("Bearer {token}"));
        }
        let resp = req
            .body(json.to_string())
            .map_err(|e| ApiError::Network(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        self.handle_response(resp).await
    }

    async fn delete_inner<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
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
        let text = resp
            .text()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
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