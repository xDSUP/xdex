use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Balance};
use crate::token::TokenMetadata;

/// Metadata on the individual token level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
struct Request{
    id: u8,
    title: String,
    description: String,
    price: Balance,
    token: TokenMetadata,
    owner_id: AccountId,
}



