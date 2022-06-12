use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::{env, AccountId, Balance};

/// Contains balance and allowances information for one account.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenAccount {
    /// Текущий баланс токена
    pub balance: Balance,
    /// Информация о том,
    /// Сколько каждому конкретному кошельку
    /// дозволено снимать с этого счета
    pub allowances: LookupMap<Vec<u8>, Balance>,
}

impl TokenAccount {
    /// Initializes a new Account with 0 balance and no allowances for a given `account_hash`.
    pub fn new(account_hash: Vec<u8>) -> Self {
        Self {
            balance: 0,
            allowances: LookupMap::new(account_hash),
        }
    }

    /// Sets allowance for account `escrow_account_id` to `allowance`.
    pub fn set_allowance(&mut self, escrow_account_id: &AccountId, allowance: Balance) {
        let escrow_hash = env::sha256(escrow_account_id.as_bytes());
        if allowance > 0 {
            self.allowances.insert(&escrow_hash, &allowance);
        } else {
            self.allowances.remove(&escrow_hash);
        }
    }

    /// Returns the allowance of account `escrow_account_id`.
    pub fn get_allowance(&self, escrow_account_id: &AccountId) -> Balance {
        let escrow_hash = env::sha256(escrow_account_id.as_bytes());
        self.allowances.get(&escrow_hash).unwrap_or(0)
    }
}
