use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};
use near_units::parse_near;
use std::collections::HashMap;
use std::hash::Hash;

use crate::{
    consts::NFT_RANKING,
    error::MetaDaoError,
    nft::{CreatorNFTRanking, CreatorNFTRankings, UserNFTRank},
};

mod consts;
mod error;
mod nft;
mod token_receiver;
mod views;

pub type CreatorAccountId = AccountId;
pub type UserAccountId = AccountId;
pub type FTAccountId = AccountId;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Epoch(u16);

impl Epoch {
    fn count(&self) -> u16 {
        self.0
    }

    fn next_epoch(&self) -> Epoch {
        Self(self.0 + 1)
    }

    fn previous_epoch(&self) -> Option<Epoch> {
        match self.0.checked_sub(1) {
            None => None,
            Some(val) => Some(Epoch(val)),
        }
    }

    fn update_epoch(&mut self) {
        self.0 += 1
    }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct FundedTokenAmount {
    pub creator_id: CreatorAccountId,
    pub ft_token_id: FTAccountId,
    pub amount: u128,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ObtainedTokenAmounts {
    pub user_id: UserAccountId,
    pub ft_token_id: FTAccountId,
    pub amount: u128,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MetaDaoContract {
    pub admin: AccountId, // TODO: can we do it without an admin ?
    pub epoch: Epoch,
    pub creator_funding:
        UnorderedMap<Epoch, UnorderedMap<CreatorAccountId, Vec<ObtainedTokenAmounts>>>,
    pub user_funds: UnorderedMap<Epoch, UnorderedMap<UserAccountId, Vec<FundedTokenAmount>>>,
    pub creator_obtained_complete_funding: LookupMap<Epoch, HashMap<UserAccountId, bool>>,
    pub creator_per_epoch_set: UnorderedMap<Epoch, UnorderedSet<CreatorAccountId>>,
    pub creator_nft_ranks: UnorderedMap<Epoch, UnorderedMap<CreatorAccountId, CreatorNFTRankings>>,
    pub allowed_fungible_tokens_funding: UnorderedMap<Epoch, UnorderedSet<FTAccountId>>,
    pub is_epoch_on: bool,
    pub in_minting: bool,
    pub in_funding: bool,
}

#[near_bindgen]
impl MetaDaoContract {
    #[init]
    pub fn new(admin: AccountId) -> Self {
        let creator_funding = UnorderedMap::<
            Epoch,
            UnorderedMap<CreatorAccountId, Vec<ObtainedTokenAmounts>>,
        >::new(b"a".to_vec());
        let user_funds =
            UnorderedMap::<Epoch, UnorderedMap<UserAccountId, Vec<FundedTokenAmount>>>::new(
                b"b".to_vec(),
            );

        let creator_obtained_complete_funding =
            LookupMap::<Epoch, HashMap<CreatorAccountId, bool>>::new(b"e".to_vec());

        let creator_per_epoch_set =
            UnorderedMap::<Epoch, UnorderedSet<CreatorAccountId>>::new(b"f".to_vec());

        let creator_nft_ranks = UnorderedMap::<
            Epoch,
            UnorderedMap<CreatorAccountId, CreatorNFTRankings>,
        >::new(b"h".to_vec());

        let allowed_fungible_tokens_funding =
            UnorderedMap::<Epoch, UnorderedSet<FTAccountId>>::new(b"g".to_vec());

        Self {
            admin,
            epoch: Epoch(0u16),
            creator_funding,
            user_funds,
            creator_obtained_complete_funding,
            is_epoch_on: false,
            in_minting: false,
            in_funding: false,
            creator_per_epoch_set,
            creator_nft_ranks,
            allowed_fungible_tokens_funding,
        }
    }

    #[handle_result]
    fn set_funding(&mut self) -> Result<(), MetaDaoError> {
        if env::predecessor_account_id() != self.admin {
            return Err(MetaDaoError::InvalidAdminCall);
        }

        if self.in_funding {
            return Err(MetaDaoError::AlreadyInFunding);
        }

        if !self.in_minting {
            return Err(MetaDaoError::NotInMintingPeriod);
        }

        self.in_minting = false;
        self.in_funding = true;

        Ok(())
    }

    #[handle_result]
    fn set_minting(&mut self) -> Result<(), MetaDaoError> {
        if env::predecessor_account_id() != self.admin {
            return Err(MetaDaoError::InvalidAdminCall);
        }

        if self.in_minting {
            return Err(MetaDaoError::AlreadyInMinting);
        }

        self.in_minting = true;

        Ok(())
    }

    #[handle_result]
    fn create_new_epoch(
        &mut self,
        allowed_ft_account_ids: Option<Vec<FTAccountId>>,
    ) -> Result<(), MetaDaoError> {
        if env::predecessor_account_id() != self.admin {
            return Err(MetaDaoError::InvalidAdminCall);
        }

        if self.is_epoch_on {
            return Err(MetaDaoError::UnableToCreateNewEpoch);
        }

        // update epoch
        self.epoch.update_epoch();

        // create new entries for other contract fields, for new epoch
        self.user_funds.insert(
            &self.epoch,
            &UnorderedMap::<UserAccountId, Vec<FundedTokenAmount>>::new(
                format!("user_funds for epoch: {}", self.epoch.count())
                    .as_bytes()
                    .to_vec(),
            ),
        );
        self.creator_funding.insert(
            &self.epoch,
            &UnorderedMap::<CreatorAccountId, Vec<ObtainedTokenAmounts>>::new(
                format!("creator_funding for epoch: {}", self.epoch.count())
                    .as_bytes()
                    .to_vec(),
            ),
        );

        let mut allowed_ft_acc_ids = UnorderedSet::<FTAccountId>::new(
            format!("allowed_ft_acc_ids for epoch: {}", self.epoch.count())
                .as_bytes()
                .to_vec(),
        );

        if let Some(ft_acc_ids) = allowed_ft_account_ids {
            for ft_acc_id in &ft_acc_ids {
                allowed_ft_acc_ids.insert(ft_acc_id);
            }
        } else {
            let previous_allowed_ft_acc_ids = self
                .allowed_fungible_tokens_funding
                .get(
                    &self
                        .epoch
                        .previous_epoch()
                        .ok_or(MetaDaoError::InvalidInitializationOfEpoch)?,
                )
                .ok_or(MetaDaoError::InvalidCurrentEpoch)?;

            for ft_acc_id in previous_allowed_ft_acc_ids.iter() {
                allowed_ft_acc_ids.insert(&ft_acc_id);
            }
        }
        self.allowed_fungible_tokens_funding
            .insert(&self.epoch, &allowed_ft_acc_ids);

        self.is_epoch_on = true;

        Ok(())
    }

    #[handle_result]
    fn end_epoch(&mut self) -> Result<(), MetaDaoError> {
        if env::predecessor_account_id() != self.admin {
            return Err(MetaDaoError::InvalidAdminCall);
        }

        if self.is_epoch_on {
            return Err(MetaDaoError::EpochIsOff);
        }

        self.is_epoch_on = false;
        self.in_funding = false;
        self.in_minting = false;

        Ok(())
    }

    #[payable]
    #[private]
    #[handle_result]
    fn user_funding_creator(
        &mut self,
        user_id: UserAccountId,
        creator_account_id: CreatorAccountId,
        nft_rank: UserNFTRank,
        ft_token_id: FTAccountId,
    ) -> Result<(), MetaDaoError> {
        if env::attached_deposit() < parse_near!("0.01 N") {
            return Err(MetaDaoError::UserDidNotAttachEnoughFunds);
        }

        if !self.is_epoch_on {
            return Err(MetaDaoError::EpochIsOff);
        }

        if !self.in_funding {
            return Err(MetaDaoError::NotInFundingPeriod);
        }

        if self.creator_per_epoch_set.get(&self.epoch).is_none() {
            return Err(MetaDaoError::CreatorIsNotRegistered);
        }

        let user_id = env::predecessor_account_id();

        let nft_rankings = self
            .creator_nft_ranks
            .get(&self.epoch)
            .ok_or(MetaDaoError::InvalidCurrentEpoch)?;
        let creator_nft_ranks = nft_rankings
            .get(&creator_account_id)
            .ok_or(MetaDaoError::CreatorIsNotRegistered)?;

        let amount = creator_nft_ranks
            .get_ranking(nft_rank)
            .get_amount_from_nft_rank(&ft_token_id)?;

        let funded_token_amount = FundedTokenAmount {
            creator_id: creator_account_id.clone(),
            ft_token_id: ft_token_id.clone(),
            amount,
        };

        let mut user_funds = self
            .user_funds
            .get(&self.epoch)
            .ok_or(MetaDaoError::InvalidCurrentEpoch)?;

        match user_funds.get(&user_id) {
            None => {
                user_funds.insert(&user_id, &vec![funded_token_amount]);
            }
            Some(mut funds) => {
                funds.push(funded_token_amount);
                user_funds.insert(&user_id, &funds);
            }
        }

        self.user_funds.insert(&self.epoch, &user_funds);

        let obtained_token_amount = ObtainedTokenAmounts {
            user_id,
            ft_token_id: ft_token_id,
            amount,
        };

        let mut creator_fundings = self
            .creator_funding
            .get(&self.epoch)
            .ok_or(MetaDaoError::InvalidCurrentEpoch)?;

        match creator_fundings.get(&creator_account_id) {
            None => {
                let amounts = vec![obtained_token_amount];
                creator_fundings.insert(&creator_account_id, &amounts);
            }
            Some(mut amounts) => {
                amounts.push(obtained_token_amount);
                creator_fundings.insert(&creator_account_id, &amounts);
            }
        }

        self.creator_funding.insert(&self.epoch, &creator_fundings);

        Ok(())
    }
}
