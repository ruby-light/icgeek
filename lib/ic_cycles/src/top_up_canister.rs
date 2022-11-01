use crate::types::{CanisterIdRecord, Cycles, PrincipalId};
use candid::Principal;

pub async fn top_up_canister(canister_id: Principal, count: Cycles) -> Result<(), String> {
    let args = CanisterIdRecord {
        canister_id: PrincipalId(canister_id),
    };

    let result: ic_cdk::api::call::CallResult<((),)> = ic_cdk::api::call::call_with_payment(
        Principal::management_canister(),
        "deposit_cycles",
        (args,),
        count.try_into().unwrap(),
    )
    .await;

    result
        .map(|r| r.0)
        .map_err(|e| format!("Failed to call top_up_canister {:?}", e))
}
