use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
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
