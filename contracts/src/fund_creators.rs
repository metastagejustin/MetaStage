use crate::{consts::GAS_FOR_FT_TRANSFER, *};
use near_contract_standards::fungible_token::core::ext_ft_core;
use near_sdk::json_types::U128;
use near_sdk::{env, Promise, PromiseResult};

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

        if !matches!(env::promise_result(0), PromiseResult::Successful(_)) {
            env::panic_str("MetaDaoContract::external_send_ft_tokens::Promise failed");
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

#[cfg(test)]
mod tests {
    use crate::tests::get_registry_metadata;

    use super::*;
    use near_sdk::{
        test_utils::{accounts, VMContextBuilder},
        testing_env, Gas, MockedBlockchain, PromiseResult, RuntimeFeesConfig, VMConfig, VMContext,
    };

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

        VMContextBuilder::new()
            .current_account_id(contract_account_id)
            .attached_deposit(to_yocto("1000"))
            .signer_account_id(accounts(1))
            .predecessor_account_id(accounts(1))
            .prepaid_gas(Gas(300 * 10u64.pow(16)))
            .attached_deposit(storage)
            .build()
    }

    #[test]
    fn test_on_external_send_ft_tokens_callback_works() {
        let admin: AccountId = accounts(1);
        let storage = parse_near!("0.1 N");

        let context = get_context_with_storage(storage);
        testing_env!(context.clone());

        testing_env_with_promise_results(context, vec![PromiseResult::Successful(vec![0u8, 1, 2])]);

        let mut contract = MetaDaoContract::new(admin.clone());

        let mut protocol_accounts_fees = HashMap::<FTAccountId, f64>::new();

        protocol_accounts_fees.insert("wrap.near".to_string().try_into().unwrap(), 0.05);
        protocol_accounts_fees.insert("usn".to_string().try_into().unwrap(), 0.03);

        contract
            .create_new_epoch(Some(protocol_accounts_fees))
            .unwrap();

        contract.set_registration().unwrap();
        let metadata = get_registry_metadata();

        contract.creator_registration(metadata).unwrap();

        contract.set_funding().unwrap();

        let user_id: AccountId = "user.near".to_string().try_into().unwrap();
        let creator_account_id: AccountId = accounts(1);

        let mut creator_fundings = contract.creator_funding.get(&contract.epoch).unwrap();
        creator_fundings.insert(
            &creator_account_id,
            &vec![ObtainedTokenAmounts {
                user_id: user_id.clone(),
                amount: 100_u128,
                nft_rank: UserNFTRank::Common,
                ft_token_id: "wrap.near".to_string().try_into().unwrap(),
                already_funded: false,
            }],
        );

        contract
            .creator_funding
            .insert(&contract.epoch, &creator_fundings);

        contract.on_external_send_ft_tokens_callback(&creator_account_id, &user_id);

        assert_eq!(
            contract
                .creator_funding
                .get(&Epoch(1))
                .unwrap()
                .get(&creator_account_id)
                .unwrap(),
            vec![ObtainedTokenAmounts {
                amount: 100_u128,
                user_id,
                nft_rank: UserNFTRank::Common,
                ft_token_id: "wrap.near".to_string().try_into().unwrap(),
                already_funded: true,
            }]
        );
    }

    #[test]
    #[should_panic(expected = "MetaDaoContract::external_send_ft_tokens::Promise failed")]
    fn test_on_external_send_ft_tokens_callback_fails_if_not_successful_promise() {
        let admin: AccountId = accounts(1);
        let storage = parse_near!("0.1 N");

        let context = get_context_with_storage(storage);
        testing_env!(context.clone());

        testing_env_with_promise_results(context, vec![PromiseResult::Failed]);

        let mut contract = MetaDaoContract::new(admin.clone());

        let mut protocol_accounts_fees = HashMap::<FTAccountId, f64>::new();

        protocol_accounts_fees.insert("wrap.near".to_string().try_into().unwrap(), 0.05);
        protocol_accounts_fees.insert("usn".to_string().try_into().unwrap(), 0.03);

        contract
            .create_new_epoch(Some(protocol_accounts_fees))
            .unwrap();

        contract.set_registration().unwrap();
        let metadata = get_registry_metadata();

        contract.creator_registration(metadata).unwrap();

        contract.set_funding().unwrap();

        let user_id: AccountId = "user.near".to_string().try_into().unwrap();
        let creator_account_id: AccountId = accounts(1);

        contract.on_external_send_ft_tokens_callback(&creator_account_id, &user_id);
    }

    #[test]
    #[should_panic(expected = "MetaDaoContract::external_send_ft_tokens::Promise failed")]
    fn test_on_external_send_ft_tokens_callback_fails_if_not_successful_promise2() {
        let admin: AccountId = accounts(1);
        let storage = parse_near!("0.1 N");

        let context = get_context_with_storage(storage);
        testing_env!(context.clone());

        testing_env_with_promise_results(context, vec![PromiseResult::NotReady]);

        let mut contract = MetaDaoContract::new(admin.clone());

        let mut protocol_accounts_fees = HashMap::<FTAccountId, f64>::new();

        protocol_accounts_fees.insert("wrap.near".to_string().try_into().unwrap(), 0.05);
        protocol_accounts_fees.insert("usn".to_string().try_into().unwrap(), 0.03);

        contract
            .create_new_epoch(Some(protocol_accounts_fees))
            .unwrap();

        contract.set_registration().unwrap();
        let metadata = get_registry_metadata();

        contract.creator_registration(metadata).unwrap();

        contract.set_funding().unwrap();

        let user_id: AccountId = "user.near".to_string().try_into().unwrap();
        let creator_account_id: AccountId = accounts(1);

        contract.on_external_send_ft_tokens_callback(&creator_account_id, &user_id);
    }

    #[test]
    #[should_panic(
        expected = "MetaDaoContract::external_send_ft_tokens::Invalid promise result count, one should only have one promise result"
    )]
    fn test_on_external_send_ft_tokens_callback_fails_if_more_than_one_promise() {
        let admin: AccountId = accounts(1);
        let storage = parse_near!("0.1 N");

        let context = get_context_with_storage(storage);
        testing_env!(context.clone());

        testing_env_with_promise_results(
            context,
            vec![
                PromiseResult::Successful(vec![0_u8, 1, 2]),
                PromiseResult::Successful(vec![0_u8, 1]),
            ],
        );

        let mut contract = MetaDaoContract::new(admin.clone());

        let mut protocol_accounts_fees = HashMap::<FTAccountId, f64>::new();

        protocol_accounts_fees.insert("wrap.near".to_string().try_into().unwrap(), 0.05);
        protocol_accounts_fees.insert("usn".to_string().try_into().unwrap(), 0.03);

        contract
            .create_new_epoch(Some(protocol_accounts_fees))
            .unwrap();

        contract.set_registration().unwrap();
        let metadata = get_registry_metadata();

        contract.creator_registration(metadata).unwrap();

        contract.set_funding().unwrap();

        let user_id: AccountId = "user.near".to_string().try_into().unwrap();
        let creator_account_id: AccountId = accounts(1);

        contract.on_external_send_ft_tokens_callback(&creator_account_id, &user_id);
    }

    #[test]
    #[should_panic(expected = "MetaDaoContract::external_send_ft_tokens::Invalid current epoch id")]
    fn test_on_external_send_ft_tokens_callback_fails_if_epoch_is_invalid() {
        let admin: AccountId = accounts(1);
        let storage = parse_near!("0.1 N");

        let context = get_context_with_storage(storage);
        testing_env!(context.clone());

        testing_env_with_promise_results(
            context,
            vec![PromiseResult::Successful(vec![0_u8, 1, 2])],
        );

        let mut contract = MetaDaoContract::new(admin.clone());

        let mut protocol_accounts_fees = HashMap::<FTAccountId, f64>::new();

        protocol_accounts_fees.insert("wrap.near".to_string().try_into().unwrap(), 0.05);
        protocol_accounts_fees.insert("usn".to_string().try_into().unwrap(), 0.03);

        contract
            .create_new_epoch(Some(protocol_accounts_fees))
            .unwrap();

        contract.set_registration().unwrap();
        let metadata = get_registry_metadata();

        contract.creator_registration(metadata).unwrap();

        contract.set_funding().unwrap();

        let user_id: AccountId = "user.near".to_string().try_into().unwrap();
        let creator_account_id: AccountId = accounts(1);

        contract.epoch = Epoch(0);

        contract.on_external_send_ft_tokens_callback(&creator_account_id, &user_id);
    }

    #[test]
    #[should_panic(
        expected = "MetaDaoContract::external_send_ft_tokens::Creator is not registered"
    )]
    fn test_on_external_send_ft_tokens_callback_fails_if_creator_is_not_registered() {
        let admin: AccountId = accounts(1);
        let storage = parse_near!("0.1 N");

        let context = get_context_with_storage(storage);
        testing_env!(context.clone());

        testing_env_with_promise_results(
            context,
            vec![PromiseResult::Successful(vec![0_u8, 1, 2])],
        );

        let mut contract = MetaDaoContract::new(admin.clone());

        let mut protocol_accounts_fees = HashMap::<FTAccountId, f64>::new();

        protocol_accounts_fees.insert("wrap.near".to_string().try_into().unwrap(), 0.05);
        protocol_accounts_fees.insert("usn".to_string().try_into().unwrap(), 0.03);

        contract
            .create_new_epoch(Some(protocol_accounts_fees))
            .unwrap();

        contract.set_registration().unwrap();
        let metadata = get_registry_metadata();

        contract.creator_registration(metadata).unwrap();

        contract.set_funding().unwrap();

        let user_id: AccountId = "user.near".to_string().try_into().unwrap();
        let creator_account_id: AccountId = "creator.near".to_string().try_into().unwrap();

        contract.on_external_send_ft_tokens_callback(&creator_account_id, &user_id);
    }
}
