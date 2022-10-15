use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(
    BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug, Error
)]
#[serde(crate = "near_sdk::serde")]
pub enum MetaDaoError {

}
