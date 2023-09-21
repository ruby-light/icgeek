use crate::execute_ic_request;
use crate::types::{AgentError, QueryResponse};
use candid::Principal;
use ic_cdk::api::management_canister::http_request::HttpMethod;
use icgeek_ic_call_api::{AgentCallResponseData, AgentQueryRequest};

#[allow(clippy::too_many_arguments)]
pub async fn execute_ic_query(
    ic_url: String,
    request: AgentQueryRequest,
    transform_canister_id: Principal,
    transform_method: String,
    transformer_ctx: Vec<u8>,
    max_response_bytes: u64,
    cycles: u128,
) -> Result<AgentCallResponseData, AgentError> {
    let effective_canister_id = request.canister_id;
    let envelope = request.request_sign;

    let response = execute_ic_request(
        ic_url,
        HttpMethod::POST,
        &format!("canister/{effective_canister_id}/query"),
        Some(envelope),
        transform_canister_id,
        transform_method,
        transformer_ctx,
        max_response_bytes,
        cycles,
    )
    .await?;

    match (serde_cbor::from_slice(response.as_slice()) as serde_cbor::Result<QueryResponse>)
        .map_err(AgentError::InvalidCborData)?
    {
        QueryResponse::Replied { reply } => Ok(reply.arg),
        QueryResponse::Rejected(response) => Err(AgentError::ReplicaError(response)),
    }
}
