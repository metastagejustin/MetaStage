use near_contract_standards::non_fungible_token::{
    metadata::{NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata},
    Token, TokenId,
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, Promise, PromiseOrValue};
use std::collections::HashMap;

use crate::{consts::NFT_RANKING, error::MetaDaoError, FTAccountId};

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum UserNFTRank {
    Common,
    Uncommon,
    Rare,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum CreatorNFTRanking {
    Common(HashMap<FTAccountId, u128>),
    Uncommon(HashMap<FTAccountId, u128>),
    Rare(HashMap<FTAccountId, u128>),
}

impl CreatorNFTRanking {
    pub fn get_amount_from_nft_rank(
        &self,
        ft_token_id: &FTAccountId,
    ) -> Result<u128, MetaDaoError> {
        let inner = match self {
            Self::Common(i) => i,
            Self::Uncommon(i) => i,
            Self::Rare(i) => i,
        };

        let amount = *inner
            .get(ft_token_id)
            .ok_or(MetaDaoError::InvalidFTTokenId)?;
        Ok(amount)
    }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CreatorNFTRankings {
    rankings: Vec<CreatorNFTRanking>,
}

impl CreatorNFTRankings {
    pub fn get_ranking(&self, user_nft_rank: UserNFTRank) -> CreatorNFTRanking {
        match user_nft_rank {
            UserNFTRank::Common => self.rankings[0].clone(),
            UserNFTRank::Uncommon => self.rankings[1].clone(),
            UserNFTRank::Rare => self.rankings[2].clone(),
        }
    }
}

pub fn get_metadata(
    rarity: CreatorNFTRanking,
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
