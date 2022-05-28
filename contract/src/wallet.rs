use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{AccountId, env};
use near_sdk::collections::LookupMap;
use crate::account::TokenAccount;
use crate::token::TokenId;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenWallet{
    /// хранит для каждого токена его баланс
    pub accounts: LookupMap<TokenId, TokenAccount>,
}

impl TokenWallet{
    /// Initializes a new Account with 0 balance and no allowances for a given `account_hash`.
    pub fn new(account_hash: Vec<u8>) -> Self {
        Self {
            accounts: LookupMap::new(account_hash)
        }
    }

    /// Helper method to get the account details for `owner_id`.
    pub fn get_account(&self, owner_id: &AccountId, token_id: TokenId) -> TokenAccount {
        let account_hash = env::sha256(owner_id.as_bytes());
        self.accounts
            .get(&token_id)
            .unwrap_or(TokenAccount::new(account_hash))
    }

    /// Helper method to set the account details for `owner_id` to the state.
    pub fn set_account(&mut self, token_id: &TokenId, account: &TokenAccount) {
        self.accounts.insert(token_id, account);
    }
}
