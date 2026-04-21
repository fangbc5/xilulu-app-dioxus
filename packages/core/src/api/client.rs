use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::port::StorageProvider;

pub const BASE_URL: &str = match option_env!("BASE_URL") {
    Some(val) => val,
    None => "http://localhost:8080",
};

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub code: i32,
    pub msg: Option<String>,
    pub data: Option<T>,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct ApiError(pub String);

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for ApiError {}

#[derive(Serialize)]
struct RefreshTokenReq<'a> {
    refresh_token: &'a str,
}

#[derive(Deserialize, Debug)]
struct RefreshTokenResp {
    access_token: String,
    refresh_token: String,
}

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    storage: Arc<dyn StorageProvider>,
    /// 刷新锁：确保同一时刻只有一个请求在执行 token 刷新，其它请求等待结果
    refresh_lock: Arc<Mutex<()>>,
}

impl ApiClient {
    pub fn new(storage: Arc<dyn StorageProvider>) -> Self {
        Self {
            client: Client::builder().build().unwrap(),
            storage,
            refresh_lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub async fn inject_auth(&self, mut req: RequestBuilder) -> RequestBuilder {
        if let Ok(Some(tok)) = self.storage.get("access_token").await {
            if !tok.is_empty() {
                let end = if tok.len() > 10 { &tok[tok.len() - 10..] } else { &tok };
                tracing::info!(">>> AUTH INJECTED: token ends with: ...{}", end);
                req = req.bearer_auth(tok);
            }
        }
        req
    }

    /// 串行化 token 刷新：上锁后先检查 storage 中的 token 是否已被其它线程刷新过，
    /// 避免多个 401 并发触发 N 次 refresh 竞态。
    async fn attempt_refresh_and_retry(&self, req_clone: Option<RequestBuilder>, old_token: Option<String>) -> Option<reqwest::Response> {
        // 拿锁——同一时刻只有一个线程能进入刷新流程
        let _guard = self.refresh_lock.lock().await;

        // 在拿到锁之后，先检查 storage 里的 token 是否已被别的线程刷成了新的
        if let Some(ref old_tok) = old_token {
            if let Ok(Some(current_tok)) = self.storage.get("access_token").await {
                if &current_tok != old_tok {
                    // 别的线程已经帮我们刷新好了，直接用新 token 重试即可
                    tracing::info!("Token already refreshed by another task, reusing.");
                    let mut clone = req_clone?;
                    clone = clone.bearer_auth(&current_tok);
                    return clone.send().await.ok();
                }
            }
        }

        let r_token = self.storage.get("refresh_token").await.ok().flatten()?;
        tracing::info!("401 Detected. Attempting token refresh...");
        
        let refresh_url = format!("{}/api/v1/auth/refresh-token", BASE_URL);
        let refresh_req = RefreshTokenReq { refresh_token: &r_token };
        
        let refresh_res = match self.client.post(&refresh_url).json(&refresh_req).send().await {
            Ok(r) => r,
            Err(e) => {
                // 网络瞬断等暂态故障，不删除 token，下次请求再重试
                tracing::warn!("Token refresh network error (tokens preserved): {}", e);
                return None;
            }
        };
            
        if !refresh_res.status().is_success() {
            let status = refresh_res.status();
            tracing::warn!("Token refresh returned {}, clearing tokens.", status);
            let _ = self.storage.remove("access_token").await;
            let _ = self.storage.remove("refresh_token").await;
            return None;
        }
        
        let raw_text = refresh_res.text().await.unwrap_or_default();
        let api_resp = serde_json::from_str::<ApiResponse<RefreshTokenResp>>(&raw_text).ok()?;
        
        if !api_resp.success {
            tracing::warn!("Token refresh response unsuccessful, clearing tokens.");
            let _ = self.storage.remove("access_token").await;
            let _ = self.storage.remove("refresh_token").await;
            return None;
        }
        
        let new_tokens = api_resp.data?;
        tracing::info!("Token refresh successful. Updating storage and retrying request.");
        let _ = self.storage.set("access_token", &new_tokens.access_token).await;
        let _ = self.storage.set("refresh_token", &new_tokens.refresh_token).await;
        
        let mut clone = req_clone?;
        clone = clone.bearer_auth(&new_tokens.access_token);
        
        clone.send().await.ok()
    }

    pub async fn send_request<T: for<'de> Deserialize<'de> + std::fmt::Debug>(&self, req: RequestBuilder) -> Result<T, ApiError> {
        let req_clone = req.try_clone();
        // 记录当前使用的 token，用于刷新时的竞态检测
        let old_token = self.storage.get("access_token").await.ok().flatten();

        let mut res = req.send().await.map_err(|e| ApiError(e.to_string()))?;
        
        if res.status() == reqwest::StatusCode::UNAUTHORIZED {
            if let Some(new_res) = self.attempt_refresh_and_retry(req_clone, old_token).await {
                res = new_res;
            } else {
                return Err(ApiError("UNAUTHORIZED".to_string()));
            }
        }
        
        let status = res.status();
        let raw_text = res.text().await.unwrap_or_default();
        
        if let Ok(api_resp) = serde_json::from_str::<ApiResponse<T>>(&raw_text) {
            if api_resp.success { 
                match api_resp.data {
                    Some(d) => Ok(d),
                    None => Err(ApiError("Missing data in success response".into())),
                }
            } else {
                Err(ApiError(api_resp.msg.unwrap_or_else(|| "Unknown error".to_string())))
            }
        } else {
            if status.is_success() {
                Err(ApiError("Failed to parse successful response".into()))
            } else {
                Err(ApiError(format!("Network Error: {}", status)))
            }
        }
    }

    pub async fn send_action(&self, req: RequestBuilder) -> Result<(), ApiError> {
        let req_clone = req.try_clone();
        let old_token = self.storage.get("access_token").await.ok().flatten();
        let mut res = req.send().await.map_err(|e| ApiError(e.to_string()))?;
        
        if res.status() == reqwest::StatusCode::UNAUTHORIZED {
            if let Some(new_res) = self.attempt_refresh_and_retry(req_clone, old_token).await {
                res = new_res;
            } else {
                return Err(ApiError("UNAUTHORIZED".to_string()));
            }
        }
        
        let status = res.status();
        let raw_text = res.text().await.unwrap_or_default();

        if let Ok(api_resp) = serde_json::from_str::<ApiResponse<serde_json::Value>>(&raw_text) {
            if api_resp.success {
                Ok(())
            } else {
                Err(ApiError(api_resp.msg.unwrap_or_else(|| "Unknown error".to_string())))
            }
        } else {
            if status.is_success() {
                Err(ApiError("Failed to parse successful response".into()))
            } else {
                Err(ApiError(format!("Network Error: {}", status)))
            }
        }
    }
}
