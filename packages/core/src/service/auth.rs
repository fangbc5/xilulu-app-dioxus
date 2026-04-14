use crate::api::auth::{
    fetch_captcha, login_or_register, send_verify_code, ImageCaptchaResponse,
    LoginOrRegisterRequest, LoginOrRegisterResponse,
};
use crate::api::client::ApiClient;
use crate::model::session::{Session, SessionUserInfo};
use crate::port::StorageProvider;
use std::sync::Arc;
use tokio::sync::watch;

pub struct AuthService {
    storage: Arc<dyn StorageProvider>,
    api: ApiClient,
    session_sender: watch::Sender<Session>,
    pub session_receiver: watch::Receiver<Session>,
}

impl AuthService {
    pub async fn new(storage: Arc<dyn StorageProvider>) -> Self {
        let mut session = Session::default();

        if let Ok(Some(token)) = storage.get("access_token").await {
            session.access_token = Some(token);
        }
        if let Ok(Some(token)) = storage.get("refresh_token").await {
            session.refresh_token = Some(token);
        }
        if let Ok(Some(user_json)) = storage.get("auth_user").await {
            if let Ok(user) = serde_json::from_str(&user_json) {
                session.user = Some(user);
            }
        }

        let (tx, rx) = watch::channel(session);
        let api = ApiClient::new(storage.clone());

        Self {
            storage,
            api,
            session_sender: tx,
            session_receiver: rx,
        }
    }

    pub async fn fetch_captcha(&self) -> Result<ImageCaptchaResponse, String> {
        fetch_captcha(&self.api).await.map_err(|e| e.0)
    }

    pub async fn send_verify_code(&self, account: &str) -> Result<(), String> {
        send_verify_code(&self.api, account).await.map_err(|e| e.0)
    }

    pub async fn login_or_register(
        &self,
        mobile: Option<&str>,
        email: Option<&str>,
        code: &str,
    ) -> Result<LoginOrRegisterResponse, String> {
        let req = LoginOrRegisterRequest {
            mobile,
            email,
            code,
            region: None,
        };
        let resp = login_or_register(&self.api, req).await.map_err(|e| e.0)?;

        // Auto update state
        let _ = self
            .set_session(
                resp.login_info.access_token.clone(),
                resp.login_info.refresh_token.clone(),
                SessionUserInfo {
                    id: resp.login_info.user_info.id.clone(),
                    nickname: resp.login_info.user_info.nickname.clone(),
                    avatar: resp.login_info.user_info.avatar.clone(),
                },
            )
            .await;

        Ok(resp)
    }

    pub async fn login_with_password(
        &self,
        username: &str,
        password: &str,
        captcha_id: &str,
        captcha: &str,
    ) -> Result<crate::api::auth::LoginResponse, String> {
        let req = crate::api::auth::LoginRequest {
            username: Some(username),
            password: Some(password),
            captcha_id: Some(captcha_id),
            captcha: Some(captcha),
            mobile: None,
            email: None,
            code: None,
            region: None,
        };
        let resp = crate::api::auth::login(&self.api, req)
            .await
            .map_err(|e| e.0)?;

        // Auto update state
        let _ = self
            .set_session(
                resp.access_token.clone(),
                resp.refresh_token.clone(),
                SessionUserInfo {
                    id: resp.user_info.id.clone(),
                    nickname: resp.user_info.nickname.clone(),
                    avatar: resp.user_info.avatar.clone(),
                },
            )
            .await;

        Ok(resp)
    }

    pub async fn set_session(
        &self,
        access_token: String,
        refresh_token: String,
        user: SessionUserInfo,
    ) -> Result<(), String> {
        self.storage.set("access_token", &access_token).await?;
        self.storage.set("refresh_token", &refresh_token).await?;

        if let Ok(user_json) = serde_json::to_string(&user) {
            self.storage.set("auth_user", &user_json).await?;
        }

        let new_session = Session {
            access_token: Some(access_token),
            refresh_token: Some(refresh_token),
            user: Some(user),
        };

        let _ = self.session_sender.send(new_session);
        Ok(())
    }

    pub async fn logout(&self) -> Result<(), String> {
        let _ = crate::api::auth::logout(&self.api).await;

        self.storage.remove("access_token").await?;
        self.storage.remove("refresh_token").await?;
        self.storage.remove("auth_user").await?;

        let empty_session = Session::default();
        let _ = self.session_sender.send(empty_session);
        Ok(())
    }

    pub fn current_session(&self) -> Session {
        self.session_receiver.borrow().clone()
    }
}
