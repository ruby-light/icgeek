use crate::types::{AddTentativeDeviceResponse, DeviceData, UserNumber};
use candid::Principal;

pub async fn call_add_tentative_device(
    identity_canister: Principal,
    user_number: UserNumber,
    device_data: DeviceData,
) -> Result<AddTentativeDeviceResponse, String> {
    let result: ic_cdk::api::call::CallResult<(AddTentativeDeviceResponse,)> = ic_cdk::call(
        identity_canister,
        "add_tentative_device",
        (user_number, device_data),
    )
    .await;

    result
        .map(|r| r.0)
        .map_err(|e| format!("Failed to call add_tentative_device {:?}", e))
}
