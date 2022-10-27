use candid::{CandidType, Principal};
use ic_types::hash_tree::Label;
use serde::{Deserialize, Serialize};

pub type DeviceKey = Vec<u8>;
pub type IngressExpiryDatetimeNanos = u64;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "request_type")]
pub enum CallRequestContent {
    #[serde(rename = "call")]
    CallRequest {
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(with = "serde_bytes")]
        nonce: Option<Vec<u8>>,
        ingress_expiry: IngressExpiryDatetimeNanos,
        sender: Principal,
        canister_id: Principal,
        method_name: String,
        #[serde(with = "serde_bytes")]
        arg: Vec<u8>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "request_type")]
pub enum QueryContent {
    #[serde(rename = "query")]
    QueryRequest {
        ingress_expiry: IngressExpiryDatetimeNanos,
        sender: Principal,
        canister_id: Principal,
        method_name: String,
        #[serde(with = "serde_bytes")]
        arg: Vec<u8>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "request_type")]
pub enum ReadStateContent {
    #[serde(rename = "read_state")]
    ReadStateRequest {
        ingress_expiry: IngressExpiryDatetimeNanos,
        sender: Principal,
        paths: Vec<Vec<Label>>,
    },
}

#[derive(CandidType, Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "request_type")]
pub struct Delegation {
    #[serde(with = "serde_bytes")]
    pub pubkey: DeviceKey,
    pub expiration: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub targets: Option<Vec<Principal>>,
}

#[derive(CandidType, Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "request_type")]
pub struct SignedDelegation {
    pub delegation: Delegation,
    #[serde(with = "serde_bytes")]
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Envelope<T: Serialize> {
    pub content: T,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "serde_bytes")]
    pub sender_pubkey: Option<Vec<u8>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    // #[serde(with = "serde_bytes")]
    pub sender_delegation: Option<Vec<SignedDelegation>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "serde_bytes")]
    pub sender_sig: Option<Vec<u8>>,
}
