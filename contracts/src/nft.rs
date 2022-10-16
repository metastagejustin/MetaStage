use near_contract_standards::non_fungible_token::{
    metadata::{NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata},
    Token, TokenId,
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, Promise, PromiseOrValue};

use crate::TokenAmount;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum NFTRanking {
    Common(Vec<TokenAmount>),
    Uncommon(Vec<TokenAmount>),
    Rare(Vec<TokenAmount>),
}

pub fn get_metadata(
    rarity: NFTRanking,
    creator_id: AccountId,
    copies: u64,
    description: String,
    title: String,
) -> TokenMetadata {
    TokenMetadata {
        title: Some(title),
        description: Some(description),
        media: None,
        media_hash: None,
        copies: Some(copies),
        issued_at: None,
        expires_at: None,
        starts_at: None,
        updated_at: None,
        extra: None,
        reference: None,
        reference_hash: None,
    }
}
