use garcon::Waiter;
use ic_agent::agent::http_transport::ReqwestHttpReplicaV2Transport;
use ic_agent::agent::{PollResult, Replied, RequestStatusResponse};
use ic_agent::export::Principal;
use ic_agent::hash_tree::LookupResult;
use ic_agent::{
    agent::ReplicaV2Transport, lookup_value, Agent, AgentError, Certificate, RequestId,
};
use icgeek_ic_call_api::{
    AgentCallRequest, AgentCallResponseData, AgentQueryRequest, AgentRequest,
};
use serde::{Deserialize, Serialize};
use std::str::from_utf8;
use std::time::Duration;

pub async fn perform_request(
    agent: &Agent,
    call_transport: &ReqwestHttpReplicaV2Transport,
    request: AgentRequest,
) -> Result<AgentCallResponseData, AgentError> {
    match request {
        AgentRequest::Query(query) => perform_query(call_transport, query).await,
        AgentRequest::Call(call) => perform_call(agent, call_transport, call).await,
    }
}

async fn perform_query(
    call_transport: &ReqwestHttpReplicaV2Transport,
    request: AgentQueryRequest,
) -> Result<AgentCallResponseData, AgentError> {
    let response = call_transport
        .query(request.canister_id, request.request_sign)
        .await?;

    match (serde_cbor::from_slice(response.as_slice()) as serde_cbor::Result<QueryResponse>)? {
        QueryResponse::Replied { reply } => Ok(reply.arg),
        QueryResponse::Rejected {
            reject_code,
            reject_message,
        } => Err(AgentError::ReplicaError {
            reject_code,
            reject_message,
        }),
    }
}

async fn perform_call(
    agent: &Agent,
    call_transport: &ReqwestHttpReplicaV2Transport,
    request: AgentCallRequest,
) -> Result<AgentCallResponseData, AgentError> {
    let request_id = build_request_id(&request);

    call_transport
        .call(request.canister_id, request.request_sign, request_id)
        .await?;

    wait(
        agent,
        call_transport,
        &request_id,
        request.canister_id,
        &request.read_state_request_sign,
        create_waiter(),
    )
    .await
}

fn build_request_id(request: &AgentCallRequest) -> RequestId {
    let mut request_id = [0_u8; 32];
    request_id.copy_from_slice(request.request_id.as_slice());
    RequestId::new(&request_id)
}

fn create_waiter() -> garcon::Delay {
    garcon::Delay::builder()
        .throttle(Duration::from_millis(500))
        .timeout(Duration::from_secs(60 * 5))
        .build()
}

async fn wait<W: Waiter>(
    agent: &Agent,
    transport: &ReqwestHttpReplicaV2Transport,
    request_id: &RequestId,
    effective_canister_id: Principal,
    serialized_bytes: &[u8],
    mut waiter: W,
) -> Result<Vec<u8>, AgentError> {
    waiter.start();
    let mut request_accepted = false;
    loop {
        match poll(
            agent,
            transport,
            request_id,
            effective_canister_id,
            serialized_bytes,
        )
        .await?
        {
            PollResult::Submitted => {}
            PollResult::Accepted => {
                if !request_accepted {
                    // The system will return RequestStatusResponse::Unknown
                    // (PollResult::Submitted) until the request is accepted
                    // and we generally cannot know how long that will take.
                    // State transitions between Received and Processing may be
                    // instantaneous. Therefore, once we know the request is accepted,
                    // we should restart the waiter so the request does not time out.

                    waiter
                        .restart()
                        .map_err(|_| AgentError::WaiterRestartError())?;
                    request_accepted = true;
                }
            }
            PollResult::Completed(result) => return Ok(result),
        };

        waiter
            .async_wait()
            .await
            .map_err(|_| AgentError::TimeoutWaitingForResponse())?;
    }
}

async fn poll(
    agent: &Agent,
    transport: &ReqwestHttpReplicaV2Transport,
    request_id: &RequestId,
    effective_canister_id: Principal,
    serialized_bytes: &[u8],
) -> Result<PollResult, AgentError> {
    match request_status_raw(
        agent,
        transport,
        request_id,
        effective_canister_id,
        serialized_bytes.to_owned(),
    )
    .await?
    {
        RequestStatusResponse::Unknown => Ok(PollResult::Submitted),

        RequestStatusResponse::Received | RequestStatusResponse::Processing => {
            Ok(PollResult::Accepted)
        }

        RequestStatusResponse::Replied {
            reply: Replied::CallReplied(arg),
        } => Ok(PollResult::Completed(arg)),

        RequestStatusResponse::Rejected {
            reject_code,
            reject_message,
        } => Err(AgentError::ReplicaError {
            reject_code,
            reject_message,
        }),
        RequestStatusResponse::Done => Err(AgentError::RequestStatusDoneNoReply(String::from(
            *request_id,
        ))),
    }
}

async fn request_status_raw(
    agent: &Agent,
    transport: &ReqwestHttpReplicaV2Transport,
    request_id: &RequestId,
    effective_canister_id: Principal,
    serialized_bytes: Vec<u8>,
) -> Result<RequestStatusResponse, AgentError> {
    let cert = read_state_raw(agent, transport, effective_canister_id, serialized_bytes).await?;
    lookup_request_status(cert, request_id)
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ReadStateResponse {
    #[serde(with = "serde_bytes")]
    pub certificate: Vec<u8>,
}

async fn read_state_raw(
    agent: &Agent,
    transport: &ReqwestHttpReplicaV2Transport,
    effective_canister_id: Principal,
    serialized_bytes: Vec<u8>,
) -> Result<Certificate<'static>, AgentError> {
    let read_state_response: ReadStateResponse =
        read_state_endpoint(transport, effective_canister_id, serialized_bytes).await?;

    let cert: Certificate = serde_cbor::from_slice(&read_state_response.certificate)
        .map_err(AgentError::InvalidCborData)?;
    agent.verify(&cert, effective_canister_id)?;
    Ok(cert)
}

async fn read_state_endpoint<A>(
    transport: &ReqwestHttpReplicaV2Transport,
    effective_canister_id: Principal,
    serialized_bytes: Vec<u8>,
) -> Result<A, AgentError>
where
    A: serde::de::DeserializeOwned,
{
    let bytes = transport
        .read_state(effective_canister_id, serialized_bytes)
        .await?;
    serde_cbor::from_slice(&bytes).map_err(AgentError::InvalidCborData)
}

fn lookup_request_status(
    certificate: Certificate,
    request_id: &RequestId,
) -> Result<RequestStatusResponse, AgentError> {
    use AgentError::*;
    let path_status = [
        "request_status".into(),
        request_id.as_slice().to_vec().into(),
        "status".into(),
    ];
    match certificate.tree.lookup_path(&path_status) {
        LookupResult::Absent => Err(LookupPathAbsent(path_status.into())),
        LookupResult::Unknown => Ok(RequestStatusResponse::Unknown),
        LookupResult::Found(status) => match from_utf8(status)? {
            "done" => Ok(RequestStatusResponse::Done),
            "processing" => Ok(RequestStatusResponse::Processing),
            "received" => Ok(RequestStatusResponse::Received),
            "rejected" => lookup_rejection(&certificate, request_id),
            "replied" => lookup_reply(&certificate, request_id),
            other => Err(InvalidRequestStatus(path_status.into(), other.to_string())),
        },
        LookupResult::Error => Err(LookupPathError(path_status.into())),
    }
}

fn lookup_rejection(
    certificate: &Certificate,
    request_id: &RequestId,
) -> Result<RequestStatusResponse, AgentError> {
    let reject_code = lookup_reject_code(certificate, request_id)?;
    let reject_message = lookup_reject_message(certificate, request_id)?;

    Ok(RequestStatusResponse::Rejected {
        reject_code,
        reject_message,
    })
}

fn lookup_reject_code(
    certificate: &Certificate,
    request_id: &RequestId,
) -> Result<u64, AgentError> {
    let path = [
        "request_status".into(),
        request_id.as_slice().to_vec().into(),
        "reject_code".into(),
    ];
    let code = lookup_value(certificate, path)?;
    let mut readable = code;
    Ok(leb128::read::unsigned(&mut readable)?)
}

pub(crate) fn lookup_reject_message(
    certificate: &Certificate,
    request_id: &RequestId,
) -> Result<String, AgentError> {
    let path = [
        "request_status".into(),
        request_id.as_slice().to_vec().into(),
        "reject_message".into(),
    ];
    let msg = lookup_value(certificate, path)?;
    Ok(from_utf8(msg)?.to_string())
}

pub(crate) fn lookup_reply(
    certificate: &Certificate,
    request_id: &RequestId,
) -> Result<RequestStatusResponse, AgentError> {
    let path = [
        "request_status".into(),
        request_id.as_slice().to_vec().into(),
        "reply".into(),
    ];
    let reply_data = lookup_value(certificate, path)?;
    let reply = Replied::CallReplied(Vec::from(reply_data));
    Ok(RequestStatusResponse::Replied { reply })
}

#[derive(Debug, Clone, Deserialize)]
pub struct CallReply {
    #[serde(with = "serde_bytes")]
    pub arg: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "status")]
pub enum QueryResponse {
    #[serde(rename = "replied")]
    Replied { reply: CallReply },
    #[serde(rename = "rejected")]
    Rejected {
        reject_code: u64,
        reject_message: String,
    },
}
