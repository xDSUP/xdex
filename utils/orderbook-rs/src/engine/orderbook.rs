use std::fmt::Debug;

use super::domain::{Order, OrderSide, OrderType};
use super::order_queues::{OrderQueue};
use super::orders::OrderRequest;
use super::sequence;
use super::validation::OrderRequestValidator;

const MIN_SEQUENCE_ID: u64 = 1;
const MAX_SEQUENCE_ID: u64 = 1000;
const MAX_STALLED_INDICES_IN_QUEUE: u64 = 10;
const ORDER_QUEUE_INIT_CAPACITY: usize = 500;

extern crate near_sdk;
use self::near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use self::near_sdk::serde::Serialize;

pub type OrderProcessingResult = Vec<Result<Success, Failed>>;

#[derive(Debug, Serialize)]
pub enum Success {
    Accepted {
        id: u64,
        order_type: OrderType,
        order_creator: String,
        ts: u64,
    },

    Filled {
        order_id: u64,
        side: OrderSide,
        order_type: OrderType,
        price: f64,
        qty: u128,
        order_creator: String,
        ts: u64,
    },

    PartiallyFilled {
        order_id: u64,
        side: OrderSide,
        order_type: OrderType,
        price: f64,
        qty: u128,
        order_creator: String,
        ts: u64,
    },

    Amended {
        id: u64,
        price: f64,
        qty: u128,
        ts: u64,
    },

    Cancelled {
        id: u64,
        ts: u64,
    },
}

#[derive(Debug, Serialize)]
pub enum Failed {
    ValidationFailed(String),
    DuplicateOrderID(u64),
    NoMatch(u64),
    OrderNotFound(u64),
}

#[derive(Default, BorshDeserialize, BorshSerialize, Clone)]
pub struct Orderbook {
    order_asset: String,
    price_asset: String,
    pub bid_queue: OrderQueue<Order>,
    pub ask_queue: OrderQueue<Order>,
    seq: sequence::TradeSequence,
    order_validator: OrderRequestValidator,
}

fn get_current_time() -> u64 {
    use self::near_sdk::env;
    return env::block_timestamp();
}

impl Orderbook {
    pub fn new(order_asset: String, price_asset: String) -> Self {
        Orderbook {
            order_asset: order_asset.clone(),
            price_asset: price_asset.clone(),
            bid_queue: OrderQueue::new(
                OrderSide::Bid,
                MAX_STALLED_INDICES_IN_QUEUE,
                ORDER_QUEUE_INIT_CAPACITY,
            ),
            ask_queue: OrderQueue::new(
                OrderSide::Ask,
                MAX_STALLED_INDICES_IN_QUEUE,
                ORDER_QUEUE_INIT_CAPACITY,
            ),
            seq: sequence::new_sequence_gen(MIN_SEQUENCE_ID, MAX_SEQUENCE_ID),
            order_validator: OrderRequestValidator::new(
                order_asset,
                price_asset,
                MIN_SEQUENCE_ID,
                MAX_SEQUENCE_ID,
            ),
        }
    }

    pub fn process_order(&mut self, order: OrderRequest) -> OrderProcessingResult {
        // processing result accumulator
        let mut proc_result: OrderProcessingResult = vec![];

        // validate request
        if let Err(reason) = self.order_validator.validate(&order) {
            proc_result.push(Err(Failed::ValidationFailed(String::from(reason))));
            return proc_result;
        }

        match order {
            OrderRequest::NewMarketOrder {
                order_asset,
                price_asset,
                side,
                qty,
                order_creator,
                ts: _ts,
            } => {
                // generate new ID for order
                let order_id = self.seq.next_id();
                proc_result.push(Ok(Success::Accepted {
                    id: order_id,
                    order_type: OrderType::Market,
                    order_creator: order_creator.clone(),
                    ts: get_current_time(),
                }));

                self.process_market_order(
                    &mut proc_result,
                    order_id,
                    order_asset,
                    price_asset,
                    side,
                    qty,
                    order_creator,
                );
            }

            OrderRequest::NewLimitOrder {
                order_asset,
                price_asset,
                side,
                price,
                qty,
                order_creator,
                ts,
            } => {
                let order_id = self.seq.next_id();
                proc_result.push(Ok(Success::Accepted {
                    id: order_id,
                    order_type: OrderType::Limit,
                    order_creator: order_creator.clone(),
                    ts: get_current_time(),
                }));

                self.process_limit_order(
                    &mut proc_result,
                    order_id,
                    order_asset,
                    price_asset,
                    side,
                    price,
                    qty,
                    order_creator,
                    ts,
                );
            }

            OrderRequest::AmendOrder {
                id,
                side,
                price,
                qty,
                ts,
            } => {
                self.process_order_amend(&mut proc_result, id, side, price, qty, ts);
            }

            OrderRequest::CancelOrder { id, side } => {
                self.process_order_cancel(&mut proc_result, id, side);
            }
        }

        // return collected processing results
        proc_result
    }

    /// Get current spread as a tuple: (bid, ask)
    pub fn current_spread(&mut self) -> Option<(f64, f64)> {
        let bid = self.bid_queue.peek()?.price;
        let ask = self.ask_queue.peek()?.price;
        Some((bid, ask))
    }

    /* Processing logic */

    fn process_market_order(
        &mut self,
        results: &mut OrderProcessingResult,
        order_id: u64,
        order_asset: String,
        price_asset: String,
        side: OrderSide,
        qty: u128,
        order_creator: String,
    ) {
        // get copy of the current limit order
        let opposite_order_result = {
            let opposite_queue = match side {
                OrderSide::Bid => &mut self.ask_queue,
                OrderSide::Ask => &mut self.bid_queue,
            };
            opposite_queue.peek().cloned()
        };

        if let Some(opposite_order) = opposite_order_result {
            let matching_complete = self.order_matching(
                results,
                &opposite_order,
                order_id,
                order_asset.clone(),
                price_asset.clone(),
                OrderType::Market,
                side,
                qty,
                &order_creator,
            );

            if !matching_complete {
                // match the rest
                self.process_market_order(
                    results,
                    order_id,
                    order_asset,
                    price_asset,
                    side,
                    qty - opposite_order.qty,
                    order_creator,
                );
            }
        } else {
            // no limit orders found
            results.push(Err(Failed::NoMatch(order_id)));
        }
    }

    fn process_limit_order(
        &mut self,
        results: &mut OrderProcessingResult,
        order_id: u64,
        order_asset: String,
        price_asset: String,
        side: OrderSide,
        price: f64,
        qty: u128,
        order_creator: String,
        ts: u64,
    ) {
        // take a look at current opposite limit order
        let opposite_order_result = {
            let opposite_queue = match side {
                OrderSide::Bid => &mut self.ask_queue,
                OrderSide::Ask => &mut self.bid_queue,
            };
            opposite_queue.peek().cloned()
        };

        if let Some(opposite_order) = opposite_order_result {
            let could_be_matched = match side {
                // verify bid/ask price overlap
                OrderSide::Bid => price >= opposite_order.price,
                OrderSide::Ask => price <= opposite_order.price,
            };

            if could_be_matched {
                // match immediately
                let matching_complete = self.order_matching(
                    results,
                    &opposite_order,
                    order_id,
                    order_asset.clone(),
                    price_asset.clone(),
                    OrderType::Limit,
                    side,
                    qty,
                    &order_creator,
                );

                if !matching_complete {
                    // process the rest of new limit order
                    self.process_limit_order(
                        results,
                        order_id,
                        order_asset,
                        price_asset,
                        side,
                        price,
                        qty - opposite_order.qty,
                        order_creator,
                        ts,
                    );
                }
            } else {
                // just insert new order in queue
                self.store_new_limit_order(
                    results,
                    order_id,
                    order_asset,
                    price_asset,
                    side,
                    price,
                    qty,
                    ts,
                );
            }
        } else {
            self.store_new_limit_order(
                results,
                order_id,
                order_asset,
                price_asset,
                side,
                price,
                qty,
                ts,
            );
        }
    }

    fn process_order_amend(
        &mut self,
        results: &mut OrderProcessingResult,
        order_id: u64,
        side: OrderSide,
        price: f64,
        qty: u128,
        ts: u64,
    ) {
        let order_queue = match side {
            OrderSide::Bid => &mut self.bid_queue,
            OrderSide::Ask => &mut self.ask_queue,
        };

        if order_queue.amend(
            order_id,
            price,
            qty,
            ts,
            Order {
                order_id,
                order_asset: self.order_asset.clone(),
                price_asset: self.price_asset.clone(),
                side,
                price,
                qty,
            },
        ) {
            results.push(Ok(Success::Amended {
                id: order_id,
                price,
                qty,
                ts: get_current_time(),
            }));
        } else {
            results.push(Err(Failed::OrderNotFound(order_id)));
        }
    }

    fn process_order_cancel(
        &mut self,
        results: &mut OrderProcessingResult,
        order_id: u64,
        side: OrderSide,
    ) {
        let order_queue = match side {
            OrderSide::Bid => &mut self.bid_queue,
            OrderSide::Ask => &mut self.ask_queue,
        };

        if order_queue.cancel(order_id) {
            results.push(Ok(Success::Cancelled {
                id: order_id,
                ts: get_current_time(),
            }));
        } else {
            results.push(Err(Failed::OrderNotFound(order_id)));
        }
    }

    /* Helpers */

    fn store_new_limit_order(
        &mut self,
        results: &mut OrderProcessingResult,
        order_id: u64,
        order_asset: String,
        price_asset: String,
        side: OrderSide,
        price: f64,
        qty: u128,
        ts: u64,
    ) {
        let order_queue = match side {
            OrderSide::Bid => &mut self.bid_queue,
            OrderSide::Ask => &mut self.ask_queue,
        };
        if !order_queue.insert(
            order_id,
            price,
            qty,
            ts,
            Order {
                order_id,
                order_asset,
                price_asset,
                side,
                price,
                qty,
            },
        ) {
            results.push(Err(Failed::DuplicateOrderID(order_id)))
        };
    }

    fn order_matching(
        &mut self,
        results: &mut OrderProcessingResult,
        opposite_order: &Order,
        order_id: u64,
        order_asset: String,
        price_asset: String,
        order_type: OrderType,
        side: OrderSide,
        qty: u128,
        order_creator: &str,
    ) -> bool {
        // real processing time
        let deal_time = get_current_time();

        // match immediately
        if qty < opposite_order.qty {
            // fill new limit and modify opposite limit

            // report filled new order
            results.push(Ok(Success::Filled {
                order_id,
                side,
                order_type,
                price: opposite_order.price,
                qty,
                order_creator: order_creator.to_string(),
                ts: deal_time,
            }));

            // report partially filled opposite limit order
            results.push(Ok(Success::PartiallyFilled {
                order_id: opposite_order.order_id,
                side: opposite_order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                qty,
                order_creator: order_creator.to_string(),
                ts: deal_time,
            }));

            // modify unmatched part of the opposite limit order
            {
                let opposite_queue = match side {
                    OrderSide::Bid => &mut self.ask_queue,
                    OrderSide::Ask => &mut self.bid_queue,
                };
                opposite_queue.modify_current_order(Order {
                    order_id: opposite_order.order_id,
                    order_asset,
                    price_asset,
                    side: opposite_order.side,
                    price: opposite_order.price,
                    qty: opposite_order.qty - qty,
                });
            }
        } else if qty > opposite_order.qty {
            // partially fill new limit order, fill opposite limit and notify to process the rest

            // report new order partially filled
            results.push(Ok(Success::PartiallyFilled {
                order_id,
                side,
                order_type,
                price: opposite_order.price,
                qty: opposite_order.qty,
                order_creator: order_creator.to_string(),
                ts: deal_time,
            }));

            // report filled opposite limit order
            results.push(Ok(Success::Filled {
                order_id: opposite_order.order_id,
                side: opposite_order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                qty: opposite_order.qty,
                order_creator: order_creator.to_string(),
                ts: deal_time,
            }));

            // remove filled limit order from the queue
            {
                let opposite_queue = match side {
                    OrderSide::Bid => &mut self.ask_queue,
                    OrderSide::Ask => &mut self.bid_queue,
                };
                opposite_queue.pop();
            }

            // matching incomplete
            return false;
        } else {
            // orders exactly match -> fill both and remove old limit

            // report filled new order
            results.push(Ok(Success::Filled {
                order_id,
                side,
                order_type,
                price: opposite_order.price,
                qty,
                order_creator: order_creator.to_string(),
                ts: deal_time,
            }));
            // report filled opposite limit order
            results.push(Ok(Success::Filled {
                order_id: opposite_order.order_id,
                side: opposite_order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                qty,
                order_creator: order_creator.to_string(),
                ts: deal_time,
            }));

            // remove filled limit order from the queue
            {
                let opposite_queue = match side {
                    OrderSide::Bid => &mut self.ask_queue,
                    OrderSide::Ask => &mut self.bid_queue,
                };
                opposite_queue.pop();
            }
        }

        // complete matching
        true
    }
}

#[cfg(test)]
mod test {

    use super::super::orders;
    use super::*;



    #[test]
    fn cancel_nonexisting() {
        let mut orderbook = Orderbook::new("BTC".to_string(), "USD".to_string());
        let request = orders::limit_order_cancel_request(1, OrderSide::Bid);
        let mut result = orderbook.process_order(request);

        assert_eq!(result.len(), 1);
        match result.pop().unwrap() {
            Err(_) => (),
            _ => panic!("unexpected events"),
        }
    }
}
