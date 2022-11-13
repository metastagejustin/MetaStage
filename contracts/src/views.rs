use near_sdk::near_bindgen;

use crate::{error::MetaDaoError, *};

#[near_bindgen]
impl MetaDaoContract {
    #[handle_result]
    pub fn creator_total_funds(&self, creator_id: CreatorAccountId) -> Result<u128, MetaDaoError> {
        let creator_funds_map = self
            .creator_funding
            .get(&self.epoch)
            .ok_or(MetaDaoError::CreatorIsNotRegistered)?;

        if let Some(funds) = creator_funds_map.get(&creator_id) {
            Ok(funds.iter().rfold(0u128, |a, b| a + b.amount))
        } else {
            Ok(0u128)
        }
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

#[cfg(test)]
mod tests {}
