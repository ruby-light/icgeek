use crate::types::{EcdsaDerivationPath, EcdsaKeyCompact, EcdsaKeyId};
use candid::Principal;

pub async fn get_ecdsa_public_key(
    key_id: EcdsaKeyId,
    derivation_path: EcdsaDerivationPath,
) -> Result<EcdsaKeyCompact, String> {
    let args = crate::types::ECDSAPublicKey {
        canister_id: None,
        derivation_path,
        key_id,
    };

    let result: ic_cdk::api::call::CallResult<(crate::types::ECDSAPublicKeyReply,)> = ic_cdk::call(
        Principal::management_canister(),
        "ecdsa_public_key",
        (args,),
    )
    .await;

    result
        .map(|r| r.0.public_key)
        .map_err(|e| format!("Failed to call ecdsa_public_key {:?}", e))
}
