use crate::public_key::public_key_to_asn1_block;
use crate::render::RandGenerator;
use crate::request_id::{to_request_id, RequestId};
use crate::sha256::get_sha256;
use crate::signer::{EcdsaSignatureCompact, MessageHash, Signer};
use crate::types::{
    CallRequestContent, DeviceKey, Envelope, IngressExpiryDatetimeNanos, QueryContent,
    ReadStateContent, SignedDelegation,
};
use candid::Principal;
use ic_types::hash_tree::Label;
use icgeek_ic_call_api::{AgentCallRequest, AgentQueryRequest};
use serde::Serialize;
use std::time::Duration;

pub async fn create_query_request(
    signer: &dyn Signer,
    canister_id: &Principal,
    method_name: &str,
    arg: Vec<u8>,
    ingress_expiry: IngressExpiryDatetimeNanos,
    delegation: Option<(DeviceKey, SignedDelegation)>,
) -> Result<AgentQueryRequest, String> {
    let (public_key, signed_delegation) = detect_public_key_and_delegations(signer, delegation);

    let sender = Principal::self_authenticating(&public_key);

    // query request sign

    let request =
        build_query_request(&sender, canister_id, method_name, arg, ingress_expiry).await?;

    let request_id = to_request_id(&request).map_err(|e| std::format!("{:?}", e))?;

    let message_hash = construct_sign_message_hash(&request_id);
    let sign_result = signer.sign(&message_hash).await?;
    let request_sign = serialize_envelope(
        public_key.clone(),
        signed_delegation.clone(),
        sign_result,
        &request,
    )?;

    Ok(AgentQueryRequest {
        canister_id: *canister_id,
        request_sign,
    })
}

pub async fn create_call_request(
    rand_generator: &dyn RandGenerator,
    signer: &dyn Signer,
    canister_id: &Principal,
    method_name: &str,
    arg: Vec<u8>,
    ingress_expiry: IngressExpiryDatetimeNanos,
    delegation: Option<(DeviceKey, SignedDelegation)>,
) -> Result<AgentCallRequest, String> {
    let (public_key, signed_delegation) = detect_public_key_and_delegations(signer, delegation);

    let sender = Principal::self_authenticating(&public_key);

    // call request sign

    let request = build_call_request(
        rand_generator,
        &sender,
        canister_id,
        method_name,
        arg,
        ingress_expiry,
    )
    .await?;

    let request_id = to_request_id(&request).map_err(|e| std::format!("{:?}", e))?;

    let message_hash = construct_sign_message_hash(&request_id);
    let sign_result = signer.sign(&message_hash).await?;
    let request_sign = serialize_envelope(
        public_key.clone(),
        signed_delegation.clone(),
        sign_result,
        &request,
    )?;

    // read state request sign

    let rs_request = build_read_state_request(sender, &request_id, ingress_expiry);

    let rs_request_id = to_request_id(&rs_request).map_err(|e| std::format!("{:?}", e))?;

    let rs_message_hash = construct_sign_message_hash(&rs_request_id);
    let rs_sign_result = signer.sign(&rs_message_hash).await?;
    let read_state_request_sign =
        serialize_envelope(public_key, signed_delegation, rs_sign_result, &rs_request)?;

    Ok(AgentCallRequest {
        canister_id: *canister_id,
        request_id: request_id.as_slice().to_vec(),
        request_sign,
        read_state_request_sign,
    })
}

fn detect_public_key_and_delegations(
    signer: &dyn Signer,
    delegation: Option<(DeviceKey, SignedDelegation)>,
) -> (DeviceKey, Option<Vec<SignedDelegation>>) {
    delegation
        .map(|(k, d)| (k, Some(vec![d])))
        .unwrap_or_else(|| {
            (
                public_key_to_asn1_block(signer.get_uncompressed_public_key().as_slice()),
                None,
            )
        })
}

pub fn get_ingress_expiry_datetime_nanos(
    unix_epoch_time_nanos: u128,
) -> IngressExpiryDatetimeNanos {
    let permitted_drift = Duration::from_secs(60);
    let ingress_expiry_duration = Duration::from_secs(300);

    (ingress_expiry_duration
        .as_nanos()
        .saturating_add(unix_epoch_time_nanos)
        .saturating_sub(permitted_drift.as_nanos())) as u64
}

// PRIVATE

async fn build_query_request(
    sender: &Principal,
    canister_id: &Principal,
    method_name: &str,
    arg: Vec<u8>,
    ingress_expiry: IngressExpiryDatetimeNanos,
) -> Result<QueryContent, String> {
    Ok(QueryContent::QueryRequest {
        canister_id: *canister_id,
        method_name: method_name.into(),
        arg,
        sender: *sender,
        ingress_expiry,
    })
}

async fn build_call_request(
    rand_generator: &dyn RandGenerator,
    sender: &Principal,
    canister_id: &Principal,
    method_name: &str,
    arg: Vec<u8>,
    ingress_expiry: IngressExpiryDatetimeNanos,
) -> Result<CallRequestContent, String> {
    let nonce = rand_generator.generate_16().await?;

    Ok(CallRequestContent::CallRequest {
        canister_id: *canister_id,
        method_name: method_name.into(),
        arg,
        nonce: Some(nonce),
        sender: *sender,
        ingress_expiry,
    })
}

fn build_read_state_request(
    sender: Principal,
    request_id: &RequestId,
    ingress_expiry: IngressExpiryDatetimeNanos,
) -> ReadStateContent {
    let paths: Vec<Vec<Label>> = vec![vec!["request_status".into(), request_id.to_vec().into()]];

    ReadStateContent::ReadStateRequest {
        sender,
        paths,
        ingress_expiry,
    }
}
fn construct_sign_message_hash(request_id: &RequestId) -> MessageHash {
    get_sha256(construct_message(request_id))
}

fn construct_message(request_id: &RequestId) -> Vec<u8> {
    const IC_REQUEST_DOMAIN_SEPARATOR: &[u8; 11] = b"\x0Aic-request";

    let mut buf = vec![];
    buf.extend_from_slice(IC_REQUEST_DOMAIN_SEPARATOR);
    buf.extend_from_slice(request_id.as_slice());
    buf
}

fn serialize_envelope<'a, V>(
    asn1_public_key: Vec<u8>,
    signed_delegation: Option<Vec<SignedDelegation>>,
    signature: EcdsaSignatureCompact,
    request: &V,
) -> Result<Vec<u8>, String>
where
    V: 'a + Serialize,
{
    let envelope = Envelope {
        content: request,
        sender_pubkey: Some(asn1_public_key),
        sender_delegation: signed_delegation,
        sender_sig: Some(signature),
    };

    let mut serialized_bytes = Vec::new();
    let mut serializer = serde_cbor::Serializer::new(&mut serialized_bytes);

    serializer.self_describe().map_err(|e| e.to_string())?;
    envelope
        .serialize(&mut serializer)
        .map_err(|e| e.to_string())?;

    Ok(serialized_bytes)
}

#[cfg(test)]
mod tests {
    use crate::public_key::public_key_to_asn1_block;
    use crate::render::RandGenerator;
    use crate::request_id::to_request_id;
    use crate::signer::{EcdsaSignatureCompact, MessageHash, Signer, UncompressedPublicKey};
    use async_trait::async_trait;
    use candid::{Encode, Principal};
    use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};

    struct DevSigner {
        pub key: Vec<u8>,
    }

    #[async_trait]
    impl Signer for DevSigner {
        fn get_uncompressed_public_key(&self) -> UncompressedPublicKey {
            let secp = Secp256k1::new();
            let secret_key = SecretKey::from_slice(self.key.as_slice()).unwrap();
            PublicKey::from_secret_key(&secp, &secret_key).serialize_uncompressed()
        }

        async fn sign(&self, message_hash: &MessageHash) -> Result<EcdsaSignatureCompact, String> {
            let secp = Secp256k1::new();
            let secret_key = SecretKey::from_slice(self.key.as_slice()).unwrap();
            let message = Message::from_slice(message_hash).unwrap();
            let sig = secp.sign_ecdsa(&message, &secret_key);
            Ok(sig.serialize_compact().to_vec())
        }
    }

    pub struct DevRandGenerator;

    #[async_trait]
    impl RandGenerator for DevRandGenerator {
        async fn generate_16(&self) -> Result<Vec<u8>, String> {
            Ok([0; 16].to_vec())
        }

        async fn generate_32(&self) -> Result<Vec<u8>, String> {
            Ok([0; 32].to_vec())
        }
    }

    #[actix_rt::test]
    async fn test() {
        let ecdsa_key = vec![
            206, 178, 166, 57, 22, 65, 0, 87, 74, 174, 234, 96, 86, 212, 168, 78, 13, 132, 134,
            231, 51, 148, 2, 77, 42, 30, 223, 56, 16, 112, 34, 160,
        ];

        let signer = DevSigner { key: ecdsa_key };

        let public_key = signer.get_uncompressed_public_key();
        let asn1_public_key = public_key_to_asn1_block(public_key.as_slice());
        assert_eq!(
            asn1_public_key,
            vec![
                48, 86, 48, 16, 6, 7, 42, 134, 72, 206, 61, 2, 1, 6, 5, 43, 129, 4, 0, 10, 3, 66,
                0, 4, 5, 152, 51, 190, 30, 224, 35, 167, 57, 223, 222, 218, 186, 83, 126, 201, 62,
                122, 65, 209, 117, 183, 41, 170, 168, 69, 160, 29, 28, 82, 231, 64, 18, 68, 135,
                212, 180, 77, 130, 128, 9, 92, 52, 17, 82, 49, 118, 254, 131, 19, 172, 103, 133,
                128, 208, 241, 53, 166, 208, 179, 96, 25, 223, 107
            ]
        );
        let sender = Principal::self_authenticating(&asn1_public_key);

        let canister_id = Principal::from_text("r5m4o-xaaaa-aaaah-qbpfq-cai").unwrap();
        let method_name = "transfer";
        let arg = Encode!(&()).unwrap();

        let rand_generator = DevRandGenerator {};

        let call_request = super::build_call_request(
            &rand_generator,
            &sender,
            &canister_id,
            method_name,
            arg.clone(),
            0,
        )
        .await
        .unwrap();
        let request_id = to_request_id(&call_request).unwrap();
        let slice = [
            91, 75, 248, 199, 110, 203, 69, 241, 119, 49, 204, 87, 118, 182, 175, 64, 135, 81, 233,
            170, 100, 69, 109, 226, 178, 254, 25, 26, 46, 96, 246, 128,
        ];
        assert_eq!(request_id.as_slice(), slice.as_slice());

        // let sign_result = super::sign_request_id(&signer, &public_key, &request_id).await.unwrap();
        //
        let request_sign =
            super::serialize_envelope(asn1_public_key, None, [0_u8; 64].to_vec(), &call_request)
                .unwrap();
        assert_eq!(
            request_sign,
            vec![
                217, 217, 247, 163, 103, 99, 111, 110, 116, 101, 110, 116, 167, 108, 114, 101, 113,
                117, 101, 115, 116, 95, 116, 121, 112, 101, 100, 99, 97, 108, 108, 101, 110, 111,
                110, 99, 101, 80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 110, 105, 110,
                103, 114, 101, 115, 115, 95, 101, 120, 112, 105, 114, 121, 0, 102, 115, 101, 110,
                100, 101, 114, 88, 29, 254, 241, 36, 232, 6, 236, 209, 181, 208, 225, 14, 223, 195,
                0, 122, 233, 72, 95, 76, 81, 148, 48, 253, 99, 232, 103, 165, 55, 2, 107, 99, 97,
                110, 105, 115, 116, 101, 114, 95, 105, 100, 74, 0, 0, 0, 0, 0, 240, 11, 203, 1, 1,
                107, 109, 101, 116, 104, 111, 100, 95, 110, 97, 109, 101, 104, 116, 114, 97, 110,
                115, 102, 101, 114, 99, 97, 114, 103, 71, 68, 73, 68, 76, 0, 1, 127, 109, 115, 101,
                110, 100, 101, 114, 95, 112, 117, 98, 107, 101, 121, 88, 88, 48, 86, 48, 16, 6, 7,
                42, 134, 72, 206, 61, 2, 1, 6, 5, 43, 129, 4, 0, 10, 3, 66, 0, 4, 5, 152, 51, 190,
                30, 224, 35, 167, 57, 223, 222, 218, 186, 83, 126, 201, 62, 122, 65, 209, 117, 183,
                41, 170, 168, 69, 160, 29, 28, 82, 231, 64, 18, 68, 135, 212, 180, 77, 130, 128, 9,
                92, 52, 17, 82, 49, 118, 254, 131, 19, 172, 103, 133, 128, 208, 241, 53, 166, 208,
                179, 96, 25, 223, 107, 106, 115, 101, 110, 100, 101, 114, 95, 115, 105, 103, 88,
                64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
    }
}
