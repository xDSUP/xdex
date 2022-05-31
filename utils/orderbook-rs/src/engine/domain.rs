use std::fmt::Debug;

extern crate near_sdk;
use self::near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use self::near_sdk::serde::Serialize;

#[derive(Debug, Copy, Clone, BorshDeserialize, BorshSerialize, Serialize)]
pub enum OrderSide {
    Bid,
    Ask,
}

impl Default for OrderSide {
    fn default() -> Self {
        OrderSide::Bid
    }
}

#[derive(Default, Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct Order {
    pub order_id: u64,
    pub order_asset: String,
    pub price_asset: String,
    pub side: OrderSide,
    pub price: f64,
    pub qty: u128,
    pub order_creator: String,
}

#[derive(Eq, PartialEq, Debug, Copy, Clone, BorshDeserialize, BorshSerialize, Serialize)]
pub enum OrderType {
    Market,
    Limit,
}

impl Default for OrderType {
    fn default() -> Self {
        OrderType::Market
    }
}
