use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};
use std::collections::HashMap;
use std::hash::Hash;

use crate::error::MetaDaoError;

mod error;
mod token_receiver;
mod nft;
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
    pub user_votes_mapping: LookupMap<Epoch, HashMap<UserAccountId, CreatorAccountId>>,
    pub creator_votes_mapping: LookupMap<Epoch, HashMap<CreatorAccountId, Vec<UserAccountId>>>,
    pub creator_funds: LookupMap<Epoch, HashMap<CreatorAccountId, Vec<TokenAmount>>>,
    pub user_funding: LookupMap<Epoch, HashMap<UserAccountId, TokenAmount>>,
    pub is_epoch_on: bool,
    pub creator_obtained_funds: LookupMap<Epoch, HashMap<UserAccountId, bool>>,
}

#[near_bindgen]
impl MetaDaoContract {
    #[init]
    pub fn new(admin: AccountId) -> Self {
        let user_votes_mapping =
            LookupMap::<Epoch, HashMap<UserAccountId, CreatorAccountId>>::new(b"a".to_vec());
        let creator_votes_mapping =
            LookupMap::<Epoch, HashMap<CreatorAccountId, Vec<UserAccountId>>>::new(b"b".to_vec());

        let creator_funds =
            LookupMap::<Epoch, HashMap<CreatorAccountId, Vec<TokenAmount>>>::new(b"c".to_vec());

        let user_funding =
            LookupMap::<Epoch, HashMap<UserAccountId, TokenAmount>>::new(b"d".to_vec());

        let creator_obtained_funds =
            LookupMap::<Epoch, HashMap<CreatorAccountId, bool>>::new(b"e".to_vec());

        Self {
            admin,
            epoch: Epoch(0u16),
            user_votes_mapping,
            creator_votes_mapping,
            creator_funds,
            user_funding,
            is_epoch_on: false,
            creator_obtained_funds,
        }
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
            &HashMap::<UserAccountId, CreatorAccountId>::new(),
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
}
