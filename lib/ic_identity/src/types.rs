use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

/// Candid: https://k7gat-daaaa-aaaae-qaahq-cai.ic0.app/listing/internet-identity-10235/rdmx6-jaaaa-aaaaa-aaadq-cai

pub type UserNumber = u64;
pub type DeviceKey = Vec<u8>;
pub type Timestamp = u64;
pub type CredentialId = Vec<u8>;

#[allow(non_camel_case_types)]
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum Purpose {
    authentication,
    recovery,
}

#[allow(non_camel_case_types)]
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum KeyType {
    platform,
    seed_phrase,
    cross_platform,
    unknown,
}

#[allow(non_camel_case_types)]
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum DeviceProtection {
    unprotected,
    protected,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct DeviceData {
    pub alias: String,
    pub protection: DeviceProtection,
    pub pubkey: DeviceKey,
    pub key_type: KeyType,
    pub purpose: Purpose,
    pub credential_id: Option<CredentialId>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AddTentativeDeviceSuccess {
    pub verification_code: String,
    pub device_registration_timeout: Timestamp,
}

#[allow(non_camel_case_types)]
#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum AddTentativeDeviceResponse {
    // The device was tentatively added.
    added_tentatively(AddTentativeDeviceSuccess),
    // Device registration mode is off, either due to timeout or because it was never enabled.
    device_registration_mode_off,
    // There is another device already added tentatively
    another_device_tentatively_added,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct VerifyRetries {
    retries_left: u8,
}

#[allow(non_camel_case_types)]
#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum VerifyTentativeDeviceResponse {
    // The device was successfully verified.
    verified,
    // Wrong verification code entered. Retry with correct code.
    wrong_code(VerifyRetries),
    // Device registration mode is off, either due to timeout or because it was never enabled.
    device_registration_mode_off,
    // There is no tentative device to be verified.
    no_device_to_verify,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct DeviceRegistrationInfo {
    pub tentative_device: Option<DeviceData>,
    pub expiration: Timestamp,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct IdentityAnchorInfo {
    pub devices: Vec<DeviceData>,
    pub device_registration: Option<DeviceRegistrationInfo>,
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

#[allow(non_camel_case_types)]
#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum GetDelegationResponse {
    signed_delegation(SignedDelegation),
    no_such_delegation,
}
