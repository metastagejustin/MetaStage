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
                    "MetadaoContract::ft_contract: Funding is not currently open for epoch {}",
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
            .expect("MetaDaoContract::ft_on_transfer: Invalid epoch");

        let creator_metadata = creators_metadata.get(&creator_account_id).expect(
            "MetaDaoContract::ft_on_transfer: Invalid creator account id for current epoch",
        );

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
        match self.user_funding_creator(
            sender_id,
            creator_account_id,
            user_nft_rank,
            amount,
            ft_token_id,
        ) {
            Err(_) => PromiseOrValue::Value(U128(amount)),
            _ => PromiseOrValue::Value(U128(0)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::CREATOR_REGISTRY_STORAGE_COST;
    use crate::tests::get_registry_metadata;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{
        testing_env, AccountId, Gas, MockedBlockchain, PromiseResult, RuntimeFeesConfig, VMConfig,
        VMContext,
    };
    use std::collections::HashMap;
    use std::convert::TryInto;

    /// utility function for testing callbacks logic
    #[allow(dead_code)]
    pub fn testing_env_with_promise_results(
        context: VMContext,
        promise_results: Vec<PromiseResult>,
    ) {
        near_sdk::env::set_blockchain_interface(MockedBlockchain::new(
            context,
            VMConfig::test(),
            RuntimeFeesConfig::test(),
            promise_results,
            Default::default(),
            Default::default(),
            None,
        ));
    }

    pub fn to_yocto(value: &str) -> u128 {
        let vals: Vec<_> = value.split('.').collect();
        let part1 = vals[0].parse::<u128>().unwrap() * 10u128.pow(24);
        if vals.len() > 1 {
            let power = vals[1].len() as u32;
            let part2 = vals[1].parse::<u128>().unwrap() * 10u128.pow(24 - power);
            part1 + part2
        } else {
            part1
        }
    }

    fn get_context_with_storage(storage: u128) -> VMContext {
        let contract_account_id: AccountId = "conliq.testnet".to_string().try_into().unwrap();

        let account: AccountId = "wrap.near".to_string().try_into().unwrap();

        VMContextBuilder::new()
            .current_account_id(contract_account_id)
            .attached_deposit(to_yocto("1000"))
            .signer_account_id(account.clone())
            .predecessor_account_id(account)
            .prepaid_gas(Gas(300 * 10u64.pow(16)))
            .attached_deposit(storage)
            .build()
    }

    #[test]
    fn test_ft_on_transfer_is_successful() {
        let account: AccountId = "wrap.near".to_string().try_into().unwrap();
        let admin = account.clone();

        let storage = (CREATOR_REGISTRY_STORAGE_COST as u128) * env::STORAGE_PRICE_PER_BYTE;

        let context = get_context_with_storage(storage);
        testing_env!(context.clone());

        let sender_id = accounts(2);
        let amount = U128(1_000_000_000_u128);
        let msg = format!("{}_common", account.clone(),);

        let mut contract = MetaDaoContract::new(admin);

        let mut protocol_tokens_fees = HashMap::new();

        protocol_tokens_fees.insert("wrap.near".to_string().try_into().unwrap(), 0.001);
        protocol_tokens_fees.insert("usn".to_string().try_into().unwrap(), 0.0005);

        contract
            .create_new_epoch(Some(protocol_tokens_fees))
            .unwrap();

        contract.set_registration().unwrap();

        let metadata = get_registry_metadata();

        contract.creator_registration(metadata).unwrap();

        assert!(contract
            .creators_per_epoch_set
            .get(&Epoch(1u16))
            .unwrap()
            .contains(&account));

        contract.set_funding().unwrap();

        let amount = contract.ft_on_transfer(sender_id, amount, msg.to_string());
        assert!(matches!(amount, PromiseOrValue::Value(U128(0_u128))));
    }

    #[test]
    #[should_panic(
        expected = "MetadaoContract::ft_contract: Funding is not currently open for epoch 1"
    )]
    fn test_ft_on_transfer_panics_if_not_in_funding() {
        let account: AccountId = "wrap.near".to_string().try_into().unwrap();
        let admin = account.clone();

        let storage = (CREATOR_REGISTRY_STORAGE_COST as u128) * env::STORAGE_PRICE_PER_BYTE;

        let context = get_context_with_storage(storage);
        testing_env!(context.clone());

        let sender_id = accounts(2);
        let amount = U128(1_000_000_000_u128);
        let msg = format!("{}_common", account.clone(),);

        let mut contract = MetaDaoContract::new(admin);

        let mut protocol_tokens_fees = HashMap::new();

        protocol_tokens_fees.insert("wrap.near".to_string().try_into().unwrap(), 0.001);
        protocol_tokens_fees.insert("usn".to_string().try_into().unwrap(), 0.0005);

        contract
            .create_new_epoch(Some(protocol_tokens_fees))
            .unwrap();

        contract.set_registration().unwrap();

        let metadata = get_registry_metadata();

        contract.creator_registration(metadata).unwrap();

        assert!(contract
            .creators_per_epoch_set
            .get(&Epoch(1u16))
            .unwrap()
            .contains(&account));

        contract.ft_on_transfer(sender_id, amount, msg.to_string());
    }

    #[test]
    #[should_panic(
        expected = "MetaDaoContract::ft_on_transfer: failed to parse creator account id"
    )]
    fn test_ft_on_transfer_panics_if_cannot_parse_account_id() {
        let account: AccountId = "wrap.near".to_string().try_into().unwrap();
        let admin = account.clone();

        let storage = (CREATOR_REGISTRY_STORAGE_COST as u128) * env::STORAGE_PRICE_PER_BYTE;

        let context = get_context_with_storage(storage);
        testing_env!(context.clone());

        let sender_id = accounts(2);
        let amount = U128(1_000_000_000_u128);
        let msg = format!("-/lj_common",);

        let mut contract = MetaDaoContract::new(admin);

        let mut protocol_tokens_fees = HashMap::new();

        protocol_tokens_fees.insert("wrap.near".to_string().try_into().unwrap(), 0.001);
        protocol_tokens_fees.insert("usn".to_string().try_into().unwrap(), 0.0005);

        contract
            .create_new_epoch(Some(protocol_tokens_fees))
            .unwrap();

        contract.set_registration().unwrap();

        let metadata = get_registry_metadata();

        contract.creator_registration(metadata).unwrap();

        assert!(contract
            .creators_per_epoch_set
            .get(&Epoch(1u16))
            .unwrap()
            .contains(&account));

        contract.set_funding().unwrap();

        contract.ft_on_transfer(sender_id, amount, msg.to_string());
    }

    #[test]
    #[should_panic(
        expected = "MetaDaoContract::ft_on_transfer: Invalid creator account id for current epoch"
    )]
    fn test_ft_on_transfer_panics_if_invalid_creator_account_id() {
        let account: AccountId = "wrap.near".to_string().try_into().unwrap();
        let admin = account.clone();

        let storage = (CREATOR_REGISTRY_STORAGE_COST as u128) * env::STORAGE_PRICE_PER_BYTE;

        let context = get_context_with_storage(storage);
        testing_env!(context.clone());

        let sender_id = accounts(2);
        let amount = U128(1_000_000_000_u128);
        let msg = format!("lotus_common",);

        let mut contract = MetaDaoContract::new(admin);

        let mut protocol_tokens_fees = HashMap::new();

        protocol_tokens_fees.insert("wrap.near".to_string().try_into().unwrap(), 0.001);
        protocol_tokens_fees.insert("usn".to_string().try_into().unwrap(), 0.0005);

        contract
            .create_new_epoch(Some(protocol_tokens_fees))
            .unwrap();

        contract.set_registration().unwrap();

        let metadata = get_registry_metadata();

        contract.creator_registration(metadata).unwrap();

        assert!(contract
            .creators_per_epoch_set
            .get(&Epoch(1u16))
            .unwrap()
            .contains(&account));

        contract.set_funding().unwrap();

        contract.ft_on_transfer(sender_id, amount, msg.to_string());
    }

    #[test]
    #[should_panic(expected = "MetaDaoContract::ft_on_transfer: Invalid epoch")]
    fn test_ft_on_transfer_panics_if_invalid_epoch() {
        let account: AccountId = "wrap.near".to_string().try_into().unwrap();
        let admin = account.clone();

        let storage = (CREATOR_REGISTRY_STORAGE_COST as u128) * env::STORAGE_PRICE_PER_BYTE;

        let context = get_context_with_storage(storage);
        testing_env!(context.clone());

        let sender_id = accounts(2);
        let amount = U128(1_000_000_000_u128);
        let msg = format!("lotus_common",);

        let mut contract = MetaDaoContract::new(admin);

        let mut protocol_tokens_fees = HashMap::new();

        protocol_tokens_fees.insert("wrap.near".to_string().try_into().unwrap(), 0.001);
        protocol_tokens_fees.insert("usn".to_string().try_into().unwrap(), 0.0005);

        contract
            .create_new_epoch(Some(protocol_tokens_fees))
            .unwrap();

        contract.set_registration().unwrap();

        let metadata = get_registry_metadata();

        contract.creator_registration(metadata).unwrap();

        assert!(contract
            .creators_per_epoch_set
            .get(&Epoch(1u16))
            .unwrap()
            .contains(&account));

        contract.set_funding().unwrap();

        contract.epoch = Epoch(0_u16);

        contract.ft_on_transfer(sender_id, amount, msg.to_string());
    }

    #[test]
    #[should_panic(
        expected = "MetaDaoContract::ft_on_transfer: Invalid creator account id for current epoch"
    )]
    fn test_ft_on_transfer_panics_if_invalid_ft_account_id() {
        let account: AccountId = "usn".to_string().try_into().unwrap();
        let admin = "wrap.near".to_string().try_into().unwrap();

        let storage = (CREATOR_REGISTRY_STORAGE_COST as u128) * env::STORAGE_PRICE_PER_BYTE;

        let context = get_context_with_storage(storage);
        testing_env!(context.clone());

        let sender_id = accounts(2);
        let amount = U128(1_000_000_000_u128);
        let msg = format!("{}_common", account);

        let mut contract = MetaDaoContract::new(admin);

        let mut protocol_tokens_fees = HashMap::new();

        protocol_tokens_fees.insert("wrap.near".to_string().try_into().unwrap(), 0.001);
        protocol_tokens_fees.insert("usn".to_string().try_into().unwrap(), 0.0005);

        contract
            .create_new_epoch(Some(protocol_tokens_fees))
            .unwrap();

        contract.set_registration().unwrap();

        let metadata = get_registry_metadata();

        contract.creator_registration(metadata).unwrap();

        contract.set_funding().unwrap();

        contract.ft_on_transfer(sender_id, amount, msg.to_string());
    }

    #[test]
    #[should_panic(
        expected = "ft_on_transfer::User did not provide enough funds to obtain the chosen NFT"
    )]
    fn test_ft_on_transfer_panics_if_sender_did_not_provide_enough_amount() {
        let account: AccountId = "wrap.near".to_string().try_into().unwrap();
        let admin = "wrap.near".to_string().try_into().unwrap();

        let storage = (CREATOR_REGISTRY_STORAGE_COST as u128) * env::STORAGE_PRICE_PER_BYTE;

        let context = get_context_with_storage(storage);
        testing_env!(context.clone());

        let sender_id = accounts(2);
        let amount = U128(1_u128);
        let msg = format!("{}_common", account);

        let mut contract = MetaDaoContract::new(admin);

        let mut protocol_tokens_fees = HashMap::new();

        protocol_tokens_fees.insert("wrap.near".to_string().try_into().unwrap(), 0.001);
        protocol_tokens_fees.insert("usn".to_string().try_into().unwrap(), 0.0005);

        contract
            .create_new_epoch(Some(protocol_tokens_fees))
            .unwrap();

        contract.set_registration().unwrap();

        let metadata = get_registry_metadata();

        contract.creator_registration(metadata).unwrap();

        contract.set_funding().unwrap();

        contract.ft_on_transfer(sender_id, amount, msg.to_string());
    }
}
