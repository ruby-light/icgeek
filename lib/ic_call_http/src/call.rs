use crate::types::{AgentError, ReadStateResponse, RejectCode, RejectResponse};
use crate::verify::lookup_value;
use crate::{deserialize_cbor_data, execute_ic_request};
use candid::Principal;
use ic_cdk::api::management_canister::http_request::HttpMethod;
use ic_certification::{Certificate, LookupResult};
use icgeek_ic_call_api::{AgentCallRequest, AgentCallResponseData};
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
    F: FnOnce(Principal, Vec<u8>, Vec<u8>) -> Vec<u8>,
{
    let request_id = request.request_id.clone();

    let effective_canister_id = request.canister_id;
    let envelope = request.request_sign;

    // send call request

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

    let read_state_transformer_ctx =
        read_state_transform_ctx_builder(request.canister_id, request_id, ic_root_key);

    // wait timeout while all replicas make calls

    sleeper().await;

    // request reply from transformer

    request_response_data(
        ic_url,
        effective_canister_id,
        request.read_state_request_sign,
        transform_canister_id,
        transform_method,
        read_state_transformer_ctx,
        pool_max_response_bytes,
        pool_cycles,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn request_response_data(
    ic_url: String,
    effective_canister_id: Principal,
    envelope: Vec<u8>,
    transform_canister_id: Principal,
    transform_method: String,
    transformer_ctx: Vec<u8>,
    max_response_bytes: u64,
    cycles: u128,
) -> Result<Vec<u8>, AgentError> {
    execute_ic_request(
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
    .await

    // in transformer extract call response data
}

pub fn get_certificate_from_state_response_body(
    response_body: &[u8],
) -> Result<Certificate, AgentError> {
    let read_state_response: ReadStateResponse = deserialize_cbor_data(response_body)?;
    deserialize_cbor_data(&read_state_response.certificate)
}

pub fn get_reply_from_call_response_certificate(
    certificate: Certificate,
    request_id: &Vec<u8>,
) -> Option<AgentCallResponseData> {
    if let Ok(RequestStatusResponse::Replied {
        reply: Replied::CallReplied(data),
    }) = lookup_request_status(certificate, request_id)
    {
        Some(data)
    } else {
        None
    }
}

pub fn lookup_request_status(
    certificate: Certificate,
    request_id: &Vec<u8>,
) -> Result<RequestStatusResponse, AgentError> {
    let path_status = [
        "request_status".into(),
        request_id.clone().into(),
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
    request_id: &Vec<u8>,
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
    request_id: &Vec<u8>,
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
    request_id: &Vec<u8>,
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
    request_id: &Vec<u8>,
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
pub enum RequestStatusResponse {
    /// The status of the request is unknown.
    Unknown,
    /// The request has been received, and will probably get processed.
    Received,
    /// The request is currently being processed.
    Processing,
    /// The request has been successfully replied to.
    Replied {
        /// The reply from the replica.
        reply: Replied,
    },
    /// The request has been rejected.
    Rejected(RejectResponse),
    /// The call has been completed, and it has been long enough that the reply/reject data has been purged, but the call has not expired yet.
    Done,
}

#[derive(Debug)]
pub enum Replied {
    CallReplied(Vec<u8>),
}
