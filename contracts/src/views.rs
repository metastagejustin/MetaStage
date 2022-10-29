use near_contract_standards::fungible_token::{core::ext_ft_core, receiver::FungibleTokenReceiver};
use near_sdk::{env, ext_contract, near_bindgen, Promise, PromiseOrValue};

use crate::{error::MetaDaoError, *};

#[near_bindgen]
impl MetaDaoContract {
    #[handle_result]
    pub fn creator_got_enough_funds(
        &self,
        creator_id: CreatorAccountId,
    ) -> Result<bool, MetaDaoError> {
        let creator_funds = self
            .creator_obtained_complete_funding
            .get(&self.epoch)
            .ok_or(MetaDaoError::CreatorIsNotRegistered)?;
        Ok(!creator_funds.get(&creator_id).is_none())
    }

    // #[handle_result]
    // pub fn user_is_registered(&self, user_id: UserAccountId) -> Result<bool, MetaDaoError> {
    //     let votes_mapping = self
    //         .user_votes_mapping
    //         .get(&self.epoch)
    //         .ok_or(MetaDaoError::InvalidCurrentEpoch)?;
    //     Ok(votes_mapping.contains_key(&user_id))
    // }
}
