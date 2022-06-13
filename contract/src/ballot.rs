use near_sdk::{AccountId, Balance, env, Timestamp};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Serialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};

use orderbook::{new_sequence_gen, TradeSequence};

use crate::{Request, RequestId, Token, Vote};
use crate::request::RequestStatus;

/// Metadata on the individual token level.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct BallotHandler {
    pub requests: UnorderedMap<RequestId, Request>,
    pub votes: LookupMap<RequestId, UnorderedSet<Vote>>,
    seq: TradeSequence,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct StakeInfo {
    pub staked: Balance,
    pub created_time: Timestamp
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
pub struct LaunchPad {
    pub price: Balance,
    /// продано
    pub sell_supply: u128,
    pub launched_time: Timestamp,
    pub token: Token
}


#[derive(Serialize)]
pub struct UserRequest {
    pub token_id: String,
    pub title: String,
    pub description: String,

    pub price: Balance,
    pub supply: Balance,

    // полный текст заявки
    pub hash: String,
}

impl BallotHandler {
    pub fn new() -> Self {
        let handler = BallotHandler {
            requests: UnorderedMap::new(b"req".to_vec()),
            votes: LookupMap::new(b"v".to_vec()),
            seq: new_sequence_gen(0, 1000000),
        };

        handler
    }

    pub fn add_new_request(&mut self, request: UserRequest) {
        let request = Request {
            id: self.seq.next_id(),
            owner_id: env::predecessor_account_id(),
            status: RequestStatus::PENDING,

            token_id: request.token_id,
            title: request.title,
            description: request.description,
            
            price: request.price,
            supply: request.supply,
            hash: request.hash,
            created_time: env::block_timestamp(),
        };
        self.requests.insert(&request.id, &request);
    }

    pub fn get_all_requests(&self) -> Vec<Request> {
        let mut result: Vec<Request> = Vec::new();
        for request in self.requests.values() {
            result.push(request);
        }
        result
    }

    pub fn get_request(&self, request_id: RequestId) -> Option<Request> {
        self.requests.get(&request_id)
    }

    fn get_votes(&self, request_id: &RequestId) -> UnorderedSet<Vote>{
        self.votes.get(request_id).unwrap_or(UnorderedSet::new(b"vot".to_vec()))
    }

    pub fn get_all_votes(&self, request_id: RequestId) -> Vec<Vote>{
        self.get_votes(&request_id).to_vec()
    }

    pub fn is_vote(&self, request_id: RequestId, voter_id: AccountId) -> bool{
        let votes = self.get_votes(&request_id);

        votes.contains(&Vote{
            owner_id: voter_id.clone(),
            result: true
        }) || votes.contains(&Vote{
            owner_id: voter_id,
            result: false
        })
    }

    pub fn vote(&mut self, request_id: RequestId, vote: bool){
        let mut votes = self.get_votes(&request_id);

        votes.insert(&Vote{
            owner_id: env::predecessor_account_id(),
            result: vote
        });

        self.votes.insert(&request_id, &votes);
    }

    pub fn reject_request(&mut self, request_id: RequestId){
        self.update_request_status(&request_id, RequestStatus::REJECTED);
        self.votes.remove(&request_id);
    }

    pub fn approve_request(&mut self, request_id: RequestId){
        self.update_request_status(&request_id, RequestStatus::APPROVED);
        self.votes.remove(&request_id);
    }

    fn update_request_status(&mut self, request_id: &RequestId, new_status: RequestStatus) {
        let mut request = self.requests.get(&request_id).unwrap();
        request.status = new_status;
        self.requests.insert(&request_id, &request);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use near_sdk::{AccountId, Gas, testing_env, VMContext, MockedBlockchain};
    use crate::ballot::UserRequest;
    use crate::{BallotHandler, Request, Vote};
    use crate::request::RequestStatus;

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

    fn test_user_request() -> UserRequest {
        UserRequest{
            token_id: "XDHO".to_string(),
            title: "Тестовое название".to_string(),
            description: "ПРИВЕТИКИ".to_string(),
            price: 23,
            supply: 100000,
            hash: "сосисочка".to_string()
        }
    }

    fn test_request() -> Request {
        Request{
            id: 0,
            token_id: "XDHO".to_string(),
            title: "Тестовое название".to_string(),
            description: "ПРИВЕТИКИ".to_string(),
            price: 23,
            supply: 100000,
            status: RequestStatus::PENDING,
            created_time: 0,
            owner_id: bob(),
            hash: "сосисочка".to_string(),
        }
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

    fn get_handler_with_requests() -> BallotHandler{
        let mut handler = BallotHandler::new();
        handler.add_new_request(test_user_request());
        handler
    }

    #[test]
    fn get_new_request(){
        testing_env!(get_context(bob()));
        let handler = get_handler_with_requests();
        assert_eq!(handler.get_all_requests()[0], test_request());
    }

    #[test]
    fn get_new_vote(){
        testing_env!(get_context(bob()));
        let mut handler = get_handler_with_requests();

        assert_eq!(handler.is_vote(0, bob()), false);

        handler.vote(0, true);
        assert_eq!( handler.get_all_votes(0)[0], Vote{ owner_id: bob(), result: true });
        assert_eq!(handler.is_vote(0, bob()), true);

        handler.vote(0, false);
        assert_eq!( handler.get_all_votes(0)[0], Vote{ owner_id: bob(), result: true });
        assert_eq!(handler.is_vote(0, bob()), true);
    }

    #[test]
    fn test_reject_request(){
        testing_env!(get_context(bob()));
        let mut handler = get_handler_with_requests();
        handler.vote(0, true);

        assert_eq!(handler.get_request(0).unwrap().status, RequestStatus::PENDING);
        handler.reject_request(0);
        assert_eq!(handler.get_request(0).unwrap().status, RequestStatus::REJECTED);
        assert_eq!( handler.get_all_votes(0).len(), 0);
    }

    #[test]
    fn test_approve_request(){
        testing_env!(get_context(bob()));
        let mut handler = get_handler_with_requests();
        handler.vote(0, true);

        assert_eq!(handler.get_request(0).unwrap().status, RequestStatus::PENDING);
        handler.approve_request(0);
        assert_eq!(handler.get_request(0).unwrap().status, RequestStatus::APPROVED);
        assert_eq!( handler.get_all_votes(0).len(), 0);
    }
}

