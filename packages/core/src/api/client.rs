use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
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
}

impl ApiClient {
    pub fn new(storage: Arc<dyn StorageProvider>) -> Self {
        Self {
            client: Client::builder().build().unwrap(),
            storage,
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub async fn inject_auth(&self, mut req: RequestBuilder) -> RequestBuilder {
        if let Ok(Some(tok)) = self.storage.get("access_token").await {
            let end = if tok.len() > 10 { &tok[tok.len() - 10..] } else { &tok };
            tracing::info!(">>> AUTH INJECTED: token ends with: ...{}", end);
            println!(">>> AUTH INJECTED: token ends with: ...{}", end);
            req = req.bearer_auth(tok);
        } else {
            println!(">>> AUTH INJECTION FAILED: NO ACCESS TOKEN IN STORAGE");
        }
        req
    }

    async fn attempt_refresh_and_retry(&self, req_clone: Option<RequestBuilder>) -> Option<reqwest::Response> {
        let r_token = self.storage.get("refresh_token").await.ok().flatten()?;
        tracing::info!("401 Detected. Attempting token refresh...");
        
        let refresh_url = format!("{}/api/v1/auth/refresh-token", BASE_URL);
        let refresh_req = RefreshTokenReq { refresh_token: &r_token };
        
        let refresh_res = self.client.post(&refresh_url).json(&refresh_req).send().await.ok()?;
            
        if !refresh_res.status().is_success() {
            let _ = self.storage.remove("access_token").await;
            let _ = self.storage.remove("refresh_token").await;
            return None;
        }
        
        let raw_text = refresh_res.text().await.unwrap_or_default();
        let api_resp = serde_json::from_str::<ApiResponse<RefreshTokenResp>>(&raw_text).ok()?;
        
        if !api_resp.success {
            let _ = self.storage.remove("access_token").await;
            let _ = self.storage.remove("refresh_token").await;
            return None;
        }
        
        let new_tokens = api_resp.data?;
        tracing::info!("Token refresh successful. Updating storage and retrying request...");
        let _ = self.storage.set("access_token", &new_tokens.access_token).await;
        let _ = self.storage.set("refresh_token", &new_tokens.refresh_token).await;
        
        let mut clone = req_clone?;
        clone = clone.bearer_auth(&new_tokens.access_token);
        
        clone.send().await.ok()
    }

    pub async fn send_request<T: for<'de> Deserialize<'de> + std::fmt::Debug>(&self, req: RequestBuilder) -> Result<T, ApiError> {
        let req_clone = req.try_clone();
        let mut _request_url = String::from("Unknown URL");
        if let Some(ref r) = req_clone {
            if let Some(r_cloned) = r.try_clone() {
                if let Ok(built) = r_cloned.build() {
                    _request_url = built.url().to_string();
                }
            }
        }

        let mut res = req.send().await.map_err(|e| ApiError(e.to_string()))?;
        
        if res.status() == reqwest::StatusCode::UNAUTHORIZED {
            if let Some(new_res) = self.attempt_refresh_and_retry(req_clone).await {
                res = new_res;
            } else {
                // Return unauthorized error
                return Err(ApiError("UNAUTHORIZED".to_string()));
            }
        }
        
        if res.status().is_success() {
            let raw_text = res.text().await.unwrap_or_default();
            match serde_json::from_str::<ApiResponse<T>>(&raw_text) {
                Ok(api_resp) => {
                    if api_resp.success { 
                        match api_resp.data {
                            Some(d) => Ok(d),
                            None => Err(ApiError("Missing data in success response".into())),
                        }
                    } else {
                        Err(ApiError(api_resp.msg.unwrap_or_else(|| "Unknown error".to_string())))
                    }
                },
                Err(e) => Err(ApiError(e.to_string()))
            }
        } else {
            Err(ApiError(format!("Network Error: {}", res.status())))
        }
    }

    pub async fn send_action(&self, req: RequestBuilder) -> Result<(), ApiError> {
        let req_clone = req.try_clone();
        let mut res = req.send().await.map_err(|e| ApiError(e.to_string()))?;
        
        if res.status() == reqwest::StatusCode::UNAUTHORIZED {
            if let Some(new_res) = self.attempt_refresh_and_retry(req_clone).await {
                res = new_res;
            } else {
                return Err(ApiError("UNAUTHORIZED".to_string()));
            }
        }
        
        if res.status().is_success() {
            let raw_text = res.text().await.unwrap_or_default();
            match serde_json::from_str::<ApiResponse<serde_json::Value>>(&raw_text) {
                Ok(api_resp) => {
                    if api_resp.success {
                        Ok(())
                    } else {
                        Err(ApiError(api_resp.msg.unwrap_or_else(|| "Unknown error".to_string())))
                    }
                },
                Err(e) => Err(ApiError(format!("error decoding response body: {}", e)))
            }
        } else {
            Err(ApiError(format!("Network Error: {}", res.status())))
        }
    }
}
