use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata,
};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{
    serde::{Deserialize, Serialize},
    AccountId, Promise, PromiseOrValue,
};
use std::collections::HashMap;

use crate::{error::MetaDaoError, FTAccountId, *};

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
pub enum CreatorNFTTitle {
    Common(String),
    Uncommon(String),
    Rare(String),
}

impl CreatorNFTTitle {
    pub fn get_title(&self) -> &str {
        match self {
            Self::Common(t) | Self::Uncommon(t) | Self::Rare(t) => t,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum CreatorNFTDescription {
    Common(String),
    Uncommon(String),
    Rare(String),
}

impl CreatorNFTDescription {
    pub fn get_description(&self) -> &str {
        match self {
            Self::Common(t) | Self::Uncommon(t) | Self::Rare(t) => t,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum CreatorNFTMedia {
    Common(String),
    Uncommon(String),
    Rare(String),
}

impl CreatorNFTMedia {
    pub fn get_media(&self) -> &str {
        match self {
            Self::Common(t) | Self::Uncommon(t) | Self::Rare(t) => t,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum CreatorNFTCopies {
    Common(u64),
    Uncommon(u64),
    Rare(u64),
}

impl CreatorNFTCopies {
    pub fn get_copies(&self) -> u64 {
        match self {
            Self::Common(u) | Self::Uncommon(u) | Self::Rare(u) => *u,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum CreatorNFTExtra {
    Common(String),
    Uncommon(String),
    Rare(String),
}

impl CreatorNFTExtra {
    pub fn get_extra(&self) -> &str {
        match self {
            Self::Common(t) | Self::Uncommon(t) | Self::Rare(t) => t,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum CreatorNFTReference {
    Common(Option<String>),
    Uncommon(Option<String>),
    Rare(Option<String>),
}

impl CreatorNFTReference {
    pub fn get_reference(&self) -> Option<String> {
        match self {
            Self::Common(t) | Self::Uncommon(t) | Self::Rare(t) => t.clone(),
        }
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

#[near_bindgen]
impl MetaDaoContract {
    #[payable]
    #[private]
    pub fn nft_mint(
        &mut self,
        token_id: TokenId,
        receiver_id: AccountId,
        token_metadata: TokenMetadata,
    ) -> Token {
        self.tokens
            .internal_mint(token_id, receiver_id, Some(token_metadata))
    }
}

#[near_bindgen]
impl MetaDaoContract {
    pub fn get_token_id(
        &self,
        user_id: &UserAccountId,
        nft_rank: &UserNFTRank,
        creator_metadata: &CreatorMetadata,
    ) -> TokenId {
        let nft_id = self.nft_id;
        let titles = creator_metadata.get_titles();
        let title = match nft_rank {
            UserNFTRank::Common => titles[0].get_title(),
            UserNFTRank::Uncommon => titles[1].get_title(),
            UserNFTRank::Rare => titles[2].get_title(),
        };
        let nft_rank = match nft_rank {
            UserNFTRank::Common => "common",
            UserNFTRank::Uncommon => "uncommon",
            UserNFTRank::Rare => "rare",
        };

        format!("MetaDao|{}|{}|{}|{}|", nft_id, title, nft_rank, user_id)
    }

    #[payable]
    #[handle_result]
    pub fn mint_nfts_for_users(
        &mut self,
        creator_account_id: CreatorAccountId,
    ) -> Result<(), MetaDaoError> {
        let creators_metadata = self
            .creators_metadata
            .get(&self.epoch)
            .ok_or(MetaDaoError::InvalidCurrentEpoch)?;

        let creator_metadata = creators_metadata
            .get(&creator_account_id)
            .ok_or(MetaDaoError::CreatorIsNotRegistered)?;

        let creators_obtained_funds = self
            .creator_funding
            .get(&self.epoch)
            .ok_or(MetaDaoError::InvalidCurrentEpoch)?;

        let creator_obtained_funds = creators_obtained_funds
            .get(&creator_account_id)
            .ok_or(MetaDaoError::CreatorIsNotRegistered)?;

        for ObtainedTokenAmounts {
            user_id,
            ft_token_id,
            amount,
            nft_rank,
        } in creator_obtained_funds
        {
            let token_id = self.get_token_id(&user_id, &nft_rank, &creator_metadata);
        }

        Ok(())
    }
}

impl NonFungibleTokenMetadataProvider for MetaDaoContract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

// Implements the Near NFT interface for the MetaDaoContract
near_contract_standards::impl_non_fungible_token_approval!(MetaDaoContract, tokens);
near_contract_standards::impl_non_fungible_token_core!(MetaDaoContract, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(MetaDaoContract, tokens);

#[cfg(test)]
mod tests {}
