use crate::account::TokenAccount;
use crate::token::TokenId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap};
use near_sdk::{Balance};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenWallet {
    /// хранит для каждого токена его баланс
    pub accounts: LookupMap<TokenId, TokenAccount>,
}

impl TokenWallet {
    /// Initializes a new Account with 0 balance and no allowances for a given `account_hash`.
    pub fn new(account_hash: Vec<u8>) -> Self {
        Self {
            accounts: LookupMap::new(account_hash),
        }
    }

    pub fn get_balances(&self, token_ids: Vec<TokenId>) -> Vec<(TokenId, Balance)> {
        let mut array: Vec<(TokenId, Balance)> = Vec::new();
        for token in token_ids{
            let balance = match self.get_account(&token) {
                Some(b) => b.balance,
                None => 0u128
            };
            array.push((token, balance));
        }
        array
    }

    /// Helper method to get the account details for `owner_id`.
    pub fn get_account(&self, token_id: &TokenId) -> Option<TokenAccount> {
        self.accounts.get(&token_id)
    }

    /// Helper method to set the account details for `owner_id` to the state.
    pub fn set_account(&mut self, token_id: &TokenId, account: &TokenAccount) {
        self.accounts.insert(token_id, account);
    }
}
