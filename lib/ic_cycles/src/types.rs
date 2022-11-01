use candid::CandidType;
use candid::Principal;
use serde::{Deserialize, Serialize};

pub type Cycles = u128;
pub type BlockIndex = u64;
pub type XdrIcpRate = u64;
pub type TimestampSeconds = u64;

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct PrincipalId(pub Principal);

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct CanisterIdRecord {
    pub canister_id: PrincipalId,
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct NotifyTopUpArg {
    pub block_index: BlockIndex,
    pub canister_id: Principal,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum NotifyError {
    Refunded {
        reason: String,
        block_index: Option<BlockIndex>,
    },
    Processing,
    TransactionTooOld(BlockIndex),
    InvalidTransaction(String),
    Other {
        error_code: u64,
        error_message: String,
    },
}

pub type NotifyTopUpResult = Result<Cycles, NotifyError>;

#[derive(CandidType, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct IcpXdrConversionRate {
    pub xdr_permyriad_per_icp: XdrIcpRate,
    pub timestamp_seconds: TimestampSeconds,
}

#[derive(CandidType, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct IcpXdrConversionRateCertifiedResponse {
    pub certificate: Vec<u8>,
    pub data: IcpXdrConversionRate,
    pub hash_tree: Vec<u8>,
}
