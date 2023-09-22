use ic_cdk::api::call::RejectionCode;
use ic_certification::Label;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::fmt::{Display, Formatter};
use std::str::Utf8Error;
use thiserror::Error;

/// An error that occurred when using the http request.
#[derive(Error, Debug)]
pub enum AgentError {
    // /// The replica URL was invalid.
    // #[error(r#"Invalid Replica URL: "{0}""#)]
    // InvalidReplicaUrl(String),
    //
    // /// The request timed out.
    #[error("The request timed out.")]
    TimeoutWaitingForResponse(),
    //
    // /// An error occurred when signing with the identity.
    // #[error("Identity had a signing error: {0}")]
    // SigningError(String),
    #[error("Invalid CBOR data, could not deserialize: {0}")]
    InvalidCborData(#[from] serde_cbor::Error),

    // /// There was an error calculating a request ID.
    // #[error("Cannot calculate a RequestID: {0}")]
    // CannotCalculateRequestId(#[from] RequestIdError),
    //
    // /// There was an error when de/serializing with Candid.
    // #[error("Candid returned an error: {0}")]
    // CandidError(Box<dyn Send + Sync + std::error::Error>),
    //
    // /// There was an error parsing a URL.
    // #[error(r#"Cannot parse url: "{0}""#)]
    // UrlParseError(#[from] url::ParseError),
    //
    // /// The HTTP method was invalid.
    // #[error(r#"Invalid method: "{0}""#)]
    // InvalidMethodError(#[from] http::method::InvalidMethod),
    //
    // /// The principal string was not a valid principal.
    // #[error("Cannot parse Principal: {0}")]
    // PrincipalError(#[from] crate::export::PrincipalError),
    #[error("The replica returned a replica error: {0}")]
    ReplicaError(RejectResponse),

    // #[error("The replica returned an HTTP Error: {0}")]
    // HttpError(HttpErrorPayload),
    #[error("Status endpoint returned an invalid status: {0}")]
    InvalidReplicaStatus(u16),

    #[error("Call was marked as done but we never saw the reply. Request ID: {0}")]
    RequestStatusDoneNoReply(String),

    // /// A string error occurred in an external tool.
    // #[error("A tool returned a string message error: {0}")]
    // MessageError(String),
    #[error("Error reading LEB128 value: {0}")]
    Leb128ReadError(String),

    // /// A string was invalid UTF-8.
    #[error("Error in UTF-8 string: {0}")]
    Utf8ReadError(#[from] Utf8Error),

    #[error("The lookup path ({0:?}) is absent in the certificate.")]
    LookupPathAbsent(Vec<Label>),

    #[error("The lookup path ({0:?}) is unknown in the certificate.")]
    LookupPathUnknown(Vec<Label>),

    #[error("The lookup path ({0:?}) does not make sense for the certificate.")]
    LookupPathError(Vec<Label>),

    #[error("The request status ({1}) at path {0:?} is invalid.")]
    InvalidRequestStatus(Vec<Label>, String),

    #[error("Certificate verification failed.")]
    CertificateVerificationFailed(),

    #[error("Certificate is not authorized to respond to queries for this canister. While developing: Did you forget to set effective_canister_id?")]
    CertificateNotAuthorized(),

    #[error(
        r#"BLS DER-encoded public key must be ${expected} bytes long, but is {actual} bytes long."#
    )]
    DerKeyLengthMismatch {
        /// The expected length of the key.
        expected: usize,
        /// The actual length of the key.
        actual: usize,
    },

    #[error("BLS DER-encoded public key is invalid. Expected the following prefix: ${expected:?}, but got ${actual:?}")]
    DerPrefixMismatch {
        /// The expected key prefix.
        expected: Vec<u8>,
        /// The actual key prefix.
        actual: Vec<u8>,
    },
    // /// The status response did not contain a root key.
    // #[error("The status response did not contain a root key.  Status: {0}")]
    // NoRootKeyInStatus(Status),
    //
    // /// The invocation to the wallet call forward method failed with an error.
    // #[error("The invocation to the wallet call forward method failed with the error: {0}")]
    // WalletCallFailed(String),
    //
    // /// The wallet operation failed.
    // #[error("The  wallet operation failed: {0}")]
    // WalletError(String),
    //
    // /// The wallet canister must be upgraded. See [`dfx wallet upgrade`](https://internetcomputer.org/docs/current/references/cli-reference/dfx-wallet)
    // #[error("The wallet canister must be upgraded: {0}")]
    // WalletUpgradeRequired(String),
    //
    // /// The transport was not specified in the [`AgentBuilder`](super::AgentBuilder).
    // #[error("Missing replica transport in the Agent Builder.")]
    // MissingReplicaTransport(),
    //
    // /// The response size exceeded the provided limit.
    // #[error("Response size exceeded limit.")]
    // ResponseSizeExceededLimit(),
    //
    // /// An unknown error occurred during communication with the replica.
    // #[error("An error happened during communication with the replica: {0}")]
    // TransportError(Box<dyn std::error::Error + Send + Sync>),
    //
    // /// There was a mismatch between the expected and actual CBOR data during inspection.
    // #[error("There is a mismatch between the CBOR encoded call and the arguments: field {field}, value in argument is {value_arg}, value in CBOR is {value_cbor}")]
    // CallDataMismatch {
    //     /// The field that was mismatched.
    //     field: String,
    //     /// The value that was expected to be in the CBOR.
    //     value_arg: String,
    //     /// The value that was actually in the CBOR.
    //     value_cbor: String,
    // },
}

#[derive(Debug, Clone, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct RejectResponse {
    /// The [reject code](https://smartcontracts.org/docs/interface-spec/index.html#reject-codes) returned by the replica.
    pub reject_code: RejectCode,
    /// The rejection message.
    pub reject_message: String,
    /// The optional [error code](https://smartcontracts.org/docs/interface-spec/index.html#error-codes) returned by the replica.
    #[serde(default)]
    pub error_code: Option<String>,
}

impl Display for RejectResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_fmt(format_args!(
            "Replica Error: reject code {:?}, reject message {}, error code {:?}",
            self.reject_code, self.reject_message, self.error_code,
        ))
    }
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr, Ord, PartialOrd,
)]
#[repr(u8)]
pub enum RejectCode {
    NoError = 0,
    /// Fatal system error, retry unlikely to be useful
    SysFatal = 1,
    /// Transient system error, retry might be possible.
    SysTransient = 2,
    /// Invalid destination (e.g. canister/account does not exist)
    DestinationInvalid = 3,
    /// Explicit reject by the canister.
    CanisterReject = 4,
    /// Canister error (e.g., trap, no response)
    CanisterError = 5,

    Unknown,
}

impl TryFrom<u64> for RejectCode {
    type Error = String;

    fn try_from(value: u64) -> Result<Self, String> {
        match value {
            1 => Ok(RejectCode::SysFatal),
            2 => Ok(RejectCode::SysTransient),
            3 => Ok(RejectCode::DestinationInvalid),
            4 => Ok(RejectCode::CanisterReject),
            5 => Ok(RejectCode::CanisterError),
            _ => Err(format!("Received an invalid reject code {value:?}")),
        }
    }
}

impl From<RejectionCode> for RejectCode {
    fn from(value: RejectionCode) -> Self {
        match value {
            RejectionCode::NoError => RejectCode::NoError,
            RejectionCode::SysFatal => RejectCode::SysFatal,
            RejectionCode::SysTransient => RejectCode::SysTransient,
            RejectionCode::DestinationInvalid => RejectCode::DestinationInvalid,
            RejectionCode::CanisterReject => RejectCode::CanisterReject,
            RejectionCode::CanisterError => RejectCode::CanisterError,
            RejectionCode::Unknown => RejectCode::Unknown,
        }
    }
}

// #[derive(Debug)]
// pub struct HttpErrorPayload {
//     /// The HTTP status code.
//     pub status: u16,
//     /// The MIME type of `content`.
//     pub content_type: Option<String>,
//     /// The body of the error.
//     pub content: Vec<u8>,
// }

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "status")]
pub enum QueryResponse {
    #[serde(rename = "replied")]
    Replied { reply: CallReply },
    #[serde(rename = "rejected")]
    Rejected(RejectResponse),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CallReply {
    #[serde(with = "serde_bytes")]
    pub arg: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ReadStateResponse {
    #[serde(with = "serde_bytes")]
    pub certificate: Vec<u8>,
}
