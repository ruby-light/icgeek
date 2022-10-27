use async_trait::async_trait;

#[async_trait]
pub trait RandGenerator: Sync + Send {
    async fn generate_16(&self) -> Result<Vec<u8>, String>;

    async fn generate_32(&self) -> Result<Vec<u8>, String>;
}
