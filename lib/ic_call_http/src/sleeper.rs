use candid::Principal;
use std::time::Duration;

pub async fn sleep(duration: Duration) -> u16 {
    let start = get_current_time();
    let mut try_count = 1;
    loop {
        try_count += 1;

        sleep_random().await;

        if start + duration.as_nanos() < get_current_time() || try_count > 100 {
            return try_count;
        }
    }
}

fn get_current_time() -> u128 {
    #[cfg(target_arch = "wasm32")]
    {
        ic_cdk::api::time().into()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time wrapped around.")
            .as_nanos()
    }
}

async fn sleep_random() {
    ic_cdk::call(Principal::management_canister(), "raw_rand", ())
        .await
        .expect("Can not sleep over raw rand")
}
