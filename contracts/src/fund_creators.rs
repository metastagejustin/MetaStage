use crate::{consts::GAS_FOR_FT_TRANSFER, *};
use near_contract_standards::fungible_token::core::ext_ft_core;
use near_sdk::json_types::U128;
use near_sdk::{env, Promise};

#[near_bindgen]
impl MetaDaoContract {
    #[private]
    pub fn on_external_send_ft_tokens_callback(
        &mut self,
        creator_account_id: &CreatorAccountId,
        user_id: &UserAccountId,
    ) {
        if env::promise_results_count() != 1 {
            env::panic_str("MetaDaoContract::external_send_ft_tokens::Invalid promise result count, one should only have one promise result");
        }

        let mut creator_fundings = self
            .creator_funding
            .get(&self.epoch)
            .ok_or(MetaDaoError::InvalidCurrentEpoch)
            .expect("MetaDaoContract::external_send_ft_tokens::Invalid current epoch id");

        let creator_funding = creator_fundings
            .get(creator_account_id)
            .ok_or(MetaDaoError::CreatorIsNotRegistered)
            .expect("MetaDaoContract::external_send_ft_tokens::Creator is not registered");

        let creator_funding = creator_funding
            .iter()
            .map(|ot| {
                if ot.user_id == *user_id {
                    ObtainedTokenAmounts {
                        user_id: ot.user_id.clone(),
                        already_funded: true,
                        amount: ot.amount,
                        nft_rank: ot.nft_rank.clone(),
                        ft_token_id: ot.ft_token_id.clone(),
                    }
                } else {
                    ot.clone()
                }
            })
            .collect::<Vec<_>>();

        creator_fundings.insert(creator_account_id, &creator_funding);
        self.creator_funding.insert(&self.epoch, &creator_fundings);
    }
}

#[near_bindgen]
impl MetaDaoContract {
    #[payable]
    #[private]
    pub fn external_send_ft_tokens(
        &mut self,
        creator_account_id: CreatorAccountId,
        user_id: &UserAccountId,
        ft_account_id: FTAccountId,
        amount: u128,
    ) -> Promise {
        ext_ft_core::ext(ft_account_id)
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .with_attached_deposit(1)
            .ft_transfer(creator_account_id.clone(), U128(amount), None)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_FT_TRANSFER)
                    .on_external_send_ft_tokens_callback(&creator_account_id, user_id),
            )
    }
}
