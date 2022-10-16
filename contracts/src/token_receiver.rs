use near_contract_standards::fungible_token::{core::ext_ft_core, receiver::FungibleTokenReceiver};
use near_sdk::{env, ext_contract, near_bindgen, Promise, PromiseOrValue};

use crate::*;

#[near_bindgen]
impl MetaDaoContract {}

// #[near_bindgen]
// impl FungibleTokenReceiver for MetaDaoContract {
//     #[payable]
//     fn ft_on_transfer(
//         &mut self,
//         sender_id: AccountId,
//         amount: near_sdk::json_types::U128,
//         msg: String,
//     ) -> PromiseOrValue<near_sdk::json_types::U128> {
//     }
// }
