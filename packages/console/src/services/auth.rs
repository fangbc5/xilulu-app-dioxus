//! 认证服务
//!
//! 对接 ms-auth /api/v1/auth 接口，管理登录态。

use serde::{Deserialize, Serialize};

use super::client::{ApiClient, ApiError};

// ==================== DTO ====================

/// 登录请求
#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub username: Option<String>,
    pub password: Option<String>,
    pub mobile: Option<String>,
    pub email: Option<String>,
    pub code: Option<String>,
    pub captcha_id: Option<String>,
    pub captcha: Option<String>,
    pub region: Option<String>,
}

/// 登录响应
#[derive(Debug, Clone, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub refresh_expires_in: i64,
    pub user_info: UserInfo,
    pub tenant_list: Option<Vec<TenantInfo>>,
}

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub nickname: String,
    pub avatar: Option<String>,
}

/// 租户信息
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TenantInfo {
    pub id: String,
    pub name: String,
    pub is_owner: Option<bool>,
}

/// 选择租户请求
#[derive(Debug, Serialize)]
pub struct SelectTenantRequest {
    pub tenant_id: i64,
    pub temp_token: String,
}

/// 选择租户响应
#[derive(Debug, Clone, Deserialize)]
pub struct SelectTenantResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub refresh_expires_in: i64,
    pub user_info: UserInfo,
}

/// 刷新 Token 请求
#[derive(Debug, Serialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// 刷新 Token 响应
#[derive(Debug, Clone, Deserialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

/// 图片验证码响应
#[derive(Debug, Clone, Deserialize)]
pub struct ImageCaptchaResponse {
    pub captcha_id: String,
    pub image_base64: String,
}

// ==================== AuthStage ====================

/// 认证阶段
#[derive(Debug, Clone, PartialEq)]
pub enum AuthStage {
    /// 未登录
    Unauthenticated,
    /// 已登录，需要选择租户（多租户场景）
    NeedTenant {
        temp_token: String,
        tenant_list: Vec<TenantInfo>,
    },
    /// 已认证
    Authenticated,
}

// ==================== API 方法 ====================

impl ApiClient {
    /// 获取图片验证码
    pub async fn auth_get_captcha(&self) -> Result<ImageCaptchaResponse, ApiError> {
        #[derive(Debug, Deserialize)]
        struct Resp {
            data: Option<ImageCaptchaResponse>,
        }
        let resp: Resp = self.get("/api/v1/auth/captcha").await?;
        resp.data.ok_or_else(|| ApiError::Server {
            status: 500,
            message: "验证码响应缺少 data".to_string(),
        })
    }

    /// 登录（发往 auth 服务）
    pub async fn auth_login(&self, req: &LoginRequest) -> Result<LoginResponse, ApiError> {
        #[derive(Debug, Deserialize)]
        struct Resp {
            data: Option<LoginResponse>,
        }
        let resp: Resp = self.post("/api/v1/auth/login", req).await?;
        resp.data.ok_or_else(|| ApiError::Server {
            status: 500,
            message: "登录响应缺少 data".to_string(),
        })
    }

    /// 选择租户
    pub async fn auth_select_tenant(
        &self,
        req: &SelectTenantRequest,
    ) -> Result<SelectTenantResponse, ApiError> {
        #[derive(Debug, Deserialize)]
        struct Resp {
            data: Option<SelectTenantResponse>,
        }
        let resp: Resp = self.post("/api/v1/auth/select-tenant", req).await?;
        resp.data.ok_or_else(|| ApiError::Server {
            status: 500,
            message: "选租户响应缺少 data".to_string(),
        })
    }

    /// 登出
    pub async fn auth_logout(&self) -> Result<(), ApiError> {
        #[derive(Debug, Deserialize)]
        struct Resp {}
        let _: Resp = self.post("/api/v1/auth/logout", &()).await?;
        Ok(())
    }

    /// 刷新 token
    pub async fn auth_refresh_token(
        &self,
        req: &RefreshTokenRequest,
    ) -> Result<RefreshTokenResponse, ApiError> {
        #[derive(Debug, Deserialize)]
        struct Resp {
            data: Option<RefreshTokenResponse>,
        }
        let resp: Resp = self
            .post("/api/v1/auth/refresh-token", req)
            .await?;
        resp.data.ok_or_else(|| ApiError::Server {
            status: 500,
            message: "刷新 token 响应缺少 data".to_string(),
        })
    }
}