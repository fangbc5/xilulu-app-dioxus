use std::sync::Arc;
use crate::port::StorageProvider;
use crate::service::auth::AuthService;
use crate::service::chat::ChatService;
use crate::service::social::SocialService;
use crate::api::client::ApiClient;

pub struct CoreApp {
    pub auth: Arc<AuthService>,
    pub chat: Arc<ChatService>,
    pub social: Arc<SocialService>,
    pub api_client: Arc<ApiClient>,
}

impl CoreApp {
    pub async fn new(
        storage: Arc<dyn StorageProvider>,
    ) -> Self {
        let api = ApiClient::new(storage.clone());
        // Initialize services by passing them the ports
        let auth = Arc::new(AuthService::new(storage.clone()).await);
        let chat = Arc::new(ChatService::new(api.clone()));
        let social = Arc::new(SocialService::new(api.clone()));
        
        Self {
            auth,
            chat,
            social,
            api_client: Arc::new(api),
        }
    }
}
