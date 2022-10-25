use crate::types::{DeviceData, UserNumber};
use candid::Principal;

pub async fn query_lookup_devices(
    identity_canister: Principal,
    user_number: UserNumber,
) -> Result<Vec<DeviceData>, String> {
    let result: ic_cdk::api::call::CallResult<(Vec<DeviceData>,)> =
        ic_cdk::call(identity_canister, "lookup", (user_number,)).await;

    result
        .map(|r| r.0)
        .map_err(|e| format!("Failed to call lookup {:?}", e))
}
