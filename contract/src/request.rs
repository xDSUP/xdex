use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Serialize};
use near_sdk::{AccountId, Balance, Timestamp};

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, Serialize, PartialEq)]
pub enum RequestStatus{
    REJECTED,
    APPROVED,
    PENDING
}

pub type RequestId = u64;

/// Metadata on the individual token level.
#[derive(Clone, BorshDeserialize, BorshSerialize, Debug, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Vote {
    pub owner_id: AccountId,
    pub result: bool
}

impl PartialEq for Vote{
    fn eq(&self, other: &Self) -> bool {
        self.owner_id == other.owner_id
    }
}

/// Metadata on the individual token level.
#[derive(Clone, BorshDeserialize, BorshSerialize, Debug, Serialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct Request {
    pub id: RequestId,
    pub token_id: String,
    pub title: String,
    pub description: String,

    pub price: Balance,
    pub supply: Balance,

    pub status: RequestStatus,
    pub created_time: Timestamp,
    pub owner_id: AccountId,
    // полный текст заявки
    pub hash: String,
}
