use std::collections::BTreeMap;

pub type LedgerStorage = BTreeMap<String, u64>;

pub type PreUpgradeStableData<'a> = (&'a u8, &'a LedgerStorage);
pub type PostUpgradeStableData = (u8, LedgerStorage);

const VERSION: u8 = 1;

static mut STORAGE: Option<LedgerStorage> = None;

fn get_storage<'a>() -> &'a mut LedgerStorage {
    unsafe {
        if let Some(s) = &mut STORAGE {
            s
        } else {
            STORAGE = Some(LedgerStorage::default());
            get_storage()
        }
    }
}

// API

pub fn pre_upgrade_stable_data<'a>() -> PreUpgradeStableData<'a> {
    (&VERSION, get_storage())
}

pub fn post_upgrade_stable_data(data: PostUpgradeStableData) {
    match data {
        (VERSION, storage) => unsafe {
            STORAGE = Some(storage);
        },
        _ => {
            panic!("Can not upgrade dev ledger storage.");
        }
    }
}

pub fn get_account_balance(account: String) -> u64 {
    match get_storage().get(&account) {
        None => 0,
        Some(tokens) => *tokens,
    }
}

pub fn deposit_account(account: String, tokens: u64) {
    let storage = get_storage();
    let tokens = match storage.get(&account) {
        None => tokens,
        Some(balance_tokens) => tokens + *balance_tokens,
    };

    storage.insert(account, tokens);
}

pub fn withdraw_account(account: String, tokens: u64) -> Result<u64, u64> {
    let storage = get_storage();

    let balance_tokens: u64 = match storage.get(&account) {
        None => 0,
        Some(balance_tokens) => *balance_tokens,
    };

    if balance_tokens < tokens {
        Err(balance_tokens)
    } else {
        let new_tokens = balance_tokens - tokens;
        storage.insert(account, new_tokens);
        Ok(new_tokens)
    }
}

pub fn visit_accounts<F>(visitor: F)
where
    F: Fn(&String, &u64),
{
    for (account, balance) in get_storage().iter() {
        visitor(account, balance);
    }
}

#[cfg(test)]
mod tests {
    use crate::{deposit_account, get_account_balance, withdraw_account};

    #[test]
    fn test() {
        assert_eq!(
            0,
            get_account_balance("contract_transfer_account".to_owned())
        );
        deposit_account("contract_transfer_account".to_owned(), 20_000);

        assert_eq!(
            20_000,
            get_account_balance("contract_transfer_account".to_owned())
        );

        assert_eq!(
            Err(20_000),
            withdraw_account("contract_transfer_account".to_owned(), 20_001)
        );

        assert_eq!(
            Ok(2_000),
            withdraw_account("contract_transfer_account".to_owned(), 18_000)
        );

        assert_eq!(
            2_000,
            get_account_balance("contract_transfer_account".to_owned())
        );
    }
}
