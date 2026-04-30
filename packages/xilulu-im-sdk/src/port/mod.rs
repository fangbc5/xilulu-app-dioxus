use async_trait::async_trait;

#[async_trait]
pub trait StorageProvider: Send + Sync {
    async fn set(&self, key: &str, val: &str) -> Result<(), String>;
    async fn get(&self, key: &str) -> Result<Option<String>, String>;
    async fn remove(&self, key: &str) -> Result<(), String>;
}

#[async_trait]
pub trait NetworkProvider: Send + Sync {
    async fn request(&self, method: &str, url: &str, body: Option<String>) -> Result<String, String>;
}
