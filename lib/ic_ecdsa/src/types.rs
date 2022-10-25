use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

pub type EcdsaMessageHash = Vec<u8>;
pub type EcdsaDerivationPath = Vec<Vec<u8>>;

pub type EcdsaKeyCompact = Vec<u8>;
pub type EcdsaSignatureCompact = Vec<u8>;

// #[derive(CandidType, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
// pub struct EcdsaKeyWrapper {
//     pub key: EcdsaKeyCompact,
//     pub derivation_path: EcdsaDerivationPath,
// }

#[derive(CandidType, Serialize, Debug, Clone)]
pub enum EcdsaCurve {
    #[serde(rename = "secp256k1")]
    Secp256k1,
}

#[derive(CandidType, Serialize, Debug, Clone)]
pub struct EcdsaKeyId {
    pub curve: EcdsaCurve,
    pub name: String,
}

#[derive(CandidType, Serialize, Debug)]
pub struct ECDSAPublicKey {
    pub canister_id: Option<Principal>,
    pub derivation_path: EcdsaDerivationPath,
    pub key_id: EcdsaKeyId,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct ECDSAPublicKeyReply {
    pub public_key: EcdsaKeyCompact,
    pub chain_code: Vec<u8>,
}

#[derive(CandidType, Serialize, Debug)]
pub struct SignWithECDSA {
    pub message_hash: EcdsaMessageHash,
    pub derivation_path: EcdsaDerivationPath,
    pub key_id: EcdsaKeyId,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct SignWithECDSAReply {
    pub signature: EcdsaSignatureCompact,
}
