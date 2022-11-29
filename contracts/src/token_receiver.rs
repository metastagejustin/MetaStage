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

        let amount = amount.0;
        let epoch = self.epoch;

        let creators_metadata = self
            .creators_metadata
            .get(&epoch)
            .expect("ft_on_transfer::Invalid epoch");

        let creator_metadata = creators_metadata
            .get(&creator_account_id)
            .expect("ft_on_transfer::Invalid creator account id for current epoch");

        let user_nft_rank = match metadata[1] {
            "common" => UserNFTRank::Common,
            "uncommon" => UserNFTRank::Uncommon,
            "rare" => UserNFTRank::Rare,
            _ => return PromiseOrValue::Value(U128(amount)),
        };

        let min_fund_amount = creator_metadata
            .nft_rank(user_nft_rank.clone())
            .get_amount_from_nft_rank(&ft_token_id)
            .expect("ft_on_transfer::Invalid fungible token id");

        if amount < min_fund_amount {
            env::panic_str(
                "ft_on_transfer::User did not provide enough funds to obtain the chosen NFT",
            );
        }

        // TODO: 1. assert that the user sent enough funds to buy the NFTs
        // 2. return the value to the user, if transaction failed
        self.user_funding_creator(
            sender_id,
            creator_account_id,
            user_nft_rank,
            amount,
            ft_token_id,
        )
        .expect("MetaDaoContract::ft_on_transfer: user failed to fund creator");

        PromiseOrValue::Value(U128(0))
    }
}

#[cfg(test)]
mod tests {}
