use crate::types::AgentError;
use candid::Principal;
use ic_certification::{Certificate, Delegation, HashTree, Label, LookupResult};

const IC_STATE_ROOT_DOMAIN_SEPARATOR: &[u8; 14] = b"\x0Dic-state-root";

const DER_PREFIX: &[u8; 37] = b"\x30\x81\x82\x30\x1d\x06\x0d\x2b\x06\x01\x04\x01\x82\xdc\x7c\x05\x03\x01\x02\x01\x06\x0c\x2b\x06\x01\x04\x01\x82\xdc\x7c\x05\x03\x02\x01\x03\x61\x00";
const KEY_LENGTH: usize = 96;

pub fn verify_state_response_certificate(
    cert: &Certificate,
    effective_canister_id: Principal,
    ic_root_key: Vec<u8>,
) -> Result<(), AgentError> {
    let sig = &cert.signature;

    let root_hash = cert.tree.digest();
    let mut msg = vec![];
    msg.extend_from_slice(IC_STATE_ROOT_DOMAIN_SEPARATOR);
    msg.extend_from_slice(&root_hash);

    let der_key = check_delegation(&cert.delegation, effective_canister_id, ic_root_key)?;
    let key = extract_der(der_key)?;

    ic_verify_bls_signature::verify_bls_signature(sig, &msg, &key)
        .map_err(|_| AgentError::CertificateVerificationFailed())
}

fn check_delegation(
    delegation: &Option<Delegation>,
    effective_canister_id: Principal,
    ic_root_key: Vec<u8>,
) -> Result<Vec<u8>, AgentError> {
    match delegation {
        None => Ok(ic_root_key),
        Some(delegation) => {
            let cert: Certificate = serde_cbor::from_slice(&delegation.certificate)
                .map_err(AgentError::InvalidCborData)?;

            verify_state_response_certificate(&cert, effective_canister_id, ic_root_key)?;

            let canister_range_lookup = [
                "subnet".into(),
                delegation.subnet_id.clone().into(),
                "canister_ranges".into(),
            ];
            let canister_range = lookup_value(&cert.tree, canister_range_lookup)?;
            let ranges: Vec<(Principal, Principal)> =
                serde_cbor::from_slice(canister_range).map_err(AgentError::InvalidCborData)?;

            if !principal_is_within_ranges(&effective_canister_id, &ranges[..]) {
                // the certificate is not authorized to answer calls for this canister
                return Err(AgentError::CertificateNotAuthorized());
            }

            let public_key_path = [
                "subnet".into(),
                delegation.subnet_id.clone().into(),
                "public_key".into(),
            ];
            lookup_value(&cert.tree, public_key_path).map(|pk| pk.to_vec())
        }
    }
}

// Checks if a principal is contained within a list of principal ranges
// A range is a tuple: (low: Principal, high: Principal), as described here: https://docs.dfinity.systems/spec/public/#state-tree-subnet
fn principal_is_within_ranges(principal: &Principal, ranges: &[(Principal, Principal)]) -> bool {
    ranges
        .iter()
        .any(|r| principal >= &r.0 && principal <= &r.1)
}

pub fn lookup_value<'a, P>(tree: &'a HashTree, path: P) -> Result<&'a [u8], AgentError>
where
    for<'p> &'p P: IntoIterator<Item = &'p Label>,
    P: Into<Vec<Label>>,
{
    match tree.lookup_path(&path) {
        LookupResult::Absent => Err(AgentError::LookupPathAbsent(path.into())),
        LookupResult::Unknown => Err(AgentError::LookupPathUnknown(path.into())),
        LookupResult::Found(value) => Ok(value),
        LookupResult::Error => Err(AgentError::LookupPathError(path.into())),
    }
}

pub fn extract_der(buf: Vec<u8>) -> Result<Vec<u8>, AgentError> {
    let expected_length = DER_PREFIX.len() + KEY_LENGTH;
    if buf.len() != expected_length {
        return Err(AgentError::DerKeyLengthMismatch {
            expected: expected_length,
            actual: buf.len(),
        });
    }

    let prefix = &buf[0..DER_PREFIX.len()];
    if prefix[..] != DER_PREFIX[..] {
        return Err(AgentError::DerPrefixMismatch {
            expected: DER_PREFIX.to_vec(),
            actual: prefix.to_vec(),
        });
    }

    let key = &buf[DER_PREFIX.len()..];
    Ok(key.to_vec())
}
