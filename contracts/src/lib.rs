use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{near_bindgen, AccountId, PanicOnDefault};
use std::collections::HashMap;

pub type CreatorAccountId = AccountId;
pub type UserAccountId = AccountId;
pub type FTAccountId = AccountId;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Epoch(u16);

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenAmount {
    pub token_id: FTAccountId,
    pub amount: u128,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MetaDaoContract {
    pub epoch: Epoch,
    pub user_votes_mapping: LookupMap<Epoch, HashMap<UserAccountId, CreatorAccountId>>,
    pub creator_votes_mapping: LookupMap<Epoch, HashMap<CreatorAccountId, Vec<UserAccountId>>>,
    pub creator_funds: LookupMap<Epoch, HashMap<CreatorAccountId, Vec<TokenAmount>>>,
}

#[near_bindgen]
impl MetaDaoContract {
    #[init]
    pub fn new() -> Self {
        let user_votes_mapping =
            LookupMap::<Epoch, HashMap<UserAccountId, CreatorAccountId>>::new(b"a".to_vec());
        let creator_votes_mapping =
            LookupMap::<Epoch, HashMap<CreatorAccountId, Vec<UserAccountId>>>::new(b"b".to_vec());

        let creator_funds =
            LookupMap::<Epoch, HashMap<CreatorAccountId, Vec<TokenAmount>>>::new(b"c".to_vec());

        Self {
            epoch: Epoch(0u16),
            user_votes_mapping,
            creator_votes_mapping,
            creator_funds,
        }
    }
}
