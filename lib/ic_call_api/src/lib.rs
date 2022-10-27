use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(CandidType, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum AgentRequest {
    Query(AgentQueryRequest),
    Call(AgentCallRequest),
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AgentQueryRequest {
    pub canister_id: Principal,
    pub request_sign: AgentRequestSign,
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AgentCallRequest {
    pub canister_id: Principal,
    pub request_id: AgentRequestId,
    pub request_sign: AgentRequestSign,
    pub read_state_request_sign: AgentRequestSign,
}

pub type AgentRequestId = Vec<u8>;
pub type AgentRequestSign = Vec<u8>;

#[derive(CandidType, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum AgentCallResponse {
    Ok(AgentCallResponseData),
    Error(String),
}

pub type AgentCallResponseData = Vec<u8>;
