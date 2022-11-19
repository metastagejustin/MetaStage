use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde::{Deserialize, Serialize};

use crate::consts::CREATOR_REGISTRY_STORAGE_COST;
use crate::{
    error::MetaDaoError,
    nft::{
        CreatorNFTCopies, CreatorNFTDescription, CreatorNFTExtra, CreatorNFTMedia,
        CreatorNFTRanking, CreatorNFTReference, CreatorNFTTitle,
    },
    *,
};

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CreatorMetadata {
    nft_ranks: Vec<CreatorNFTRanking>,
    titles: Vec<CreatorNFTTitle>,
    descriptions: Vec<CreatorNFTDescription>,
    medias: Vec<CreatorNFTMedia>,
    copies: Vec<CreatorNFTCopies>,
    extras: Vec<CreatorNFTExtra>,
    references: Vec<CreatorNFTReference>,
}

impl CreatorMetadata {
    pub fn nft_rank(&self, user_nft_rank: UserNFTRank) -> CreatorNFTRanking {
        match user_nft_rank {
            UserNFTRank::Common => self.nft_ranks[0].clone(),
            UserNFTRank::Uncommon => self.nft_ranks[1].clone(),
            UserNFTRank::Rare => self.nft_ranks[2].clone(),
        }
    }

    pub fn get_nft_ranks(&self) -> Vec<CreatorNFTRanking> {
        self.nft_ranks.clone()
    }

    pub fn get_titles(&self) -> Vec<CreatorNFTTitle> {
        self.titles.clone()
    }

    pub fn get_descriptions(&self) -> Vec<CreatorNFTDescription> {
        self.descriptions.clone()
    }

    pub fn get_medias(&self) -> Vec<CreatorNFTMedia> {
        self.medias.clone()
    }

    pub fn get_copies(&self) -> Vec<CreatorNFTCopies> {
        self.copies.clone()
    }

    pub fn get_extras(&self) -> Vec<CreatorNFTExtra> {
        self.extras.clone()
    }

    pub fn get_references(&self) -> Vec<CreatorNFTReference> {
        self.references.clone()
    }

    pub fn get_token_metadata(&self, nft_rank: UserNFTRank) -> Result<TokenMetadata, MetaDaoError> {
        let index = match nft_rank {
            UserNFTRank::Common => 0_usize,
            UserNFTRank::Uncommon => 1_usize,
            UserNFTRank::Rare => 2_usize,
        };

        let title = Some(String::from(self.get_titles()[index].get_title()));
        let description = Some(String::from(
            self.get_descriptions()[index].get_description(),
        ));
        let media = Some(String::from(self.get_medias()[index].get_media()));
        let media_hash = env::sha256(media.clone().unwrap().as_bytes());
        let media_hash = Some(Base64VecU8::from(media_hash));
        let copies = Some(self.get_copies()[index].get_copies());
        let issued_at = Some(format!("block_timestamp: {}", env::block_timestamp()));
        let extra = Some(String::from(self.get_extras()[index].get_extra()));
        let reference = self.get_references()[index].get_reference();
        let reference_hash = if reference.is_none() {
            Some(Base64VecU8::from(env::sha256(
                reference.clone().unwrap().as_bytes(),
            )))
        } else {
            None
        };

        let token_metadata = TokenMetadata {
            title,
            description,
            media,
            media_hash,
            copies,
            issued_at,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra,
            reference,
            reference_hash,
        };

        Ok(token_metadata)
    }
}

#[near_bindgen]
impl MetaDaoContract {
    #[payable]
    #[handle_result]
    pub fn creator_registration(&mut self, metadata: CreatorMetadata) -> Result<(), MetaDaoError> {
        let creator_account_id = env::predecessor_account_id();

        if env::attached_deposit()
            < (CREATOR_REGISTRY_STORAGE_COST as u128) * env::STORAGE_PRICE_PER_BYTE
        {
            return Err(MetaDaoError::UncoveredStorageCosts);
        }

        if !self.is_epoch_on {
            return Err(MetaDaoError::EpochIsOff);
        }

        if !self.in_registration {
            return Err(MetaDaoError::NotInRegistrationPeriod);
        }

        let mut creator_funding = self
            .creator_funding
            .get(&self.epoch)
            .ok_or(MetaDaoError::EpochIsOff)?;

        let mut creators_metadata = self
            .creators_metadata
            .get(&self.epoch)
            .ok_or(MetaDaoError::EpochIsOff)?;

        let mut creators_per_epoch = self
            .creators_per_epoch_set
            .get(&self.epoch)
            .ok_or(MetaDaoError::EpochIsOff)?;

        creator_funding.insert(&creator_account_id, &vec![]);
        creators_metadata.insert(&creator_account_id, &metadata);
        creators_per_epoch.insert(&creator_account_id);

        self.creator_funding.insert(&self.epoch, &creator_funding);
        self.creators_metadata
            .insert(&self.epoch, &creators_metadata);
        self.creators_per_epoch_set
            .insert(&self.epoch, &creators_per_epoch);

        Ok(())
    }
}
