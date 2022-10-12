use near_sdk::{near_bindgen, AccountId,};
use near_sdk::collections::{LookupMap, Vector};

pub type CreatorAccountId = AccountId;
pub type UserAccountId = AccountId;

#[near_bindgen]
pub struct Contract {
    pub epoch: u16,
    pub user_votes_mapping: LookupMap<UserAccountId, CreatorAccountId>,
    pub creator_votes_mapping: LookupMap<CreatorAccountId, Vector<UserAccountId>>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        let user_votes_mapping = LookupMap::<UserAccountId, CreatorAccountId>::new(b"a");
        let creator_votes_mapping = LookupMap::<CreatorAccountId, Vector<UserAccountId>>::new(b"b");
        
        Self {
            epoch: 0u16,
            user_votes_mapping,
            creator_votes_mapping,
        }
    }
}