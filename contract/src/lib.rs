mod account;
mod request;

mod ballot;
mod token;
mod wallet;

use crate::account::TokenAccount;
use crate::token::{Token, TokenId};
use crate::wallet::TokenWallet;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, Vector};
use near_sdk::env::panic;
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, Balance, Promise};
use orderbook::{orders, Failed, OrderIndex, OrderSide, Orderbook, Success};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

//const SINGLE_CALL_GAS: u64 = 20_000_000_000_000; // 2 x 10^14
//const TRANSFER_FROM_NEAR_COST: u128 = 36_500_000_000_000_000_000_000; // 365 x 10^20

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// sha256(AccountID) -> Account details.
    pub wallets: LookupMap<Vec<u8>, TokenWallet>,

    pub order_books: LookupMap<TokenId, Orderbook>,
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
            order_books: LookupMap::new(b"o".to_vec()),
            wallets: LookupMap::new(b"w".to_vec()),
            tokens: Vector::new(b"t".to_vec()),
            owner_id: owner_id.clone(),
        };
        contract.add_token(Token {
            token_id: "XDHO".to_string(),
            owner_id: owner_id.clone(),
            supply: 100_000_000_000,
        });
        contract
    }

    /// Sets the `allowance` for `escrow_account_id` on the account of the caller of this contract
    /// (`predecessor_id`) who is the balance owner.
    pub fn set_allowance(
        &mut self,
        escrow_account_id: AccountId,
        token_id: Option<TokenId>,
        allowance: U128,
    ) {
        let allowance = allowance.into();
        let owner_id = env::predecessor_account_id();
        if escrow_account_id == owner_id {
            env::panic(b"Can't set allowance for yourself");
        }
        let mut account = self.get_account(&owner_id, token_id.clone());
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
    pub fn transfer_from(
        &mut self,
        owner_id: AccountId,
        new_owner_id: AccountId,
        token_id: Option<TokenId>,
        amount: U128,
    ) {
        println!(
            "Перепожу {} {} от {} к {}",
            amount.0,
            token_id.clone().unwrap_or(self.get_standard_token()),
            owner_id,
            new_owner_id
        );
        let amount = amount.into();
        if amount == 0 {
            env::panic(b"Can't transfer 0 tokens");
        }
        // Retrieving the account from the state.
        let mut account = self.get_account(&owner_id, token_id.clone());

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
        let mut new_account = self.get_account(&new_owner_id, token_id.clone());
        new_account.balance += amount;
        self.set_account(&new_owner_id, &new_account, &token_id);
    }

    /// Transfer `amount` of tokens from the caller of the contract (`predecessor_id`) to
    /// `new_owner_id`.
    /// Act the same was as `transfer_from` with `owner_id` equal to the caller of the contract
    /// (`predecessor_id`).
    pub fn transfer(&mut self, new_owner_id: AccountId, token_id: Option<TokenId>, amount: U128) {
        self.transfer_from(
            env::predecessor_account_id(),
            new_owner_id,
            token_id,
            amount,
        );
    }

    /// Returns total supply of tokens.
    pub fn get_total_supply(&self) -> Balance {
        100_000_000_000
    }

    /// Returns balance of the `owner_id` account.
    pub fn get_balance(&self, owner_id: AccountId, token_id: Option<TokenId>) -> U128 {
        self.get_account(&owner_id, token_id).balance.into()
    }

    /// Returns balance of the `owner_id` account.
    pub fn get_balances(
        &self,
        owner_id: AccountId,
        token_ids: Vec<TokenId>,
    ) -> Vec<(TokenId, Balance)> {
        self.get_wallet(&owner_id).get_balances(token_ids)
    }

    /// Returns current allowance of `escrow_account_id` for the account of `owner_id`.
    ///
    /// NOTE: Other contracts should not rely on this information, because by the moment a contract
    /// receives this information, the allowance may already be changed by the owner.
    /// So this method should only be used on the front-end to see the current allowance.
    pub fn get_allowance(
        &self,
        owner_id: AccountId,
        escrow_account_id: AccountId,
        token_id: Option<TokenId>,
    ) -> U128 {
        self.get_account(&owner_id, token_id)
            .get_allowance(&escrow_account_id)
            .into()
    }
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn pay_standard_token(&mut self, amount: U128, to: AccountId) -> Promise {
        self.transfer_from(
            self.owner_id.clone(),
            to.clone(),
            Option::Some(self.get_standard_token()),
            U128::from(self.get_standard_price() * amount.0),
        );
        Promise::new(to).transfer(amount.0)
    }

    pub fn get_standard_price(&self) -> Balance {
        52u128 // монет за один NEAR
    }
}

#[near_bindgen]
impl Contract {
    pub fn get_tokens(&self) -> Vec<Token> {
        self.tokens.to_vec()
    }

    #[private]
    pub fn add_token(&mut self, token: Token) {
        self.tokens.push(&token);
        let token_id = token.token_id;
        let owner_id = &token.owner_id;
        let mut account = self.get_account(owner_id, Option::Some(token_id.clone()));
        account.balance = token.supply;
        self.set_account(&token.owner_id, &account, &Option::Some(token_id.clone()));
        let orderbook = Orderbook::new(token_id.clone(), self.get_standard_token());
        self.order_books.insert(&token_id.clone(), &orderbook);
    }

    fn get_standard_token(&self) -> String {
        "XDHO".to_string()
    }

    fn get_wallet(&self, owner_id: &AccountId) -> TokenWallet {
        let account_hash = env::sha256(owner_id.as_bytes());
        self.wallets
            .get(&account_hash)
            .unwrap_or(TokenWallet::new(account_hash))
    }

    /// Helper method to get the account details for `owner_id`.
    fn get_account(&self, owner_id: &AccountId, token_id: Option<TokenId>) -> TokenAccount {
        let token = token_id.unwrap_or(self.get_standard_token());
        self.get_wallet(owner_id)
            .get_account(&token)
            .unwrap_or(TokenAccount::new(env::sha256(owner_id.as_bytes())))
    }

    /// Helper method to set the account details for `owner_id` to the state.
    fn set_account(
        &mut self,
        owner_id: &AccountId,
        account: &TokenAccount,
        token_id: &Option<TokenId>,
    ) {
        self.get_wallet(owner_id).set_account(
            &token_id.clone().unwrap_or(self.get_standard_token()),
            &account,
        )
    }
}

fn parse_side(side: &str) -> Option<OrderSide> {
    match side {
        "Ask" => Some(OrderSide::Ask),
        "Bid" => Some(OrderSide::Bid),
        _ => panic(b"Side not parsed!"),
    }
}

fn get_current_time() -> u64 {
    return env::block_timestamp();
}

#[near_bindgen]
impl Contract {
    pub fn new_limit_order(&mut self, token_id: TokenId, price: f64, quantity: u128, side: String) {
        let token = match parse_side(side.as_str()).unwrap() {
            OrderSide::Ask => self.get_standard_token(),
            OrderSide::Bid => token_id.clone(),
        };
        println!(
            "Acc_id {}, signer: {}, token: {}",
            env::current_account_id(),
            env::predecessor_account_id(),
            token_id
        );
        self.transfer_from(
            env::predecessor_account_id(),
            env::current_account_id(),
            Option::Some(token),
            U128(quantity),
        );

        self.post_transfer(token_id, price, quantity, side)
    }

    pub fn cancel_limit_order(
        &mut self,
        token_id: TokenId,
        id: u64,
        side: String,
    ) -> Vec<Result<Success, Failed>> {
        let order = orders::limit_order_cancel_request(id, parse_side(&side).unwrap());
        let mut order_book = self.order_books.get(&token_id).unwrap();
        let res = order_book.process_order(order);
        self.order_books.insert(&token_id, &order_book);
        res
    }

    pub fn get_ask_orders(&self, token_id: TokenId) -> Vec<OrderIndex> {
        let order_book = self.order_books.get(&token_id).unwrap();
        order_book.ask_queue.clone().idx_queue.unwrap().into_vec()
    }

    pub fn get_bid_orders(&self, token_id: TokenId) -> Vec<OrderIndex> {
        let order_book = self.order_books.get(&token_id).unwrap();
        order_book.bid_queue.clone().idx_queue.unwrap().into_vec()
    }

    pub fn get_current_spread(&self, token_id: TokenId) -> Vec<f64> {
        let order_book = self.order_books.get(&token_id).unwrap();
        if let Some((bid, ask)) = order_book.clone().current_spread() {
            vec![ask, bid]
        } else {
            vec![0.0, 0.0]
        }
    }

    fn post_transfer(&mut self, token_id: TokenId, price: f64, quantity: u128, side: String) {
        env::log(b"Token Transfer Successful.");
        let order = orders::new_limit_order_request(
            token_id.clone(),
            self.get_standard_token(),
            parse_side(&side).unwrap(),
            price,
            quantity,
            env::signer_account_id(),
            get_current_time(),
        );

        let mut order_book = self.order_books.get(&token_id).unwrap();
        let res = order_book.process_order(order);
        self.order_books.insert(&token_id, &order_book);

        self.process_orderbook_result(token_id, res);
    }

    fn process_orderbook_result(
        &mut self,
        token_id: TokenId,
        order: Vec<Result<Success, Failed>>,
    ) -> Vec<Result<Success, Failed>> {
        for temp_variable in &order {
            let success = temp_variable.as_ref().unwrap();

            match success {
                Success::Accepted {
                    id: _,
                    order_type: _,
                    order_creator: _,
                    ts: _,
                } => {}
                Success::Filled {
                    order_id: _,
                    side,
                    order_type: _,
                    price: _,
                    qty,
                    order_creator,
                    ts: _,
                } => {
                    let reverse_side = match side {
                        OrderSide::Ask => OrderSide::Bid,
                        OrderSide::Bid => OrderSide::Ask,
                    };
                    self.transfer(
                        order_creator.to_string(),
                        Some(token_id.clone()),
                        U128::from(*qty),
                    );
                }
                Success::PartiallyFilled {
                    order_id: _,
                    side,
                    order_type: _,
                    price: _,
                    qty,
                    order_creator,
                    ts: _,
                } => {
                    let reverse_side = match side {
                        OrderSide::Ask => OrderSide::Bid,
                        OrderSide::Bid => OrderSide::Ask,
                    };

                    self.transfer(
                        order_creator.to_string(),
                        Some(token_id.clone()),
                        U128::from(*qty),
                    );
                }
                Success::Amended {
                    id: _,
                    price: _,
                    qty: _,
                    ts: _,
                } => {}
                Success::Cancelled { id: _, ts: _ } => {}
            };
        }
        order
    }

    fn _only_owner_predecessor(&mut self) {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Only contract owner can sign transactions for this method."
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use crate::{Contract, Token};
    use near_sdk::json_types::U128;
    use near_sdk::{env, AccountId, Gas, MockedBlockchain};
    use near_sdk::{testing_env, VMContext};
    use std::borrow::Borrow;

    use super::*;
    fn standart_token() -> Token {
        Token {
            token_id: "XDHO".to_string(),
            owner_id: carol().to_string(),
            supply: 100_000_000_000,
        }
    }

    fn test_token() -> Token {
        Token {
            token_id: "TEST".to_string(),
            owner_id: carol(),
            supply: 10000,
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

    fn init_contract_with_tokens() -> Contract {
        let context = get_context(carol());
        testing_env!(context);

        let mut contract = Contract::new(carol());
        assert_eq!(
            contract.get_balance(carol(), None).0,
            standart_token().supply
        );

        contract.add_token(test_token());
        let balance = contract
            .get_balance(carol(), Option::Some(test_token().token_id))
            .0;
        assert_eq!(balance, 10000u128);
        contract
    }

    #[test]
    fn test_new() {
        let context = get_context(carol());
        testing_env!(context);

        let contract = Contract::new(bob());
        assert_eq!(contract.get_balance(bob(), None).0, standart_token().supply);
    }

    #[test]
    fn test_transfer() {
        let context = get_context(carol());
        testing_env!(context);
        let total_supply = standart_token().supply;
        let mut contract = Contract::new(carol());
        let transfer_amount = total_supply / 3;
        contract.transfer(bob(), None, transfer_amount.into());
        assert_eq!(
            contract.get_balance(carol(), None).0,
            (total_supply - transfer_amount)
        );
        assert_eq!(contract.get_balance(bob(), None).0, transfer_amount);
    }

    #[test]
    fn test_wallets() {
        let context = get_context(carol());
        testing_env!(context);
        let mut contract = Contract::new(carol());

        contract.pay_standard_token(U128(3), bob());
        assert_eq!(env::account_balance(), 97);
        let balance = contract
            .get_balance(bob(), Option::Some(standart_token().token_id))
            .0;
        let account = contract
            .get_wallet(&bob())
            .get_account(&standart_token().token_id);

        assert_eq!(account.unwrap().balance, 156u128);
        assert_eq!(balance, 156u128);
    }

    #[test]
    fn test_pay() {
        let context = get_context(carol());
        testing_env!(context);
        let mut contract = Contract::new(carol());

        contract.pay_standard_token(U128(3), bob());
        assert_eq!(env::account_balance(), 97);
        let balance = contract
            .get_balance(bob(), Option::Some(standart_token().token_id))
            .0;
        assert_eq!(balance, 156u128);
    }

    #[test]
    fn test_self_allowance_fail() {
        let context = get_context(carol());
        testing_env!(context);
        let total_supply = standart_token().supply;
        let mut contract = Contract::new(carol());
        catch_unwind_silent(move || {
            contract.set_allowance(carol(), None, (total_supply / 2).into());
        })
        .unwrap_err();
    }

    #[test]
    fn test_carol_add_new_token() {
        // Acting as carol
        testing_env!(get_context(carol()));
        let total_supply = 10000;
        let mut contract = Contract::new(carol());

        contract.add_token(test_token());
        let balance = contract
            .get_balance(carol(), Option::Some(test_token().token_id))
            .0;
        assert_eq!(balance, 10000u128);

        let allowance = total_supply / 3;
        let transfer_amount = allowance / 3;
        contract.set_allowance(bob(), Some(test_token().token_id), allowance.into());
        assert_eq!(
            contract
                .get_allowance(carol(), bob(), Some(test_token().token_id))
                .0,
            allowance
        );
        // Acting as bob now
        testing_env!(get_context(bob()));
        contract.transfer_from(
            carol(),
            alice(),
            Some(test_token().token_id),
            transfer_amount.into(),
        );
        assert_eq!(
            contract.get_balance(carol(), Some(test_token().token_id)).0,
            total_supply - transfer_amount
        );
        assert_eq!(
            contract.get_balance(alice(), Some(test_token().token_id)).0,
            transfer_amount
        );
        assert_eq!(
            contract.get_allowance(carol(), bob(), None).0,
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
        assert_eq!(contract.get_allowance(carol(), bob(), None).0, allowance);
        // Acting as bob now
        testing_env!(get_context(bob()));
        contract.transfer_from(carol(), alice(), None, transfer_amount.into());
        assert_eq!(
            contract.get_balance(carol(), None).0,
            total_supply - transfer_amount
        );
        assert_eq!(contract.get_balance(alice(), None).0, transfer_amount);
        assert_eq!(
            contract.get_allowance(carol(), bob(), None).0,
            allowance - transfer_amount
        );
    }

    fn print_all_balances(contract: &Contract, account: AccountId) {
        for (token, balance) in contract.get_balances(
            account.clone(),
            [standart_token().token_id, test_token().token_id].to_vec(),
        ) {
            println!("Acc:{} token: {} balance: {}", account, token, balance);
        }
    }

    fn init_contract_with_tokens_and_limit_bids() -> Contract{
        let mut contract = init_contract_with_tokens();

        contract
    }

    #[test]
    fn get_ask_order() {
        testing_env!(get_context(carol()));
        //let total_supply = standart_token().supply;
        let mut contract = init_contract_with_tokens();

        // Currrent Spread
        let spread = contract.get_current_spread(test_token().token_id);
        println!("Spread => Ask: {}, Bid: {}", spread[0], spread[1]);
        assert_eq!(spread[0], 0.0);
        assert_eq!(spread[1], 0.0);

        // Ask Order
        let res = contract.new_limit_order(test_token().token_id, 1.25, 2, "Ask".to_string());

        // Bid Order
        let res2 = contract.new_limit_order(test_token().token_id, 1.22, 100, "Bid".to_string());

        assert_eq!(contract.get_balance(alice(), Some(standart_token().token_id)).0, 2u128);
        assert_eq!(contract.get_balance(alice(), Some(test_token().token_id)).0, 100u128);
        assert_eq!(contract.get_balance(carol(), Some(standart_token().token_id)).0, 99999999998u128);
        assert_eq!(contract.get_balance(carol(), Some(test_token().token_id)).0, 9900u128);

        // Currrent Spread
        let spread = contract.get_current_spread(test_token().token_id);
        println!("Spread => Ask: {}, Bid: {}", spread[0], spread[1]);
        assert_eq!(spread[0], 1.25);
        assert_eq!(spread[1], 1.22);
    }
}