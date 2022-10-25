use crate::types::{EcdsaDerivationPath, EcdsaKeyId, EcdsaMessageHash, EcdsaSignatureCompact};
use candid::Principal;

pub async fn perform_sign_with_ecdsa(
    key_id: EcdsaKeyId,
    derivation_path: EcdsaDerivationPath,
    message_hash: EcdsaMessageHash,
    count: u128,
) -> Result<EcdsaSignatureCompact, String> {
    let args = crate::types::SignWithECDSA {
        message_hash,
        derivation_path,
        key_id,
    };

    let result: ic_cdk::api::call::CallResult<(crate::types::SignWithECDSAReply,)> =
        ic_cdk::api::call::call_with_payment(
            Principal::management_canister(),
            "sign_with_ecdsa",
            (args,),
            count.try_into().unwrap(),
        )
        .await;

    result
        .map(|r| r.0.signature)
        .map_err(|e| format!("Failed to call sign_with_ecdsa {:?}", e))
}
