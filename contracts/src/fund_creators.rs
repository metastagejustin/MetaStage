use crate::{consts::GAS_FOR_FT_TRANSFER, *};
use near_contract_standards::fungible_token::core::ext_ft_core;
use near_sdk::json_types::U128;
use near_sdk::Promise;

#[near_bindgen]
impl MetaDaoContract {
    #[payable]
    #[private]
    pub fn external_send_ft_tokens(
        &mut self,
        creator_account_id: CreatorAccountId,
        ft_account_id: FTAccountId,
        amount: u128,
    ) -> Promise {
        ext_ft_core::ext(ft_account_id)
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .with_attached_deposit(1)
            .ft_transfer(creator_account_id, U128(amount), None)
    }
}
