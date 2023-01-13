use candid::Principal;
use ic_certification::{Certificate, Delegation, HashTree, Label, LookupResult};
use ic_verify_bls_signature::verify_bls_signature;

const DER_PREFIX: &[u8; 37] = b"\x30\x81\x82\x30\x1d\x06\x0d\x2b\x06\x01\x04\x01\x82\xdc\x7c\x05\x03\x01\x02\x01\x06\x0c\x2b\x06\x01\x04\x01\x82\xdc\x7c\x05\x03\x02\x01\x03\x61\x00";
const KEY_LENGTH: usize = 96;
const IC_STATE_ROOT_DOMAIN_SEPARATOR: &[u8; 14] = b"\x0Dic-state-root";

pub fn verify_certified_data(
    certificate: &[u8],
    canister_id: &Principal,
    root_pk: &[u8],
    certified_data: &[u8],
) -> Result<(), String> {
    let verified_certificate = verify_certificate(certificate, root_pk)?;

    let certified_data_path = [
        "canister".into(),
        canister_id.into(),
        "certified_data".into(),
    ];

    let certificate_certified_data = lookup_value(&verified_certificate.tree, certified_data_path)?;
    if certified_data != certificate_certified_data {
        return Err("wrong certified_data".to_owned());
    }

    Ok(())
}

/// Verification of the delegation certificate ensures that
/// * the certificate is well-formed and contains a tree, a signature, and
///   _no_ further delegation, i.e., it comes directly from the root subnet,
/// * the signature is valid w.r.t. `root_pk`,
/// * the tree is well-formed and contains time as well as subnet information
///   (i.e., a public_key and canister ranges) for the given subnet,
/// * the canister ranges are well-formed and contain the `canister_id`, and
/// * the public key is well-formed.
///
/// Returns the verified certificate, if verification is successful.
pub fn verify_certificate<'a>(
    certificate: &'a [u8],
    root_pk: &'a [u8],
) -> Result<Certificate<'a>, String> {
    let certificate: Certificate = parse_certificate(certificate)?;
    verify(root_pk.to_vec(), &certificate)?;
    Ok(certificate)
}

pub fn parse_certificate(certificate: &[u8]) -> Result<Certificate, String> {
    serde_cbor::from_slice(certificate)
        .map_err(|err| format!("failed to decode certificate: {}", err))
}

/// Verify a certificate, checking delegation if present.
pub fn verify(root_key: Vec<u8>, cert: &Certificate) -> Result<(), String> {
    let sig = &cert.signature;

    let root_hash = cert.tree.digest();
    let mut msg = vec![];
    msg.extend_from_slice(IC_STATE_ROOT_DOMAIN_SEPARATOR);
    msg.extend_from_slice(&root_hash);

    let der_key = check_delegation(root_key, &cert.delegation)?;
    let key = extract_der(der_key)?;

    verify_bls_signature(sig, &msg, &key).map_err(|_| "fail verify signature".to_owned())
}

pub fn extract_der(buf: Vec<u8>) -> Result<Vec<u8>, String> {
    let expected_length = DER_PREFIX.len() + KEY_LENGTH;
    if buf.len() != expected_length {
        return Err("DerKeyLengthMismatch".to_owned());
    }

    let prefix = &buf[0..DER_PREFIX.len()];
    if prefix[..] != DER_PREFIX[..] {
        return Err("DerPrefixMismatch".to_owned());
    }

    let key = &buf[DER_PREFIX.len()..];
    Ok(key.to_vec())
}

pub fn check_delegation(
    root_key: Vec<u8>,
    delegation: &Option<Delegation>,
) -> Result<Vec<u8>, String> {
    match delegation {
        None => Ok(root_key),
        Some(delegation) => {
            let cert: Certificate = serde_cbor::from_slice(&delegation.certificate)
                .map_err(|_| "can not obtain certificate from delegation".to_owned())?;

            verify(root_key, &cert)?;
            let public_key_path = [
                "subnet".into(),
                delegation.subnet_id.clone().into(),
                "public_key".into(),
            ];
            lookup_value(&cert.tree, public_key_path).map(|pk| pk.to_vec())
        }
    }
}

pub fn lookup_value<'a, P>(tree: &'a HashTree<'a>, path: P) -> Result<&'a [u8], String>
where
    for<'p> &'p P: IntoIterator<Item = &'p Label>,
    P: Into<Vec<Label>>,
{
    match tree.lookup_path(&path) {
        LookupResult::Found(value) => Ok(value),
        _ => Err(format!("Can not lookup {:?}", path.into())),
    }
}
