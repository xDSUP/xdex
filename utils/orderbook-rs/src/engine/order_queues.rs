use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use super::domain::OrderSide;

extern crate near_sdk;
use self::near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use self::near_sdk::serde::Serialize;

#[derive(Clone, BorshDeserialize, BorshSerialize, Debug, Serialize)]
pub struct OrderIndex {
    pub id: u64,
    pub price: f64,
    pub quantity: u128,
    pub timestamp: u64,
    pub order_side: OrderSide,
}

// Arrange at first by price and after that by time
impl Ord for OrderIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.price < other.price {
            match self.order_side {
                OrderSide::Bid => Ordering::Less,
                OrderSide::Ask => Ordering::Greater,
            }
        } else if self.price > other.price {
            match self.order_side {
                OrderSide::Bid => Ordering::Greater,
                OrderSide::Ask => Ordering::Less,
            }
        } else {
            // FIFO
            other.timestamp.cmp(&self.timestamp)
        }
    }
}

impl PartialOrd for OrderIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for OrderIndex {
    fn eq(&self, other: &Self) -> bool {
        if self.price > other.price || self.price < other.price {
            false
        } else {
            self.timestamp == other.timestamp
        }
    }
}

impl Eq for OrderIndex {}

/// Public methods
#[derive(Default, BorshDeserialize, BorshSerialize, Clone, Debug)]
pub struct OrderQueue<T> {
    // use Option in order to replace heap in mutable borrow
    pub idx_queue: Option<BinaryHeap<OrderIndex>>,
    orders: HashMap<u64, T>,
    op_counter: u64,
    max_stalled: u64,
    queue_side: OrderSide,
}

impl<T> OrderQueue<T> {
    /// Create new order queue
    ///
    /// Queue is universal and could be used for both asks and bids
    pub fn new(side: OrderSide, max_stalled: u64, capacity: usize) -> Self {
        OrderQueue {
            idx_queue: Some(BinaryHeap::with_capacity(capacity)),
            orders: HashMap::with_capacity(capacity),
            op_counter: 0,
            max_stalled,
            queue_side: side,
        }
    }

    pub fn peek(&mut self) -> Option<&T> {
        // get best order ID
        let order_id = self.get_current_order_id()?;

        // obtain order info
        if self.orders.contains_key(&order_id) {
            self.orders.get(&order_id)
        } else {
            self.idx_queue.as_mut().unwrap().pop()?;
            self.peek()
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        // remove order index from queue in any case
        let order_id = self.idx_queue.as_mut()?.pop()?.id;

        if self.orders.contains_key(&order_id) {
            self.orders.remove(&order_id)
        } else {
            self.pop()
        }
    }

    // Add new limit order to the queue
    pub fn insert(&mut self, id: u64, price: f64, qty: u128, ts: u64, order: T) -> bool {
        if self.orders.contains_key(&id) {
            // do not update existing order
            return false;
        }

        // store new order
        self.idx_queue.as_mut().unwrap().push(OrderIndex {
            id,
            price,
            timestamp: ts,
            quantity: qty,
            order_side: self.queue_side,
        });
        self.orders.insert(id, order);
        true
    }

    // use it when price was changed
    pub fn amend(&mut self, id: u64, price: f64, qty: u128, ts: u64, order: T) -> bool {
        if self.orders.contains_key(&id) {
            // store new order data
            self.orders.insert(id, order);
            self.rebuild_idx(id, price, qty, ts);
            true
        } else {
            false
        }
    }

    pub fn cancel(&mut self, id: u64) -> bool {
        match self.orders.remove(&id) {
            Some(_) => {
                self.clean_check();
                true
            }
            None => false,
        }
    }

    /* Internal methods */

    /// Used internally when current order is partially matched.
    ///
    /// Note: do not modify price or time, cause index doesn't change!
    pub fn modify_current_order(&mut self, new_order: T) -> bool {
        if let Some(order_id) = self.get_current_order_id() {
            if self.orders.contains_key(&order_id) {
                self.orders.insert(order_id, new_order);
                return true;
            }
        }
        false
    }

    /// Verify if queue should be cleaned
    fn clean_check(&mut self) {
        if self.op_counter > self.max_stalled {
            self.op_counter = 0;
            self.remove_stalled()
        } else {
            self.op_counter += 1;
        }
    }

    /// Remove dangling indices without orders from queue
    fn remove_stalled(&mut self) {
        if let Some(idx_queue) = self.idx_queue.take() {
            let mut active_orders = idx_queue;
            active_orders.retain(|order_ptr| self.orders.contains_key(&order_ptr.id));
            self.idx_queue = Some(BinaryHeap::from(active_orders));
        }
    }

    /// Recreate order-index queue with changed index info
    fn rebuild_idx(&mut self, id: u64, price: f64, qty: u128, ts: u64) {
        if let Some(idx_queue) = self.idx_queue.take() {
            // deconstruct queue
            let mut active_orders = idx_queue;
            // remove old idx value
            active_orders.retain(|order_ptr| order_ptr.id != id);
            // insert new one
            active_orders.push(OrderIndex {
                id,
                price,
                quantity: qty,
                timestamp: ts,
                order_side: self.queue_side,
            });
            // construct new queue
            let amended_queue = BinaryHeap::from(active_orders);
            self.idx_queue = Some(amended_queue);
        }
    }

    /// Return ID of current order in queue
    fn get_current_order_id(&self) -> Option<u64> {
        let order_id = self.idx_queue.as_ref()?.peek()?;
        Some(order_id.id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use near_sdk::{testing_env, AccountId, Gas, MockedBlockchain, VMContext};

    #[derive(Debug, Eq, PartialEq)]
    struct TestOrder {
        pub name: &'static str,
    }

    fn carol() -> AccountId {
        "carol.near".to_string()
    }
    fn alice() -> AccountId {
        "alice.near".to_string()
    }
    fn bob() -> AccountId {
        "bob.near".to_string()
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

    fn get_queue_empty(side: OrderSide) -> OrderQueue<TestOrder> {
        OrderQueue::new(side, 5, 10)
    }

    fn get_current_time() -> u64 {
        use std::time::SystemTime;
        let duration_since_epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let timestamp_nanos = duration_since_epoch.as_nanos(); // u128
        timestamp_nanos as u64
    }

    fn get_queue_bids() -> OrderQueue<TestOrder> {
        let mut bid_queue = get_queue_empty(OrderSide::Bid);

        assert!(bid_queue.insert(
            1,
            1.01,
            2,
            get_current_time(),
            TestOrder { name: "low bid" },
        ));
        assert!(bid_queue.insert(
            2,
            1.02,
            1,
            get_current_time(),
            TestOrder {
                name: "high bid first"
            },
        ));
        // same price but later
        assert!(bid_queue.insert(
            3,
            1.02,
            2,
            get_current_time(),
            TestOrder {
                name: "high bid second"
            },
        ));
        let first_bid = bid_queue.peek();
        assert_eq!(first_bid.unwrap().name, "high bid first");

        bid_queue
    }

    fn get_queue_asks() -> OrderQueue<TestOrder> {
        let mut ask_queue = get_queue_empty(OrderSide::Ask);
        assert!(ask_queue.insert(
            1,
            1.01,
            1,
            get_current_time(),
            TestOrder {
                name: "low ask first"
            },
        ));
        assert!(ask_queue.insert(
            2,
            1.02,
            1,
            get_current_time(),
            TestOrder { name: "high ask" },
        ));
        assert!(ask_queue.insert(
            3,
            1.01,
            2,
            get_current_time(),
            TestOrder {
                name: "low ask second"
            },
        ));
        print!("{0}", ask_queue.peek().unwrap().name);
        assert_eq!(ask_queue.peek().unwrap().name, "low ask first");

        ask_queue
    }

    #[test]
    fn queue_index_cmp() {
        let index1 = OrderIndex {
            id: 1,
            price: 1.01,
            timestamp: get_current_time(),
            quantity: 2,
            order_side: OrderSide::Ask,
        };

        let index2 = OrderIndex {
            id: 3,
            price: 1.01,
            timestamp: get_current_time(),
            quantity: 3,
            order_side: OrderSide::Ask,
        };

        assert_eq!(index1.cmp(&index2), Ordering::Greater);

        let mut bid_queue = get_queue_empty(OrderSide::Ask);
        bid_queue.insert(
            index1.id,
            index1.price,
            index1.quantity,
            index1.timestamp,
            TestOrder {
                name: "high bid first",
            },
        );
        bid_queue.insert(
            index2.id,
            index2.price,
            index2.quantity,
            index2.timestamp,
            TestOrder {
                name: "high bid second",
            },
        );
        bid_queue.insert(
            4,
            1.5,
            index2.quantity,
            get_current_time(),
            TestOrder {
                name: "low bid first",
            },
        );
        bid_queue.insert(
            5,
            1.1,
            index2.quantity,
            get_current_time(),
            TestOrder { name: "low bid 11" },
        );
        assert_eq!(bid_queue.peek().unwrap().name, "high bid first");
    }

    #[test]
    fn queue_operations_insert_unique() {
        testing_env!(get_context(carol()));

        let mut bid_queue = get_queue_empty(OrderSide::Bid);
        assert_eq!(bid_queue.peek(), None);

        // insert unique
        assert!(bid_queue.insert(
            1,
            1.01,
            2,
            get_current_time(),
            TestOrder { name: "first bid" },
        ));

        // discard order with existing ID
        assert!(!bid_queue.insert(
            1,
            1.02,
            5,
            get_current_time(),
            TestOrder {
                name: "another first bid"
            },
        ));
    }

    #[test]
    fn queue_operations_ordering_bid() {
        testing_env!(get_context(carol()));
        let mut bid_queue = get_queue_bids();

        assert_eq!(bid_queue.pop().unwrap().name, "high bid first");
        assert_eq!(bid_queue.pop().unwrap().name, "high bid second");
        assert_eq!(bid_queue.pop().unwrap().name, "low bid");
    }

    #[test]
    fn queue_operations_ordering_ask() {
        //testing_env!(get_context(carol()));
        let mut ask_queue = get_queue_asks();

        assert_eq!(ask_queue.pop().unwrap().name, "low ask first");
        assert_eq!(ask_queue.pop().unwrap().name, "low ask second");
        assert_eq!(ask_queue.pop().unwrap().name, "high ask");
    }

    #[test]
    fn queue_operations_modify_order() {
        testing_env!(get_context(carol()));
        let mut bid_queue = get_queue_bids();

        assert!(bid_queue.modify_current_order(TestOrder {
            name: "current bid partially matched"
        },));

        assert_eq!(
            bid_queue.pop().unwrap().name,
            "current bid partially matched"
        );
        assert_eq!(bid_queue.pop().unwrap().name, "high bid second");
        assert_eq!(bid_queue.pop().unwrap().name, "low bid");
    }

    #[test]
    fn queue_operations_amend() {
        testing_env!(get_context(carol()));
        let mut ask_queue = get_queue_asks();

        // amend two orders in the queue
        assert!(ask_queue.amend(
            2,
            0.99,
            3,
            get_current_time(),
            TestOrder { name: "new first" },
        ));
        assert!(ask_queue.amend(
            1,
            1.01,
            1,
            get_current_time(),
            TestOrder { name: "new last" },
        ));
        // non-exist order
        assert!(!ask_queue.amend(
            4,
            3.03,
            2,
            get_current_time(),
            TestOrder {
                name: "nonexistent"
            },
        ));

        assert_eq!(ask_queue.pop().unwrap().name, "new first");
        assert_eq!(ask_queue.pop().unwrap().name, "low ask second");
        assert_eq!(ask_queue.pop().unwrap().name, "new last");
    }

    #[test]
    fn queue_operations_cancel_order1() {
        testing_env!(get_context(carol()));
        let mut bid_queue = get_queue_bids();

        bid_queue.cancel(2);

        assert_eq!(bid_queue.pop().unwrap().name, "high bid second");
        assert_eq!(bid_queue.pop().unwrap().name, "low bid");
    }

    #[test]
    fn queue_operations_cancel_order2() {
        testing_env!(get_context(carol()));
        let mut ask_queue = get_queue_asks();

        ask_queue.cancel(3);

        assert_eq!(ask_queue.pop().unwrap().name, "low ask first");
        assert_eq!(ask_queue.pop().unwrap().name, "high ask");
    }
}
