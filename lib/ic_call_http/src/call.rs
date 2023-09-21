use crate::types::{
    AgentError, ReadStateResponse, RejectCode, RejectResponse, Replied, RequestStatusResponse,
};
use crate::verify::{lookup_value, verify_state_response_certificate};
use crate::{deserialize_cbor_data, execute_ic_request};
use candid::Principal;
use ic_cdk::api::management_canister::http_request::HttpMethod;
use ic_certification::{Certificate, LookupResult};
use icgeek_ic_call_api::{AgentCallRequest, AgentCallResponseData};
use icgeek_ic_call_backend::request_id::RequestId;
use std::future::Future;
use std::pin::Pin;
use std::str::from_utf8;

#[allow(clippy::too_many_arguments)]
pub async fn execute_ic_call<F>(
    ic_url: String,
    request: AgentCallRequest,
    call_max_response_bytes: u64,
    call_cycles: u128,
    transform_canister_id: Principal,
    transform_method: String,
    transformer_ctx: Vec<u8>,
    sleeper: Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()>>>>,
    read_state_transform_ctx_builder: F,
    pool_max_response_bytes: u64,
    pool_cycles: u128,
    ic_root_key: Vec<u8>,
) -> Result<AgentCallResponseData, AgentError>
where
    F: FnOnce(Principal, RequestId) -> Vec<u8>,
{
    let request_id = build_request_id(&request);

    let effective_canister_id = request.canister_id;
    let envelope = request.request_sign;

    execute_ic_request(
        ic_url.clone(),
        HttpMethod::POST,
        &format!("canister/{effective_canister_id}/call"),
        Some(envelope),
        transform_canister_id,
        transform_method.clone(),
        transformer_ctx,
        call_max_response_bytes,
        call_cycles,
    )
    .await?;

    sleeper().await;

    let read_state_transformer_ctx =
        read_state_transform_ctx_builder(request.canister_id, request_id);

    match poll(
        &request_id,
        ic_url,
        effective_canister_id,
        request.read_state_request_sign,
        transform_canister_id,
        transform_method,
        read_state_transformer_ctx,
        pool_max_response_bytes,
        pool_cycles,
        ic_root_key,
    )
    .await?
    {
        PollResult::Completed(result) => Ok(result),
        PollResult::Submitted => Err(AgentError::TimeoutWaitingForResponse()),
        PollResult::Accepted => Err(AgentError::TimeoutWaitingForResponse()),
    }
}

fn build_request_id(request: &AgentCallRequest) -> RequestId {
    let mut request_id = [0_u8; 32];
    request_id.copy_from_slice(request.request_id.as_slice());
    RequestId::new(&request_id)
}

#[allow(clippy::too_many_arguments)]
async fn poll(
    request_id: &RequestId,
    ic_url: String,
    effective_canister_id: Principal,
    envelope: Vec<u8>,
    transformer_canister_id: Principal,
    transformer_method: String,
    transformer_ctx: Vec<u8>,
    max_response_bytes: u64,
    cycles: u128,
    ic_root_key: Vec<u8>,
) -> Result<PollResult, AgentError> {
    match request_state(
        request_id,
        ic_url,
        effective_canister_id,
        envelope,
        transformer_canister_id,
        transformer_method,
        transformer_ctx,
        max_response_bytes,
        cycles,
        ic_root_key,
    )
    .await?
    {
        RequestStatusResponse::Replied {
            reply: Replied::CallReplied(arg),
        } => Ok(PollResult::Completed(arg)),
        RequestStatusResponse::Unknown => Ok(PollResult::Submitted),
        RequestStatusResponse::Received | RequestStatusResponse::Processing => {
            Ok(PollResult::Accepted)
        }
        RequestStatusResponse::Rejected(response) => Err(AgentError::ReplicaError(response)),
        RequestStatusResponse::Done => Err(AgentError::RequestStatusDoneNoReply(format!(
            "{:?}",
            request_id
        ))),
    }
}

#[allow(clippy::too_many_arguments)]
async fn request_state(
    request_id: &RequestId,
    ic_url: String,
    effective_canister_id: Principal,
    envelope: Vec<u8>,
    transform_canister_id: Principal,
    transform_method: String,
    transformer_ctx: Vec<u8>,
    max_response_bytes: u64,
    cycles: u128,
    ic_root_key: Vec<u8>,
) -> Result<RequestStatusResponse, AgentError> {
    let bytes = execute_ic_request(
        ic_url,
        HttpMethod::POST,
        &format!("canister/{effective_canister_id}/read_state"),
        Some(envelope),
        transform_canister_id,
        transform_method,
        transformer_ctx,
        max_response_bytes,
        cycles,
    )
    .await?;

    let read_state_response: ReadStateResponse = deserialize_cbor_data(&bytes)?;

    let cert: Certificate = deserialize_cbor_data(&read_state_response.certificate)?;
    verify_state_response_certificate(&cert, effective_canister_id, ic_root_key)?;

    lookup_request_status(cert, request_id)
}

pub fn lookup_request_status(
    certificate: Certificate,
    request_id: &RequestId,
) -> Result<RequestStatusResponse, AgentError> {
    let path_status = [
        "request_status".into(),
        request_id.as_slice().to_vec().into(),
        "status".into(),
    ];

    match certificate.tree.lookup_path(&path_status) {
        LookupResult::Absent => Ok(RequestStatusResponse::Unknown),
        LookupResult::Unknown => Ok(RequestStatusResponse::Unknown),
        LookupResult::Found(status) => {
            match from_utf8(status).map_err(AgentError::Utf8ReadError)? {
                "done" => Ok(RequestStatusResponse::Done),
                "processing" => Ok(RequestStatusResponse::Processing),
                "received" => Ok(RequestStatusResponse::Received),
                "rejected" => lookup_rejection(&certificate, request_id),
                "replied" => lookup_reply(&certificate, request_id),
                other => Err(AgentError::InvalidRequestStatus(
                    path_status.into(),
                    other.to_string(),
                )),
            }
        }
        LookupResult::Error => Err(AgentError::LookupPathError(path_status.into())),
    }
}

fn lookup_rejection(
    certificate: &Certificate,
    request_id: &RequestId,
) -> Result<RequestStatusResponse, AgentError> {
    let reject_code = lookup_reject_code(certificate, request_id)?;
    let reject_message = lookup_reject_message(certificate, request_id)?;

    Ok(RequestStatusResponse::Rejected(RejectResponse {
        reject_code,
        reject_message,
        error_code: None,
    }))
}

fn lookup_reject_code(
    certificate: &Certificate,
    request_id: &RequestId,
) -> Result<RejectCode, AgentError> {
    let path = [
        "request_status".into(),
        request_id.as_slice().to_vec().into(),
        "reject_code".into(),
    ];
    let code = lookup_value(&certificate.tree, path)?;
    let mut readable = code;
    let code_digit = leb128::read::unsigned(&mut readable)
        .map_err(|error| AgentError::Leb128ReadError(format!("{error:?}")))?;
    RejectCode::try_from(code_digit).map_err(AgentError::Leb128ReadError)
}

fn lookup_reject_message(
    certificate: &Certificate,
    request_id: &RequestId,
) -> Result<String, AgentError> {
    let path = [
        "request_status".into(),
        request_id.as_slice().to_vec().into(),
        "reject_message".into(),
    ];
    let msg = lookup_value(&certificate.tree, path)?;
    Ok(from_utf8(msg)
        .map_err(AgentError::Utf8ReadError)?
        .to_string())
}

fn lookup_reply(
    certificate: &Certificate,
    request_id: &RequestId,
) -> Result<RequestStatusResponse, AgentError> {
    let path = [
        "request_status".into(),
        request_id.as_slice().to_vec().into(),
        "reply".into(),
    ];
    let reply_data = lookup_value(&certificate.tree, path)?;
    let reply = Replied::CallReplied(Vec::from(reply_data));
    Ok(RequestStatusResponse::Replied { reply })
}

#[derive(Debug)]
pub enum PollResult {
    /// The request has been submitted, but we do not know yet if it
    /// has been accepted or not.
    Submitted,
    /// The request has been received and may be processing.
    Accepted,
    /// The request completed and returned some data.
    Completed(Vec<u8>),
}
