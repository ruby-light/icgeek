use crate::types::{IcpXdrConversionRateCertifiedResponse, NotifyTopUpArg, NotifyTopUpResult};
use candid::Principal;

pub async fn notify_cycles_minting(
    minting_canister: Principal,
    canister_id: Principal,
    block_index: u64,
) -> Result<NotifyTopUpResult, String> {
    let args = NotifyTopUpArg {
        block_index,
        canister_id,
    };

    let result: ic_cdk::api::call::CallResult<(NotifyTopUpResult,)> =
        ic_cdk::call(minting_canister, "notify_top_up", (args,)).await;

    result
        .map(|r| r.0)
        .map_err(|e| format!("Failed to call notify_cycles_minting {:?}", e))
}

pub async fn get_average_icp_xdr_rate(
    minting_canister: Principal,
) -> Result<IcpXdrConversionRateCertifiedResponse, String> {
    let result: ic_cdk::api::call::CallResult<(IcpXdrConversionRateCertifiedResponse,)> =
        ic_cdk::call(
            minting_canister,
            "get_average_icp_xdr_conversion_rate",
            ((),),
        )
        .await;

    result
        .map(|r| r.0)
        .map_err(|e| format!("Failed to call get_average_icp_xdr_rate {:?}", e))
}

pub async fn get_icp_xdr_rate(
    minting_canister: Principal,
) -> Result<IcpXdrConversionRateCertifiedResponse, String> {
    let result: ic_cdk::api::call::CallResult<(IcpXdrConversionRateCertifiedResponse,)> =
        ic_cdk::call(minting_canister, "get_icp_xdr_conversion_rate", ((),)).await;

    result
        .map(|r| r.0)
        .map_err(|e| format!("Failed to call get_average_icp_xdr_rate {:?}", e))
}
