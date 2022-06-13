use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Balance, Timestamp};

pub type TokenId = String;

/// Metadata on the individual token level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    pub title: Option<String>, // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
    pub description: Option<String>, // free-form description
    pub icon: Option<String>, // free-form description
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Token {
    pub token_id: TokenId,
    pub owner_id: AccountId,
    pub supply: Balance,
    pub meta: Option<TokenMetadata>,

    /// запущен на вторичный рынок или нет
    pub launched: bool,
    /// время первых торгов
    pub launched_time: Timestamp,
}

impl PartialEq for Token{
    fn eq(&self, other: &Self) -> bool {
        self.token_id == other.token_id
    }
}
