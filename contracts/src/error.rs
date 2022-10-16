use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug, Error)]
#[serde(crate = "near_sdk::serde")]
pub enum MetaDaoError {
    #[error("Invalid Admin call")]
    InvalidAdminCall,
    #[error("Unable to create a new epoch, while previous epoch is still ongoing")]
    UnableToCreatNewEpoch,
    #[error("Creator is not registered for current epoch")]
    CreatorIsNotRegistered,
    #[error("Invalid current epoch")]
    InvalidCurrentEpoch,
}
