use async_trait::async_trait;

pub type MessageHash = [u8; 32];
pub type UncompressedPublicKey = [u8; 65];
pub type EcdsaSignatureCompact = Vec<u8>;

#[async_trait]
pub trait Signer: Sync + Send {
    fn get_uncompressed_public_key(&self) -> UncompressedPublicKey;

    async fn sign(&self, message_hash: &MessageHash) -> Result<EcdsaSignatureCompact, String>;
}
