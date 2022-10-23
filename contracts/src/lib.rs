use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};
use std::collections::HashMap;
use std::hash::Hash;

use crate::error::MetaDaoError;

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

    fn next_epoch(&mut self) {
        self.0 += 1
    }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenAmount {
    pub token_id: FTAccountId,
    pub amount: u128,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MetaDaoContract {
    pub admin: AccountId, // TODO: can we do it without an admin ?
    pub epoch: Epoch,
    pub user_votes_mapping: LookupMap<Epoch, HashMap<UserAccountId, Vec<CreatorAccountId>>>,
    pub creator_votes_mapping: LookupMap<Epoch, HashMap<CreatorAccountId, Vec<UserAccountId>>>,
    pub creator_funds: LookupMap<Epoch, HashMap<CreatorAccountId, Vec<TokenAmount>>>,
    pub user_funding: LookupMap<Epoch, HashMap<UserAccountId, TokenAmount>>,
    pub creator_obtained_funds: LookupMap<Epoch, HashMap<UserAccountId, bool>>,
    pub creator_per_epoch_set: UnorderedMap<Epoch, UnorderedSet<CreatorAccountId>>,
    pub is_epoch_on: bool,
    pub in_minting: bool,
    pub in_funding: bool,
}

#[near_bindgen]
impl MetaDaoContract {
    #[init]
    pub fn new(admin: AccountId) -> Self {
        let user_votes_mapping =
            LookupMap::<Epoch, HashMap<UserAccountId, Vec<CreatorAccountId>>>::new(b"a".to_vec());
        let creator_votes_mapping =
            LookupMap::<Epoch, HashMap<CreatorAccountId, Vec<UserAccountId>>>::new(b"b".to_vec());

        let creator_funds =
            LookupMap::<Epoch, HashMap<CreatorAccountId, Vec<TokenAmount>>>::new(b"c".to_vec());

        let user_funding =
            LookupMap::<Epoch, HashMap<UserAccountId, TokenAmount>>::new(b"d".to_vec());

        let creator_obtained_funds =
            LookupMap::<Epoch, HashMap<CreatorAccountId, bool>>::new(b"e".to_vec());

        let creator_per_epoch_set = UnorderedMap::<Epoch, UnorderedSet<CreatorAccountId>>::new(b"f".to_vec());

        Self {
            admin,
            epoch: Epoch(0u16),
            user_votes_mapping,
            creator_votes_mapping,
            creator_funds,
            user_funding,
            creator_obtained_funds,
            is_epoch_on: false,
            in_minting: false,
            in_funding: false,
            creator_per_epoch_set,
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
    }

    #[handle_result]
    fn create_new_epoch(&mut self) -> Result<(), MetaDaoError> {
        if env::predecessor_account_id() != self.admin {
            return Err(MetaDaoError::InvalidAdminCall);
        }

        if self.is_epoch_on {
            return Err(MetaDaoError::UnableToCreatNewEpoch);
        }

        // update epoch
        self.epoch.next_epoch();

        // create new entries for other contract fields, for new epoch
        self.user_votes_mapping.insert(
            &self.epoch,
            &HashMap::<UserAccountId, Vec<CreatorAccountId>>::new(),
        );
        self.creator_votes_mapping.insert(
            &self.epoch,
            &HashMap::<CreatorAccountId, Vec<UserAccountId>>::new(),
        );
        self.creator_funds.insert(
            &self.epoch,
            &HashMap::<CreatorAccountId, Vec<TokenAmount>>::new(),
        );
        self.user_funding
            .insert(&self.epoch, &HashMap::<UserAccountId, TokenAmount>::new());
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
    }

    #[payable]
    #[private]
    #[handle_result]
    fn register_user(&mut self, creator_account_id: CreatorAccountId, nft_rank: NFTRanking) -> Result<(), MetaDaoError> {
        // TODO: refactor this
        if env::attached_deposit() < 1 {
            return Err(MetadataoError::UserDidNotAttachEnoughFunds);
        }

        if !self.is_epoch_on {
            return Err(MetaDaoError::EpochIsOff);
        }

        if !self.in_funding {
            return Err(MetaDaoError::NotInFundingPeriod);
        }

        if !self.creator_per_epoch_set.contains(&self.epoch) {
            return Err(MetaDaoError::CreatorIsNotRegistered);
        }

        let user_id = env::predecessor_account_id();

        let mut creator_votes_mapping = self.creator_votes_mapping.get(&self.epoch).ok_or(MetaDaoError::InvalidCurrentEpoch)?;
        let mut creator_votes = creator_votes_mapping.get(&creator_account_id);

        match creator_votes {
            None => {
                let users = vec![user_id];
                creator_votes_mapping.insert(creator_account_id.clone(), users);
                self.creator_votes_mapping.insert(&self.epoch, &creator_votes_mapping);
            },
            Some(cv) => {
                if creator_votes.contains(user_id.clone()) {
                    return Err(MetaDaoError::UserAlreadyRegisteredFundsToCreator);
                }
                users.push(user_id.clone());
                creator_votes.insert(user_id, users);
                self.creator_votes_mapping.insert(&self.epoch, &creator_votes_mapping);
            }
        }

        let mut user_votes_mapping = self.user_votes_mapping.get(&self.epoch).ok_or(MetaDaoError::InvalidCurrentEpoch)?;
        let mut user_votes = user_votes_mapping.get(&user_id);

        match user_votes {
            None => {
                let creators = vec![creator_account_id.clone()];
                user_votes_mapping.insert(user_id.clone(), creators);
                self.user_votes_mapping.insert(&self.epoch, user_votes_mapping);
            },
            Some(uv) => {
                if uv.contains(creator_id.clone()) {
                    return Err(MetaDaoError::UserAlreadyRegisteredFundsToCreator);
                }
                uv.push(creator_id.clone());
                user_votes_mapping.insert(user_id.clone(), uv);
                self.user_votes_mapping.insert(&self.epoch, &user_votes_mapping);
            }
        }

    }
}
