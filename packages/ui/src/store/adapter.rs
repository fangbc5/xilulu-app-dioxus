use xilulu_im_sdk::port::StorageProvider;
use async_trait::async_trait;
use crate::store::storage::local;

pub struct DesktopStorageAdapter;

#[async_trait]
impl StorageProvider for DesktopStorageAdapter {
    async fn set(&self, key: &str, val: &str) -> Result<(), String> {
        local::set(key, val);
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<String>, String> {
        Ok(local::get(key))
    }

    async fn remove(&self, key: &str) -> Result<(), String> {
        local::remove(key);
        Ok(())
    }
}
