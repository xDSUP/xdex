use std::fmt::Debug;

use super::domain::OrderSide;

#[derive(Debug)]
pub enum OrderRequest {
    NewMarketOrder {
        order_asset: String,
        price_asset: String,
        side: OrderSide,
        qty: u128,
        order_creator: String,
        ts: u64,
    },

    NewLimitOrder {
        order_asset: String,
        price_asset: String,
        side: OrderSide,
        price: f64,
        qty: u128,
        order_creator: String,
        ts: u64,
    },

    AmendOrder {
        id: u64,
        side: OrderSide,
        price: f64,
        qty: u128,
        ts: u64,
        order_creator: String,
    },

    CancelOrder {
        id: u64,
        side: OrderSide,
        //ts: SystemTime,
    },
}

/* Constructors */

/// Create request for the new market order
pub fn new_market_order_request(
    order_asset: String,
    price_asset: String,
    side: OrderSide,
    qty: u128,
    order_creator: String,
    ts: u64,
) -> OrderRequest {
    OrderRequest::NewMarketOrder {
        order_asset,
        price_asset,
        qty,
        side,
        order_creator,
        ts,
    }
}

/// Create request for the new limit order
pub fn new_limit_order_request(
    order_asset: String,
    price_asset: String,
    side: OrderSide,
    price: f64,
    qty: u128,
    order_creator: String,
    ts: u64,
) -> OrderRequest {
    OrderRequest::NewLimitOrder {
        order_asset,
        price_asset,
        side,
        price,
        qty,
        order_creator,
        ts,
    }
}

/// Create request for changing price/qty for the active limit order.
///
/// Note: do not change order side!
/// Instead cancel existing order and create a new one.
pub fn amend_order_request(
    id: u64,
    side: OrderSide,
    price: f64,
    qty: u128,
    ts: u64,
    order_creator: String,
) -> OrderRequest {
    OrderRequest::AmendOrder {
        id,
        side,
        price,
        qty,
        ts,
        order_creator
    }
}

/// Create request for cancelling active limit order
pub fn limit_order_cancel_request(order_id: u64, side: OrderSide) -> OrderRequest {
    OrderRequest::CancelOrder { id: order_id, side }
}
