/**7
 * Fungible Token implementation with JSON serialization.
 * NOTES:
 *  - The maximum balance value is limited by U128 (2**128 - 1).
 *  - JSON calls should pass U128 as a base-10 string. E.g. "100".
 *  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
 *    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
 *  - This contract doesn't optimize the amount of storage, since any account can create unlimited
 *    amount of allowances to other accounts. It's unclear how to address this issue unless, this
 *    contract limits the total number of different allowances possible at the same time.
 *    And even if it limits the total number, it's still possible to transfer small amounts to
 *    multiple accounts.
 */
mod token;
mod wallet;
mod account;
mod request;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, Vector};
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, Balance, Promise};
use crate::account::TokenAccount;
use crate::token::{Token, TokenId};
use crate::wallet::TokenWallet;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct TokenBalance{
    token_id: TokenId,
    balance: Balance
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// sha256(AccountID) -> Account details.
    pub wallets: LookupMap<Vec<u8>, TokenWallet>,
    /// разрешенные токены
    pub tokens: Vector<Token>,
    /// владелец контракта
    owner_id: AccountId,
}

impl Default for Contract {
    fn default() -> Self {
        panic!("Contract should be initialized before usage")
    }
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given total supply owned by the given `owner_id`.
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        assert!(env::state_read::<Self>().is_none(), "Already initialized");
        let mut contract = Self {
            wallets: LookupMap::new(b"w".to_vec()),
            tokens: Vector::new(b"t".to_vec()),
            owner_id:owner_id.clone(),
        };
        contract.add_token(Token{
            token_id: "XDHO".to_string(),
            owner_id: owner_id.clone(),
            supply: 100_000_000_000
        });
        contract
    }

    /// Sets the `allowance` for `escrow_account_id` on the account of the caller of this contract
    /// (`predecessor_id`) who is the balance owner.
    pub fn set_allowance(&mut self, escrow_account_id: AccountId, token_id: Option<TokenId>, allowance: U128) {
        let allowance = allowance.into();
        let owner_id = env::predecessor_account_id();
        if escrow_account_id == owner_id {
            env::panic(b"Can't set allowance for yourself");
        }
        let mut account = self.get_account(&owner_id, &token_id);
        account.set_allowance(&escrow_account_id, allowance);
        self.set_account(&owner_id, &account, &token_id);
    }

    /// Transfers the `amount` of tokens from `owner_id` to the `new_owner_id`.
    /// Requirements:
    /// * `amount` should be a positive integer.
    /// * `owner_id` should have balance on the account greater or equal than the transfer `amount`.
    /// * If this function is called by an escrow account (`owner_id != predecessor_account_id`),
    ///   then the allowance of the caller of the function (`predecessor_account_id`) on
    ///   the account of `owner_id` should be greater or equal than the transfer `amount`.
    pub fn transfer_from(&mut self, owner_id: AccountId, new_owner_id: AccountId, token_id: Option<TokenId>, amount: U128) {
        let amount = amount.into();
        if amount == 0 {
            env::panic(b"Can't transfer 0 tokens");
        }
        // Retrieving the account from the state.
        let mut account = self.get_account(&owner_id, &token_id);

        // Checking and updating unlocked balance
        if account.balance < amount {
            env::panic(b"Not enough balance");
        }
        account.balance -= amount;

        // If transferring by escrow, need to check and update allowance.
        let escrow_account_id = env::predecessor_account_id();
        if escrow_account_id != owner_id {
            let allowance = account.get_allowance(&escrow_account_id);
            if allowance < amount {
                env::panic(b"Not enough allowance");
            }
            account.set_allowance(&escrow_account_id, allowance - amount);
        }

        // Saving the account back to the state.
        self.set_account(&owner_id, &account, &token_id);

        // Deposit amount to the new owner and save the new account to the state.
        let mut new_account = self.get_account(&new_owner_id, &token_id);
        new_account.balance += amount;
        self.set_account(&new_owner_id, &new_account, &token_id);
    }

    /// Transfer `amount` of tokens from the caller of the contract (`predecessor_id`) to
    /// `new_owner_id`.
    /// Act the same was as `transfer_from` with `owner_id` equal to the caller of the contract
    /// (`predecessor_id`).
    pub fn transfer(&mut self, new_owner_id: AccountId, token_id: Option<TokenId>, amount: U128) {
        self.transfer_from(env::predecessor_account_id(), new_owner_id, token_id, amount);
    }

    /// Returns total supply of tokens.
    pub fn get_total_supply(&self) -> Balance {
        100_000_000_000
    }

    /// Returns balance of the `owner_id` account.
    pub fn get_balance(&self, owner_id: AccountId, token_id: Option<TokenId>) -> U128 {
        self.get_account(&owner_id, &token_id).balance.into()
    }

    /// Returns balance of the `owner_id` account.
    pub fn get_balances(&self, owner_id: AccountId) -> Vector<TokenBalance>{

        self.get
    }

    /// Returns current allowance of `escrow_account_id` for the account of `owner_id`.
    ///
    /// NOTE: Other contracts should not rely on this information, because by the moment a contract
    /// receives this information, the allowance may already be changed by the owner.
    /// So this method should only be used on the front-end to see the current allowance.
    pub fn get_allowance(&self, owner_id: AccountId, escrow_account_id: AccountId, token_id: Option<TokenId>) -> U128 {
        self.get_account(&owner_id, &token_id)
            .get_allowance(&escrow_account_id)
            .into()
    }
}

#[near_bindgen]
impl Contract{
    #[payable]
    pub fn pay_standard_token(&mut self, amount: U128, to: AccountId) -> Promise{
        self.transfer_from(self.owner_id.clone(),
                           to.clone(),
                           Option::Some("XDHO".to_string()),
                           U128::from(self.get_standard_price() * amount.0));
        Promise::new(to).transfer(amount.0)
    }

    pub fn get_standard_price(&self) -> Balance{
        52u128 // монет за один NEAR
    }
}

#[near_bindgen]
impl Contract {
    pub fn get_tokens(&self) -> Vec<Token>{
        self.tokens.to_vec()
    }

    #[private]
    pub fn add_token(&mut self, token: Token){
        self.tokens.push(&token);
        let token_id = token.token_id;
        let owner_id = &token.owner_id;
        let mut account = self.get_account(owner_id, &Option::Some(token_id.clone()));
        account.balance = token.supply;
        self.set_account(&token.owner_id, &account, &Option::Some(token_id));
    }

    fn get_wallet(&self, owner_id: &AccountId) -> TokenWallet{
        let account_hash = env::sha256(owner_id.as_bytes());
        self.wallets.get(&account_hash)
            .unwrap_or(TokenWallet::new(account_hash))
    }

    /// Helper method to get the account details for `owner_id`.
    fn get_account(&self, owner_id: &AccountId, token_id: &Option<TokenId>) -> TokenAccount {
        self.get_wallet(owner_id)
            .get_account(owner_id, token_id.clone().unwrap_or("XDHO".to_string()))
    }

    /// Helper method to set the account details for `owner_id` to the state.
    fn set_account(&mut self, owner_id: &AccountId, account: &TokenAccount, token_id: &Option<TokenId>) {
        self.get_wallet(owner_id)
            .set_account(&token_id.clone().unwrap_or("XDHO".to_string()), &account)
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use near_sdk::{Gas, MockedBlockchain};
    use near_sdk::{testing_env, VMContext};

    use super::*;
    fn standart_token() -> Token{
        Token{
            token_id: "XDHO".to_string(),
            owner_id: "XDHO".to_string(),
            supply: 100_000_000_000
        }
    }
    fn alice() -> AccountId {
        "alice.near".to_string()
    }
    fn bob() -> AccountId {
        "bob.near".to_string()
    }
    fn carol() -> AccountId {
        "carol.near".to_string()
    }

    fn catch_unwind_silent<F: FnOnce() -> R + std::panic::UnwindSafe, R>(
        f: F,
    ) -> std::thread::Result<R> {
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let result = std::panic::catch_unwind(f);
        std::panic::set_hook(prev_hook);
        result
    }

    fn get_context(predecessor_account_id: AccountId) -> VMContext {
        VMContext {
            current_account_id: alice(),
            signer_account_id: bob(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id,
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 100,
            account_locked_balance: 0,
            storage_usage: 10u64.pow(6),
            attached_deposit: 0,
            prepaid_gas: Gas::from(10u64.pow(18)),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 0,
        }
    }

    #[test]
    fn test_new() {
        let context = get_context(carol());
        testing_env!(context);

        let contract = Contract::new(bob(), );
        assert_eq!(contract.get_balance(bob(), None).0, standart_token().supply);
    }

    #[test]
    fn test_transfer() {
        let context = get_context(carol());
        testing_env!(context);
        let total_supply = standart_token().supply;
        let mut contract = Contract::new(carol());
        let transfer_amount = total_supply / 3;
        contract.transfer(bob(), None,transfer_amount.into());
        assert_eq!(
            contract.get_balance(carol(), None).0,
            (total_supply - transfer_amount)
        );
        assert_eq!(contract.get_balance(bob(), None).0, transfer_amount);
    }

    #[test]
    fn test_pay() {
        let context = get_context(carol());
        testing_env!(context);
        let mut contract = Contract::new(carol());

        contract.pay_standard_token(U128(3), bob());
        assert_eq!(env::account_balance(), 97);
        let balance = contract.get_balance(bob(), Option::Some("XDHO".to_string())).0;
        assert_eq!(balance, 156u128);
    }


    #[test]
    fn test_self_allowance_fail() {
        let context = get_context(carol());
        testing_env!(context);
        let total_supply = standart_token().supply;
        let mut contract = Contract::new(carol());
        catch_unwind_silent(move || {
            contract.set_allowance(carol(), None,(total_supply / 2).into());
        })
            .unwrap_err();
    }

    #[test]
    fn test_carol_add_new_token() {
        // Acting as carol
        testing_env!(get_context(carol()));
        let total_supply = standart_token().supply;
        let mut contract = Contract::new(carol());

        assert_eq!(contract.get_total_supply(), total_supply);
        contract.add_token(Token{
            token_id: "TEST".to_string(),
            supply: 10000,
            owner_id: carol()
        });
        let balance = contract.get_balance(carol(), Option::Some("TEST".to_string())).0;
        assert_eq!(balance, 10000u128);

        let allowance = total_supply / 3;
        let transfer_amount = allowance / 3;
        contract.set_allowance(bob(), None, allowance.into());
        assert_eq!(contract.get_allowance(carol(), bob(),  None).0, allowance);
        // Acting as bob now
        testing_env!(get_context(bob()));
        contract.transfer_from(carol(), alice(),  None,transfer_amount.into());
        assert_eq!(
            contract.get_balance(carol(),  None).0,
            total_supply - transfer_amount
        );
        assert_eq!(contract.get_balance(alice(),  None).0, transfer_amount);
        assert_eq!(
            contract.get_allowance(carol(), bob(),  None).0,
            allowance - transfer_amount
        );
    }

    #[test]
    fn test_carol_escrows_to_bob_transfers_to_alice() {
        // Acting as carol
        testing_env!(get_context(carol()));
        let total_supply = standart_token().supply;
        let mut contract = Contract::new(carol());
        assert_eq!(contract.get_total_supply(), total_supply);
        let allowance = total_supply / 3;
        let transfer_amount = allowance / 3;
        contract.set_allowance(bob(), None, allowance.into());
        assert_eq!(contract.get_allowance(carol(), bob(),  None).0, allowance);
        // Acting as bob now
        testing_env!(get_context(bob()));
        contract.transfer_from(carol(), alice(),  None,transfer_amount.into());
        assert_eq!(
            contract.get_balance(carol(),  None).0,
            total_supply - transfer_amount
        );
        assert_eq!(contract.get_balance(alice(),  None).0, transfer_amount);
        assert_eq!(
            contract.get_allowance(carol(), bob(),  None).0,
            allowance - transfer_amount
        );
    }
}
