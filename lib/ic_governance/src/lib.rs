use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

/// Candid: https://k7gat-daaaa-aaaae-qaahq-cai.ic0.app/listing/nns-governance-10222/rrkah-fqaaa-aaaaa-aaaaq-cai
/// Topic: https://github.com/dfinity/ic/blob/master/rs/nns/governance/gen/ic_nns_governance.pb.v1.rs
// mod topic;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ClaimOrRefreshNeuronFromAccount {
    pub controller: Option<Principal>,
    pub memo: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ClaimOrRefreshNeuronFromAccountResponse {
    pub result: Option<Result_1>,
}

#[allow(non_camel_case_types)]
#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum Result_1 {
    Error(GovernanceError),
    NeuronId(NeuronId),
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct GovernanceError {
    pub error_message: String,
    pub error_type: i32,
}

#[derive(CandidType, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct NeuronId {
    pub id: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct BallotInfo {
    pub vote: i32,
    pub proposal_id: Option<NeuronId>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Followees {
    pub followees: Vec<NeuronId>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct NeuronStakeTransfer {
    pub to_subaccount: Vec<u8>,
    pub neuron_stake_e8s: u64,
    pub from: Option<Principal>,
    pub memo: u64,
    pub from_subaccount: Vec<u8>,
    pub transfer_timestamp: u64,
    pub block_height: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Neuron {
    pub id: Option<NeuronId>,
    pub controller: Option<Principal>,
    pub recent_ballots: Vec<BallotInfo>,
    pub kyc_verified: bool,
    pub not_for_profit: bool,
    pub maturity_e8s_equivalent: u64,
    pub cached_neuron_stake_e8s: u64,
    pub created_timestamp_seconds: u64,
    pub aging_since_timestamp_seconds: u64,
    pub hot_keys: Vec<Principal>,
    pub account: Vec<u8>,
    pub dissolve_state: Option<DissolveState>,
    pub followees: Vec<(i32, Followees)>,
    pub neuron_fees_e8s: u64,
    pub transfer: Option<NeuronStakeTransfer>,
    pub staked_maturity_e8s_equivalent: Option<u64>,
    pub auto_stake_maturity: Option<bool>,
    pub spawn_at_timestamp_seconds: Option<u64>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum DissolveState {
    DissolveDelaySeconds(u64),
    WhenDissolvedTimestampSeconds(u64),
}

#[allow(clippy::large_enum_variant)]
#[allow(non_camel_case_types)]
#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum Result_2 {
    Ok(Neuron),
    Err(GovernanceError),
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Amount {
    pub e8s: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AccountIdentifier {
    pub hash: Vec<u8>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Disburse {
    pub to_account: Option<AccountIdentifier>,
    pub amount: Option<Amount>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Spawn {
    pub new_controller: Option<Principal>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Split {
    pub amount_e8s: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Follow {
    pub topic: i32,
    pub followees: Vec<NeuronId>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Configure {
    pub operation: Option<Operation>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum Operation {
    // RemoveHotKey : RemoveHotKey;
    // AddHotKey : AddHotKey;
    // ChangeAutoStakeMaturity : ChangeAutoStakeMaturity;
    // StopDissolving : record {};
    // StartDissolving : record {};
    IncreaseDissolveDelay(IncreaseDissolveDelay),
    // JoinCommunityFund : record {};
    // LeaveCommunityFund : record {};
    // SetDissolveTimestamp : SetDissolveTimestamp;
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct IncreaseDissolveDelay {
    pub additional_dissolve_delay_seconds: u32,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum Command {
    Spawn(Spawn),
    Split(Split),
    Follow(Follow),
    // ClaimOrRefresh : ClaimOrRefresh;
    Configure(Configure),
    // RegisterVote : RegisterVote;
    // DisburseToNeuron : DisburseToNeuron;
    // MakeProposal : Proposal;
    // MergeMaturity : MergeMaturity;
    Disburse(Disburse),
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct DisburseResponse {
    pub transfer_block_height: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct SpawnResponse {
    pub created_neuron_id: Option<NeuronId>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct EmptyRecord {}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ClaimOrRefreshResponse {
    pub refreshed_neuron_id: Option<NeuronId>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct MakeProposalResponse {
    pub proposal_id: Option<NeuronId>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct MergeMaturityResponse {
    pub merged_maturity_e8s: u64,
    pub new_stake_e8s: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct StakeMaturityResponse {
    pub maturity_e8s: u64,
    pub staked_maturity_e8s: u64,
}

#[allow(non_camel_case_types)]
#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum Command_1 {
    Error(GovernanceError),
    Spawn(SpawnResponse),
    Split(SpawnResponse),
    Follow(EmptyRecord),
    ClaimOrRefresh(ClaimOrRefreshResponse),
    Configure(EmptyRecord),
    RegisterVote(EmptyRecord),
    Merge(EmptyRecord),
    DisburseToNeuron(SpawnResponse),
    MakeProposal(MakeProposalResponse),
    StakeMaturity(StakeMaturityResponse),
    MergeMaturity(MergeMaturityResponse),
    Disburse(DisburseResponse),
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum NeuronIdOrSubaccount {
    Subaccount(Vec<u8>),
    NeuronId(NeuronId),
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ManageNeuron {
    pub id: Option<NeuronId>,
    pub command: Option<Command>,
    pub neuron_id_or_subaccount: Option<NeuronIdOrSubaccount>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ManageNeuronResponse {
    pub command: Option<Command_1>,
}
