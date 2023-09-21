use crate::types::{AgentError, RejectResponse};
use candid::Principal;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, TransformContext,
    TransformFunc,
};
use std::ops::Add;

pub mod call;
pub mod query;
pub mod sleeper;
pub mod types;
pub mod verify;

pub use call::*;
pub use query::*;

#[allow(clippy::too_many_arguments)]
pub(crate) async fn execute_ic_request(
    ic_url: String,
    method: HttpMethod,
    endpoint: &str,
    body: Option<Vec<u8>>,
    transform_canister_id: Principal,
    transform_method: String,
    transformer_ctx: Vec<u8>,
    max_response_bytes: u64,
    cycles: u128,
) -> Result<Vec<u8>, AgentError> {
    let url = ic_url.add(endpoint);

    let headers = vec![HttpHeader {
        name: "content-type".to_string(),
        value: "application/cbor".to_string(),
    }];

    let request = CanisterHttpRequestArgument {
        url,
        method,
        body,
        max_response_bytes: Some(max_response_bytes),
        transform: Some(TransformContext {
            function: TransformFunc(candid::Func {
                principal: transform_canister_id,
                method: transform_method,
            }),
            context: transformer_ctx,
        }),
        headers,
    };

    match http_request(request, cycles).await {
        //See:https://docs.rs/ic-cdk/latest/ic_cdk/api/management_canister/http_request/struct.HttpResponse.html
        Ok((response,)) => {
            let status: u16 = response.status.to_string().parse().unwrap();
            // let headers = response.headers;
            let body = response.body;

            // status == OK means we have an error message for call requests
            // see https://internetcomputer.org/docs/current/references/ic-interface-spec#http-call
            if status == 200 && endpoint.ends_with("call") {
                let cbor_decoded_body: Result<RejectResponse, serde_cbor::Error> =
                    serde_cbor::from_slice(&body);

                Err(match cbor_decoded_body {
                    Ok(replica_error) => AgentError::ReplicaError(replica_error),
                    Err(cbor_error) => AgentError::InvalidCborData(cbor_error),
                })
            } else if status_is_client_error(status) || status_is_server_error(status) {
                Err(AgentError::InvalidReplicaStatus(status))
                // Err(AgentError::HttpError(HttpErrorPayload {
                //     status: status.into(),
                //     content_type: headers
                //         .get("content-type")
                //         .and_then(|value| value.to_str().ok())
                //         .map(|x| x.to_string()),
                //     content: body,
                // }))
            } else {
                Ok(body)
            }
        }
        Err((rejection_code, reject_message)) => Err(AgentError::ReplicaError(RejectResponse {
            reject_code: rejection_code.into(),
            reject_message,
            error_code: None,
        })),
    }
}

pub(crate) fn status_is_client_error(status: u16) -> bool {
    (400..500).contains(&status)
}

pub(crate) fn status_is_server_error(status: u16) -> bool {
    (500..600).contains(&status)
}

pub fn deserialize_cbor_data<A>(serialized_bytes: &[u8]) -> Result<A, AgentError>
where
    A: serde::de::DeserializeOwned,
{
    serde_cbor::from_slice(serialized_bytes).map_err(AgentError::InvalidCborData)
}
