extern crate core;


use std::vec;

use near_sdk::{AccountId, Balance, env, near_bindgen, Promise};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet, Vector};
use near_sdk::json_types::U128;
use num_traits::cast::ToPrimitive;

use orderbook::{Failed, Order, Orderbook, orders, OrderSide, OrderType, Success};

use crate::account::TokenAccount;
use crate::ballot::{BallotHandler, StakeInfo, UserRequest};
use crate::request::{Request, RequestId, Vote};
use crate::token::{Token, TokenId};
use crate::wallet::TokenWallet;

mod account;
mod request;

mod ballot;
mod token;
mod wallet;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
const NANOSEC_IN_DAY: u64 = 86_400_000_000_000;
const NANOSEC_IN_DAY_F64: f64 = 86_400_000_000_000.0;
const PERCENT_STAKING_PER_YEAR: f64 = 0.20;
const STAKING_PERCENT: f64 = PERCENT_STAKING_PER_YEAR / 365.0 / NANOSEC_IN_DAY_F64;
/// за наносекунду
const VOTING_TIME: u64 = NANOSEC_IN_DAY;
//const SINGLE_CALL_GAS: u64 = 20_000_000_000_000; // 2 x 10^14
//const TRANSFER_FROM_NEAR_COST: u128 = 36_500_000_000_000_000_000_000; // 365 x 10^20

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// sha256(AccountID) -> Account details.
    pub wallets: LookupMap<Vec<u8>, TokenWallet>,
    pub order_books: LookupMap<TokenId, Orderbook>,
    /// разрешенные токены
    pub tokens: UnorderedMap<TokenId, Token>,
    pub ballot_handler: BallotHandler,
    pub staking: LookupMap<Vec<u8>, StakeInfo>,

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
    /// Инициализирует контракт и переводит все биржевые токены XDHO к `owner_id`.
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        assert!(env::state_read::<Self>().is_none(), "Already initialized");
        let mut contract = Self {
            order_books: LookupMap::new(b"o".to_vec()),
            wallets: LookupMap::new(b"w".to_vec()),
            tokens: UnorderedMap::new(b"t".to_vec()),
            ballot_handler: BallotHandler::new(),
            owner_id: owner_id.clone(),
            staking: LookupMap::new(b"s".to_vec()),
        };
        contract.add_token(Token {
            token_id: "XDHO".to_string(),
            owner_id: owner_id.clone(),
            supply: 100_000_000_000,
        });

        contract
    }

    /// Устанавливает допустимое кол-во `allowance` для `escrow_account_id` которое он сможет
    /// списывать с владельца этого аккаунта (`predecessor_id`)
    pub fn set_allowance(
        &mut self,
        escrow_account_id: AccountId,
        token_id: TokenId,
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
        token_id: TokenId,
        amount: U128,
    ) {
        println!(
            "Перепожу {} {} от {} к {}. Иниц: {}",
            amount.0,
            token_id.clone(),
            owner_id,
            new_owner_id,
            env::predecessor_account_id()
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
    pub fn transfer(&mut self, new_owner_id: AccountId, token_id: TokenId, amount: U128) {
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
    pub fn get_balance(&self, owner_id: AccountId, token_id: TokenId) -> U128 {
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
        token_id: TokenId,
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
            self.get_standard_token(),
            U128::from(self.get_standard_price() * amount.0),
        );
        Promise::new(to).transfer(amount.0)
    }

    pub fn get_standard_price(&self) -> Balance {
        52u128 // монет за один NEAR
    }
}

/// Секция работы с голосованием
#[near_bindgen]
impl Contract {
    pub fn stake(&mut self, amount: Balance) {
        let old_staking = self.get_staking(env::predecessor_account_id());
        if old_staking.staked != 0 {
            env::panic(b"You have staked tokens. First do the unstaking");
        }

        self.transfer_from(
            env::predecessor_account_id(),
            self.owner_id.clone(),
            "XDHO".to_string(),
            U128(amount),
        );
        self.set_staking(env::predecessor_account_id(), amount);
    }

    pub fn get_staking(&self, owner_id: AccountId) -> StakeInfo {
        let account_hash = env::sha256(owner_id.as_bytes());
        self.staking.get(&account_hash).unwrap_or(StakeInfo {
            staked: 0,
            created_time: env::block_timestamp(),
        })
    }

    fn set_staking(&mut self, owner_id: AccountId, amount: Balance) {
        let account_hash = env::sha256(owner_id.as_bytes());
        self.staking.insert(&account_hash, &StakeInfo {
            staked: amount,
            created_time: env::block_timestamp(),
        });
    }

    pub fn unstake(&mut self) {
        let old_staking = self.get_staking(env::predecessor_account_id());
        if old_staking.staked == 0 {
            env::panic(b"You did't have staked tokens. First do the staking");
        }
        let created_time = env::block_timestamp() - old_staking.created_time;
        let mut amount = created_time.to_f64().unwrap() * STAKING_PERCENT + 1.0;
        amount *= old_staking.staked.to_f64().unwrap();
        self.transfer_from_user(
            self.owner_id.clone(),
            env::predecessor_account_id(),
            "XDHO".to_string(),
            U128(amount.to_u128().unwrap()),
        );
        let account_hash = env::sha256(env::predecessor_account_id().as_bytes());
        self.staking.remove(&account_hash);
    }

    pub fn add_new_request(&mut self, request: UserRequest) {
        if self.tokens.get(&request.token_id).is_some(){
            env::panic(b"A token with this ID already exists");
        }

        //TODO: брать плату за запросики
        self.ballot_handler.add_new_request(request);
    }

    pub fn get_all_requests(&self) -> Vec<Request> {
        self.ballot_handler.get_all_requests()
    }

    pub fn vote(&mut self, request_id: RequestId, vote: bool) {
        let request = self.ballot_handler.get_request(request_id);
        match request {
            None => env::panic(b"There is no request with this id"),
            Some(req) => {
                if req.created_time + VOTING_TIME < env::block_timestamp(){
                    //TODO: финализировать тута
                    env::panic(b"The voting has already ended");
                }
            },
        }
        let staked = self.get_staking(env::predecessor_account_id());
        if staked.staked == 0 {
            env::panic(b"You did't have staked tokens. First do the staking");
        }
        if self.is_voting(env::predecessor_account_id(), request_id) {
            env::panic(b"You have already voted for this request");
        }
        self.ballot_handler.vote(request_id, vote);
    }

    pub fn is_voting(&self, voter_id: AccountId, request_id: RequestId) -> bool {
        self.ballot_handler.is_vote(request_id, voter_id)
    }

    pub fn finalize(&mut self, request_id: RequestId) {
        let request = self.ballot_handler.get_request(request_id);
        match request {
            None => env::panic(b"There is no request with this id"),
            Some(req) => {
                if req.created_time + VOTING_TIME > env::block_timestamp(){
                    env::panic(b"The voting is not over yet");
                }
            },
        }


    }
}

#[near_bindgen]
impl Contract {
    pub fn get_tokens(&self) -> Vec<Token> {
        let mut result: Vec<Token> = Vec::new();
        for token in self.tokens.values() {
            result.push(token);
        }
        result
    }

    #[private]
    pub fn add_token(&mut self, token: Token) {
        self.tokens.insert(&token.token_id,&token);
        let token_id = token.token_id;
        let owner_id = &token.owner_id;
        let mut account = self.get_account(owner_id, token_id.clone());
        account.balance = token.supply;
        self.set_account(&token.owner_id, &account, &token_id.clone());
        let orderbook = Orderbook::new(token_id.clone(), self.get_standard_token());
        self.order_books.insert(&token_id.clone(), &orderbook);
    }

    fn get_standard_token(&self) -> String {
        "XDHO".to_string()
    }

    fn get_standard_decimal(&self) -> i8 {
        2
    }

    fn get_wallet(&self, owner_id: &AccountId) -> TokenWallet {
        let account_hash = env::sha256(owner_id.as_bytes());
        self.wallets
            .get(&account_hash)
            .unwrap_or(TokenWallet::new(account_hash))
    }

    /// Helper method to get the account details for `owner_id`.
    fn get_account(&self, owner_id: &AccountId, token_id: TokenId) -> TokenAccount {
        self.get_wallet(owner_id)
            .get_account(&token_id)
            .unwrap_or(TokenAccount::new(env::sha256(owner_id.as_bytes())))
    }

    /// Helper method to set the account details for `owner_id` to the state.
    fn set_account(
        &mut self,
        owner_id: &AccountId,
        account: &TokenAccount,
        token_id: &TokenId,
    ) {
        self.get_wallet(owner_id).set_account(
            token_id,
            &account,
        )
    }
}

fn parse_side(side: &str) -> Option<OrderSide> {
    match side {
        "Ask" => Some(OrderSide::Ask),
        "Bid" => Some(OrderSide::Bid),
        _ => env::panic(b"Side not parsed!"),
    }
}

fn get_current_time() -> u64 {
    return env::block_timestamp();
}

#[near_bindgen]
impl Contract {
    pub fn new_ask_limit_order(&mut self, token_id: TokenId, price: f64, quantity: u128) {
        self.new_limit_order(token_id, price, quantity, "Ask".to_string())
    }

    pub fn new_bid_limit_order(&mut self, token_id: TokenId, price: f64, quantity: u128) {
        self.new_limit_order(token_id, price, quantity, "Bid".to_string())
    }

    /// Создает новый лимитный ордер:
    /// * 'side':
    /// Ask - заявка на продажу
    /// Bid - заявка на покупку
    pub fn new_limit_order(&mut self, token_id: TokenId, price: f64, quantity: u128, side: String) {
        let side = parse_side(side.as_str()).unwrap();
        let token = match side {
            OrderSide::Bid => self.get_standard_token(),
            OrderSide::Ask => token_id.clone(),
        };
        let amount = match side {
            OrderSide::Bid => (price * quantity.to_f64().unwrap()).to_u128().unwrap(),
            OrderSide::Ask => quantity,
        };
        /**
        Цена бид – это цена спроса или максимальная цена, по которой покупатель согласен купить товар.
        Покупатель не хочет покупать дорого. Это логика закона спроса и предложения

        Цена аск – это цена предложения или наименьшая цена, по которой продавец согласен продать товар.
        Продавец не хочет продавать дешево
         */
        match side {
            OrderSide::Ask => {
                println!(
                    "New limit order на продажу {} ${} по цене {} от signer: {}",
                    quantity,
                    token_id,
                    price,
                    env::signer_account_id(),
                );
            }
            OrderSide::Bid => {
                println!(
                    "New limit order на покупку {} ${} по цене {} от signer: {}",
                    quantity,
                    token_id,
                    price,
                    env::signer_account_id(),
                );
            }
        }

        self.transfer_from(
            env::signer_account_id(),
            env::current_account_id(),
            token,
            U128(amount),
        );
        self.post_transfer(token_id, price, quantity, side);
    }

    #[private]
    fn post_transfer(&mut self, token_id: TokenId, price: f64, quantity: u128, side: OrderSide) {
        env::log(b"Token Transfer Successful.");
        let order = orders::new_limit_order_request(
            token_id.clone(),
            self.get_standard_token(),
            side,
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

    /// Создает новый рыночный ордер:
    /// * 'side':
    /// Ask - заявка на продажу
    /// Bid - заявка на покупку
    pub fn new_market_order(&mut self, token_id: TokenId, quantity: u128, side: String) {
        let side = parse_side(side.as_str()).unwrap();
        println!(
            "New рыночн order на {} {} ${} от signer: {}",
            match side {
                OrderSide::Ask => { "продажу" }
                OrderSide::Bid => { "покупку" }
            },
            quantity,
            token_id,
            env::signer_account_id(),
        );

        // для продажи сразу переводим, для покупки будем переводить потом, когда будем знать цену
        match side {
            OrderSide::Ask => {
                self.transfer_from(
                    env::signer_account_id(),
                    env::current_account_id(),
                    token_id.clone(),
                    U128::from(quantity),
                );
            }
            _ => {}
        };

        let order = orders::new_market_order_request(
            token_id.clone(),
            self.get_standard_token(),
            side,
            quantity,
            env::signer_account_id(),
            get_current_time(),
        );

        let mut order_book = self.order_books.get(&token_id.clone()).unwrap();
        let res = order_book.process_order(order);
        self.order_books.insert(&token_id.clone(), &order_book);

        self.process_orderbook_result(token_id, res);
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

    pub fn get_ask_orders(&self, token_id: TokenId) -> Vec<Order> {
        let order_book = self.order_books.get(&token_id).unwrap();
        Vec::from_iter(order_book.ask_queue.clone().orders.into_values())
    }

    pub fn get_orders(&self, account_id: AccountId, token_id: TokenId, side: String) -> Vec<Order> {
        let side = parse_side(side.as_str()).unwrap();
        let order_book = self.order_books.get(&token_id);
        match order_book {
            Some(t) => (t.get_orders(account_id, side)),
            None => env::panic(b"OrderBook not init!"),
        }
    }

    /**
    pub fn get_all_orders(&self, account_id: AccountId) -> Vec<Order> {
        let orders = Vec::new();
        for token in self.tokens {
            let order_book = self.order_books.get(&token.token_id);
            orders.push(order_book.)
                    }

                match order_book {
            Some(t) => (t.get_orders(account_id, side)),
            None => panic(b"OrderBook not init!"),
        }

            }
     */

    pub fn get_bid_orders(&self, token_id: TokenId) -> Vec<Order> {
        let order_book = self.order_books.get(&token_id).unwrap();
        Vec::from_iter(order_book.bid_queue.clone().orders.into_values())
    }

    pub fn get_current_spread(&self, token_id: TokenId) -> Vec<f64> {
        let order_book = self.order_books.get(&token_id).unwrap();
        if let Some((bid, ask)) = order_book.clone().current_spread() {
            vec![ask, bid]
        } else {
            vec![0.0, 0.0]
        }
    }

    fn transfer_from_contract(&mut self, new_owner_id: AccountId, token_id: TokenId, amount: U128) {
        self.transfer_from_user(env::current_account_id(), new_owner_id, token_id, amount);
    }

    fn transfer_from_user(&mut self, owner_id: AccountId, new_owner_id: AccountId, token_id: TokenId, amount: U128) {
        let mut acc = self.get_account(&owner_id.clone(), token_id.clone());
        let in_acc = env::predecessor_account_id();
        let allownce = acc.get_allowance(&in_acc.clone());
        acc.set_allowance(&in_acc.clone(), allownce + amount.0);
        self.set_account(&owner_id.clone(), &acc, &token_id.clone());

        self.transfer_from(
            owner_id,
            new_owner_id,
            token_id.clone(),
            amount,
        );
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
                    id,
                    order_type: _,
                    order_creator,
                    ts: _,
                } => {
                    println!("Принят №{} ордер ${} от {}", id, token_id, order_creator);
                }
                Success::Filled {
                    order_id: id,
                    side,
                    order_type,
                    price,
                    qty,
                    order_creator,
                    ts: _,
                } => {
                    let token = match *side {
                        OrderSide::Ask => self.get_standard_token(),
                        OrderSide::Bid => token_id.clone(),
                    };

                    println!("Выполнен ордер №{} от {} {} {} ${}",
                             id,
                             order_creator,
                             match side {
                                 OrderSide::Bid => "на покупку",
                                 OrderSide::Ask => "на продажу",
                             },
                             *qty,
                             token_id,
                    );
                    match *order_type {
                        OrderType::Limit => {
                            match side {
                                OrderSide::Bid => {
                                    self.transfer_from_contract(
                                        order_creator.to_string(),
                                        token,
                                        U128::from(*qty),
                                    );
                                }
                                OrderSide::Ask => {
                                    let amount = U128::from((price * (*qty).to_f64().unwrap()).to_u128().unwrap());
                                    self.transfer_from_contract(
                                        order_creator.to_string(),
                                        token,
                                        amount,
                                    );
                                }
                            };
                        }
                        OrderType::Market => {
                            match side {
                                OrderSide::Bid => {
                                    let amount = U128::from((price * (*qty).to_f64().unwrap()).to_u128().unwrap());
                                    // переводим токены на контракт, откуда их возьмет продавец
                                    self.transfer(
                                        env::current_account_id(),
                                        self.get_standard_token(),
                                        amount,
                                    );
                                    self.transfer_from_contract(
                                        order_creator.to_string(),
                                        token,
                                        U128::from(*qty),
                                    );
                                }
                                OrderSide::Ask => {
                                    self.transfer_from_contract(
                                        order_creator.to_string(),
                                        self.get_standard_token(),
                                        U128::from((price * (*qty).to_f64().unwrap()).to_u128().unwrap()),
                                    );
                                }
                            };
                        }
                    }
                }
                Success::PartiallyFilled {
                    order_id: id,
                    side,
                    order_type,
                    price,
                    qty,
                    order_creator,
                    ts: _,
                } => {
                    let token = match *side {
                        OrderSide::Ask => self.get_standard_token(),
                        OrderSide::Bid => token_id.clone(),
                    };

                    println!("Частично выполнен ордер №{} от {} {} {} ${}",
                             id,
                             order_creator,
                             match side {
                                 OrderSide::Bid => "на покупку",
                                 OrderSide::Ask => "на продажу",
                             },
                             *qty,
                             token_id,
                    );
                    match *order_type {
                        OrderType::Limit => {
                            let amount = match *side {
                                OrderSide::Ask => (price * (*qty).to_f64().unwrap()).to_u128().unwrap(),
                                OrderSide::Bid => *qty,
                            };

                            self.transfer_from_contract(
                                order_creator.to_string(),
                                token,
                                U128::from(amount),
                            );
                        }
                        OrderType::Market => {
                            match side {
                                OrderSide::Bid => {
                                    let amount = U128::from((price * (*qty).to_f64().unwrap()).to_u128().unwrap());
                                    // переводим токены на контракт, откуда их возьмет продавец
                                    self.transfer(
                                        env::current_account_id(),
                                        self.get_standard_token(),
                                        amount,
                                    );
                                    self.transfer_from_contract(
                                        order_creator.to_string(),
                                        token,
                                        U128::from(*qty),
                                    );
                                }
                                OrderSide::Ask => {
                                    self.transfer_from_contract(
                                        order_creator.to_string(),
                                        self.get_standard_token(),
                                        U128::from((price * (*qty).to_f64().unwrap()).to_u128().unwrap()),
                                    );
                                }
                            };
                        }
                    }
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

    fn _only_our_token(&mut self, token_id: TokenId) {
        //self.tokens
        //assert!(self.tok
        //    "Only contract owner can sign transactions for this method."
        //);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use near_sdk::{AccountId, env, Gas, MockedBlockchain};
    use near_sdk::{testing_env, VMContext};
    use near_sdk::json_types::U128;
    use num_traits::ToPrimitive;

    use crate::{Contract, NANOSEC_IN_DAY, PERCENT_STAKING_PER_YEAR, Token, UserRequest};

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
            owner_id: bob(),
            supply: 10000,
        }
    }

    fn alice() -> AccountId {
        "alice.near".to_string()
    }

    fn bob() -> AccountId {
        "bob.near".to_string()
    }

    fn ivan() -> AccountId {
        "ivan.near".to_string()
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
        get_extend_context(predecessor_account_id, bob())
    }

    fn get_extend_context(predecessor_account_id: AccountId, signer: AccountId) -> VMContext {
        VMContext {
            current_account_id: alice(),
            signer_account_id: signer,
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
        let context = get_context(bob());
        testing_env!(context);

        let mut contract = Contract::new(bob());
        assert_eq!(
            contract.get_balance(bob(), standart_token().token_id).0,
            standart_token().supply
        );

        contract.add_token(test_token());
        let balance = contract
            .get_balance(bob(), test_token().token_id)
            .0;
        assert_eq!(balance, 10000u128);
        contract
    }

    fn get_contract_with_request() -> Contract{
        let mut contract = init_contract_with_tokens_and_limit_bids();
        contract.add_new_request(UserRequest{
            token_id: "SOSI".to_string(),
            title: "Тестовое название".to_string(),
            description: "ПРИВЕТИКИ".to_string(),
            price: 23,
            supply: 100000,
            hash: "сосисочка".to_string()
        });
        contract
    }

    #[test]
    fn test_new() {
        let context = get_context(carol());
        testing_env!(context);

        let contract = Contract::new(bob());
        assert_eq!(contract.get_balance(bob(), standart_token().token_id).0, standart_token().supply);
    }

    #[test]
    fn test_transfer() {
        let context = get_context(carol());
        testing_env!(context);
        let total_supply = standart_token().supply;
        let mut contract = Contract::new(carol());
        let transfer_amount = total_supply / 3;
        contract.transfer(bob(), standart_token().token_id, transfer_amount.into());
        assert_eq!(
            contract.get_balance(carol(), standart_token().token_id).0,
            (total_supply - transfer_amount)
        );
        assert_eq!(contract.get_balance(bob(), standart_token().token_id).0, transfer_amount);
    }

    #[test]
    fn test_failed_unstaking() {
        let mut contract = init_contract_with_tokens_and_limit_bids();
        let context = get_extend_context(carol(), carol());
        testing_env!(context);

        catch_unwind_silent(move || {
            contract.unstake();
        }).unwrap_err();
    }

    #[test]
    fn test_failed_staking() {
        let mut contract = init_contract_with_tokens_and_limit_bids();
        let context = get_extend_context(carol(), carol());
        testing_env!(context);

        contract.stake(123);
        catch_unwind_silent(move || {
            contract.stake(123);
        }).unwrap_err();
    }

    #[test]
    fn test_staking() {
        let mut contract = init_contract_with_tokens_and_limit_bids();
        let context = get_extend_context(carol(), carol());
        testing_env!(context);
        let amount = 100;
        let old_balance = contract.get_balance(carol(), standart_token().token_id);
        contract.stake(amount);
        assert_eq!(contract.get_balance(carol(), standart_token().token_id).0, old_balance.0 - amount);
        assert_eq!(contract.get_staking(carol()).staked, amount);

        let mut context = get_extend_context(carol(), carol());
        context.block_timestamp = NANOSEC_IN_DAY * 365;
        testing_env!(context);
        contract.unstake();
        assert_eq!(
            contract.get_balance(carol(), standart_token().token_id).0,
            old_balance.0 + (amount.to_f64().unwrap() * PERCENT_STAKING_PER_YEAR).to_u128().unwrap()
        );
        assert_eq!(contract.get_staking(carol()).staked, 0);
    }

    #[test]
    fn test_voting_without_staking() {
        let mut contract = get_contract_with_request();
        let context = get_extend_context(carol(), carol());
        testing_env!(context);
        let amount = 100;
        let old_balance = contract.get_balance(carol(), standart_token().token_id);
        catch_unwind_silent(move || {
            contract.vote(0, true);
        }).unwrap_err();
    }

    #[test]
    fn test_voting_after_staking() {
        let mut contract = get_contract_with_request();
        let context = get_extend_context(carol(), carol());
        testing_env!(context);
        let amount = 100;
        let old_balance = contract.get_balance(carol(), standart_token().token_id);
        contract.stake(amount);
        contract.vote(0, true);
        assert!(contract.is_voting(carol(), 0));
    }

    #[test]
    fn test_wallets() {
        let context = get_context(carol());
        testing_env!(context);
        let mut contract = Contract::new(carol());

        contract.pay_standard_token(U128(3), bob());
        assert_eq!(env::account_balance(), 97);
        let balance = contract
            .get_balance(bob(), standart_token().token_id)
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
            .get_balance(bob(), standart_token().token_id)
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
            contract.set_allowance(carol(), standart_token().token_id, (total_supply / 2).into());
        })
            .unwrap_err();
    }

    #[test]
    fn test_carol_add_new_token() {
        // Acting as carol
        testing_env!(get_context(carol()));
        let total_supply = 10000;
        let mut contract = Contract::new(carol());

        contract.add_token(Token {
            token_id: test_token().token_id,
            supply: test_token().supply,
            owner_id: carol(),
        });
        let balance = contract
            .get_balance(carol(), test_token().token_id)
            .0;
        assert_eq!(balance, 10000u128);

        let allowance = total_supply / 3;
        let transfer_amount = allowance / 3;
        contract.set_allowance(bob(), test_token().token_id, allowance.into());
        assert_eq!(
            contract
                .get_allowance(carol(), bob(), test_token().token_id)
                .0,
            allowance
        );
        // Acting as bob now
        testing_env!(get_context(bob()));
        contract.transfer_from(
            carol(),
            alice(),
            test_token().token_id,
            transfer_amount.into(),
        );
        assert_eq!(
            contract.get_balance(carol(), test_token().token_id).0,
            total_supply - transfer_amount
        );
        assert_eq!(
            contract.get_balance(alice(), test_token().token_id).0,
            transfer_amount
        );
        assert_eq!(
            contract.get_allowance(carol(), bob(), standart_token().token_id).0,
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
        contract.set_allowance(bob(), standart_token().token_id, allowance.into());
        assert_eq!(contract.get_allowance(carol(), bob(), standart_token().token_id).0, allowance);
        // Acting as bob now
        testing_env!(get_context(bob()));
        contract.transfer_from(carol(), alice(), standart_token().token_id, transfer_amount.into());
        assert_eq!(
            contract.get_balance(carol(), standart_token().token_id).0,
            total_supply - transfer_amount
        );
        assert_eq!(contract.get_balance(alice(), standart_token().token_id).0, transfer_amount);
        assert_eq!(
            contract.get_allowance(carol(), bob(), standart_token().token_id).0,
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

    fn init_contract_with_tokens_and_limit_bids() -> Contract {
        let mut contract = init_contract_with_tokens();
        contract.new_limit_order(test_token().token_id, 5.0, 100, "Ask".to_string());
        contract.new_limit_order(test_token().token_id, 6.0, 200, "Ask".to_string());
        contract.new_limit_order(test_token().token_id, 10.0, 300, "Ask".to_string());
        contract.new_limit_order(test_token().token_id, 11.0, 500, "Ask".to_string());
        contract.new_limit_order(test_token().token_id, 16.0, 1000, "Ask".to_string());
        contract.new_limit_order(test_token().token_id, 3.0, 20, "Bid".to_string());
        contract.new_limit_order(test_token().token_id, 2.0, 40, "Bid".to_string());
        contract.new_limit_order(test_token().token_id, 1.0, 100, "Bid".to_string());

        contract.transfer(carol(), standart_token().token_id, U128::from(1000u128));
        contract.transfer(ivan(), standart_token().token_id, U128::from(1000u128));
        contract.transfer(carol(), test_token().token_id, U128::from(1000u128));
        contract.transfer(ivan(), test_token().token_id, U128::from(1000u128));

        let context = get_extend_context(carol(), carol());
        testing_env!(context);
        contract.new_limit_order(test_token().token_id, 5.0, 50, "Ask".to_string());
        contract.new_limit_order(test_token().token_id, 4.0, 30, "Ask".to_string());
        contract.new_limit_order(test_token().token_id, 1.0, 100, "Bid".to_string());

        let spread = contract.get_current_spread(test_token().token_id);
        println!("Spread => Ask: {}, Bid: {}", spread[0], spread[1]);
        assert_eq!(spread[0], 4.0);
        assert_eq!(spread[1], 3.0);
        contract
    }

    #[test]
    fn new_limit_order() {
        testing_env!(get_context(bob()));
        let mut contract = init_contract_with_tokens();

        // Currrent Spread
        let spread = contract.get_current_spread(test_token().token_id);
        println!("Spread => Ask: {}, Bid: {}", spread[0], spread[1]);
        assert_eq!(spread[0], 0.0);
        assert_eq!(spread[1], 0.0);

        let std_balance = contract.get_balance(bob(), standart_token().token_id).0;
        let test_balance = contract.get_balance(bob(), test_token().token_id).0;
        // Ask Order
        contract.new_limit_order(test_token().token_id, 1.25, 2, "Ask".to_string());
        // Bid Order
        contract.new_limit_order(test_token().token_id, 1.22, 50, "Bid".to_string());
        contract.new_limit_order(test_token().token_id, 1.20, 50, "Bid".to_string());

        assert_eq!(
            contract.get_balance(alice(), standart_token().token_id).0 +
                contract.get_balance(bob(), standart_token().token_id).0,
            std_balance
        );
        assert_eq!(
            contract.get_balance(alice(), test_token().token_id).0 +
                contract.get_balance(bob(), test_token().token_id).0,
            test_balance
        );
        let spread = contract.get_current_spread(test_token().token_id);
        println!("Spread => Ask: {}, Bid: {}", spread[0], spread[1]);
        assert_eq!(spread[0], 1.25);
        assert_eq!(spread[1], 1.22);
    }

    #[test]
    fn matching_limit_order() {
        let mut contract = init_contract_with_tokens_and_limit_bids();
        let context = get_extend_context(ivan(), ivan());
        testing_env!(context);

        assert_eq!(contract.get_balance(ivan(), standart_token().token_id).0, 1000u128);
        assert_eq!(contract.get_balance(bob(), test_token().token_id).0, 5900u128);
        contract.new_limit_order(test_token().token_id, 1.0, 50, "Ask".to_string());
        assert_eq!(contract.get_balance(ivan(), standart_token().token_id).0, 1120u128);
        assert_eq!(contract.get_balance(bob(), test_token().token_id).0, 5950u128);

        let spread = contract.get_current_spread(test_token().token_id);
        println!("Spread => Ask: {}, Bid: {}", spread[0], spread[1]);
        assert_eq!(spread[0], 4.0);
        assert_eq!(spread[1], 2.0);

        print_all_balances(&contract, ivan());
        contract.new_limit_order(test_token().token_id, 5.0, 50, "Bid".to_string());
        print_all_balances(&contract, bob());
        print_all_balances(&contract, ivan());
        assert_eq!(contract.get_balance(bob(), standart_token().token_id).0, 99999997860u128);
        assert_eq!(contract.get_balance(ivan(), test_token().token_id).0, 1000u128);
        let spread = contract.get_current_spread(test_token().token_id);
        println!("Spread => Ask: {}, Bid: {}", spread[0], spread[1]);
        assert_eq!(spread[0], 5.0);
        assert_eq!(spread[1], 2.0);
    }

    #[test]
    fn matching_market_order() {
        let mut contract = init_contract_with_tokens_and_limit_bids();
        let context = get_extend_context(ivan(), ivan());
        testing_env!(context);


        assert_eq!(contract.get_balance(ivan(), standart_token().token_id).0, 1000u128);
        assert_eq!(contract.get_balance(bob(), test_token().token_id).0, 5900u128);
        contract.new_market_order(test_token().token_id, 50, "Ask".to_string());
        assert_eq!(contract.get_balance(ivan(), standart_token().token_id).0, 1120u128);
        assert_eq!(contract.get_balance(bob(), test_token().token_id).0, 5950u128);
        print_all_balances(&contract, bob());
        print_all_balances(&contract, ivan());

        let spread = contract.get_current_spread(test_token().token_id);
        println!("Spread => Ask: {}, Bid: {}", spread[0], spread[1]);
        assert_eq!(spread[0], 4.0);
        assert_eq!(spread[1], 2.0);

        contract.new_market_order(test_token().token_id, 50, "Bid".to_string());

        print_all_balances(&contract, ivan());
        let spread = contract.get_current_spread(test_token().token_id);
        println!("Spread => Ask: {}, Bid: {}", spread[0], spread[1]);
        assert_eq!(spread[0], 5.0);
        assert_eq!(spread[1], 2.0);
    }

    #[test]
    fn cansel_limit_order() {
        let mut contract = init_contract_with_tokens_and_limit_bids();

        let context = get_extend_context(bob(), bob());
        testing_env!(context);

        contract.cancel_limit_order(test_token().token_id, 6, "Bid".to_string());

        let spread = contract.get_current_spread(test_token().token_id);
        println!("Spread => Ask: {}, Bid: {}", spread[0], spread[1]);
        assert_eq!(spread[0], 4.0);
        assert_eq!(spread[1], 2.0);
    }
}
