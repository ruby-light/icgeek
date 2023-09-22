use crate::types::{AgentError, QueryResponse};
use crate::{deserialize_cbor_data, execute_ic_request};
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

    execute_ic_request(
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
    .await

    // in transformer we extract query reply information
}

pub fn get_reply_from_query_response_body(response_body: &[u8]) -> Option<Vec<u8>> {
    let query_response: Result<QueryResponse, AgentError> = deserialize_cbor_data(response_body);
    if let Ok(QueryResponse::Replied { reply }) = query_response {
        Some(reply.arg)
    } else {
        None
    }
}
