use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, PromiseOrValue};

use crate::*;

#[near_bindgen]
impl MetaDaoContract {}

#[near_bindgen]
impl FungibleTokenReceiver for MetaDaoContract {
    #[payable]
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: near_sdk::json_types::U128,
        msg: String,
    ) -> PromiseOrValue<near_sdk::json_types::U128> {
        if !self.in_funding {
            env::panic_str(
                format!(
                    "ft_contract::ft_transfer_call: Funding is not currently open for epoch {}",
                    self.epoch.count()
                )
                .as_str(),
            );
        }

        let ft_token_id = env::predecessor_account_id();

        let metadata = msg.split('_').collect::<Vec<_>>();
        let creator_account_id = AccountId::try_from(metadata[0].to_string())
            .expect("MetaDaoContract::ft_on_transfer: failed to parse creator account id");
        let nft_rank = match metadata[1] {
            "common" => UserNFTRank::Common,
            "uncommon" => UserNFTRank::Uncommon,
            "rare" => UserNFTRank::Rare,
            _ => return PromiseOrValue::Value(amount),
        };

        // TODO: 1. assert that the user sent enough funds to buy the NFTs
        // 2. return the value to the user, if transaction failed
        self.user_funding_creator(sender_id, creator_account_id, nft_rank, ft_token_id)
            .expect("MetaDaoContract::ft_on_transfer: user failed to fund creator");

        PromiseOrValue::Value(U128(0))
    }
}

#[cfg(test)]
mod tests {}
