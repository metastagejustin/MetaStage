use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};
use near_units::parse_near;
use registry::CreatorMetadata;

use crate::{
    error::MetaDaoError,
    nft::{CreatorNFTRankings, UserNFTRank},
};

mod consts;
mod error;
mod nft;
mod registry;
mod tests;
mod token_receiver;
mod views;

pub type CreatorAccountId = AccountId;
pub type UserAccountId = AccountId;
pub type FTAccountId = AccountId;

#[derive(
    BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug, Clone, Copy, Eq, PartialEq,
)]
#[serde(crate = "near_sdk::serde")]
pub struct Epoch(u16);

impl Epoch {
    fn count(&self) -> u16 {
        self.0
    }

    fn next_epoch(&self) -> Epoch {
        Self(self.0 + 1)
    }

    fn previous_epoch(&self) -> Option<Epoch> {
        self.0.checked_sub(1).map(Epoch)
    }

    fn update_epoch(&mut self) {
        self.0 += 1
    }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct FundedTokenAmount {
    pub creator_id: CreatorAccountId,
    pub ft_token_id: FTAccountId,
    pub amount: u128,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ObtainedTokenAmounts {
    pub user_id: UserAccountId,
    pub ft_token_id: FTAccountId,
    pub amount: u128,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MetaDaoContract {
    pub admin: AccountId, // TODO: can we do it without an admin ?
    pub epoch: Epoch,
    pub creator_funding:
        UnorderedMap<Epoch, UnorderedMap<CreatorAccountId, Vec<ObtainedTokenAmounts>>>,
    pub user_funds: UnorderedMap<Epoch, UnorderedMap<UserAccountId, Vec<FundedTokenAmount>>>,
    pub creators_per_epoch_set: UnorderedMap<Epoch, UnorderedSet<CreatorAccountId>>,
    pub creators_metadata: UnorderedMap<Epoch, UnorderedMap<CreatorAccountId, CreatorMetadata>>,
    pub allowed_fungible_tokens_funding: UnorderedMap<Epoch, UnorderedSet<FTAccountId>>,
    pub is_epoch_on: bool,
    pub in_registration: bool,
    pub in_funding: bool,
}

#[near_bindgen]
impl MetaDaoContract {
    #[init]
    pub fn new(admin: AccountId) -> Self {
        let creator_funding = UnorderedMap::<
            Epoch,
            UnorderedMap<CreatorAccountId, Vec<ObtainedTokenAmounts>>,
        >::new(b"a".to_vec());
        let user_funds =
            UnorderedMap::<Epoch, UnorderedMap<UserAccountId, Vec<FundedTokenAmount>>>::new(
                b"b".to_vec(),
            );

        let creators_per_epoch_set =
            UnorderedMap::<Epoch, UnorderedSet<CreatorAccountId>>::new(b"f".to_vec());

        let creators_metadata = UnorderedMap::<
            Epoch,
            UnorderedMap<CreatorAccountId, CreatorMetadata>,
        >::new(b"h".to_vec());

        let allowed_fungible_tokens_funding =
            UnorderedMap::<Epoch, UnorderedSet<FTAccountId>>::new(b"g".to_vec());

        Self {
            admin,
            epoch: Epoch(0u16),
            creator_funding,
            user_funds,
            is_epoch_on: false,
            in_registration: false,
            in_funding: false,
            creators_per_epoch_set,
            creators_metadata,
            allowed_fungible_tokens_funding,
        }
    }

    #[handle_result]
    fn set_funding(&mut self) -> Result<(), MetaDaoError> {
        if env::predecessor_account_id() != self.admin {
            return Err(MetaDaoError::InvalidAdminCall);
        }

        if !self.is_epoch_on {
            return Err(MetaDaoError::EpochIsOff);
        }

        if self.in_funding {
            return Err(MetaDaoError::AlreadyInFunding);
        }

        if !self.in_registration {
            return Err(MetaDaoError::NotInRegistrationPeriod);
        }

        self.in_registration = false;
        self.in_funding = true;

        Ok(())
    }

    #[handle_result]
    fn set_Registration(&mut self) -> Result<(), MetaDaoError> {
        if env::predecessor_account_id() != self.admin {
            return Err(MetaDaoError::InvalidAdminCall);
        }

        if !self.is_epoch_on {
            return Err(MetaDaoError::EpochIsOff);
        }

        if self.in_funding {
            return Err(MetaDaoError::AlreadyInFunding);
        }

        if self.in_registration {
            return Err(MetaDaoError::AlreadyInRegistration);
        }

        self.in_registration = true;

        Ok(())
    }

    #[handle_result]
    fn create_new_epoch(
        &mut self,
        allowed_ft_account_ids: Option<Vec<FTAccountId>>,
    ) -> Result<(), MetaDaoError> {
        if env::predecessor_account_id() != self.admin {
            return Err(MetaDaoError::InvalidAdminCall);
        }

        // it is enough to check this, as if epoch is set to false
        // Registration and funding should also be set to false
        if self.is_epoch_on {
            return Err(MetaDaoError::UnableToCreateNewEpoch);
        }

        // update epoch
        self.epoch.update_epoch();

        // create new entries for other contract fields, for new epoch
        self.user_funds.insert(
            &self.epoch,
            &UnorderedMap::<UserAccountId, Vec<FundedTokenAmount>>::new(
                format!("user_funds for epoch: {}", self.epoch.count())
                    .as_bytes()
                    .to_vec(),
            ),
        );
        self.creator_funding.insert(
            &self.epoch,
            &UnorderedMap::<CreatorAccountId, Vec<ObtainedTokenAmounts>>::new(
                format!("creator_funding for epoch: {}", self.epoch.count())
                    .as_bytes()
                    .to_vec(),
            ),
        );
        self.creators_metadata.insert(
            &self.epoch,
            &UnorderedMap::<CreatorAccountId, CreatorMetadata>::new(
                format!("creator nft rankings for epoch: {}", self.epoch.count())
                    .as_bytes()
                    .to_vec(),
            ),
        );
        self.creators_per_epoch_set.insert(
            &self.epoch,
            &UnorderedSet::<CreatorAccountId>::new(
                format!("Creator per epoch set for epoch: {}", self.epoch.count())
                    .as_bytes()
                    .to_vec(),
            ),
        );

        let mut allowed_ft_acc_ids = UnorderedSet::<FTAccountId>::new(
            format!("allowed_ft_acc_ids for epoch: {}", self.epoch.count())
                .as_bytes()
                .to_vec(),
        );

        if let Some(ft_acc_ids) = allowed_ft_account_ids {
            for ft_acc_id in &ft_acc_ids {
                allowed_ft_acc_ids.insert(ft_acc_id);
            }
        } else {
            let previous_allowed_ft_acc_ids = self
                .allowed_fungible_tokens_funding
                .get(
                    &self
                        .epoch
                        .previous_epoch()
                        .ok_or(MetaDaoError::InvalidInitializationOfEpoch)?,
                )
                .ok_or(MetaDaoError::InvalidCurrentEpoch)?;

            for ft_acc_id in previous_allowed_ft_acc_ids.iter() {
                allowed_ft_acc_ids.insert(&ft_acc_id);
            }
        }
        self.allowed_fungible_tokens_funding
            .insert(&self.epoch, &allowed_ft_acc_ids);

        self.is_epoch_on = true;

        Ok(())
    }

    #[handle_result]
    fn end_epoch(&mut self) -> Result<(), MetaDaoError> {
        if env::predecessor_account_id() != self.admin {
            return Err(MetaDaoError::InvalidAdminCall);
        }

        if !self.is_epoch_on {
            return Err(MetaDaoError::EpochIsOff);
        }

        if self.in_registration {
            return Err(MetaDaoError::AlreadyInRegistration);
        }

        if self.in_funding {
            return Err(MetaDaoError::AlreadyInFunding);
        }

        self.is_epoch_on = false;
        self.in_funding = false;
        self.in_registration = false;

        Ok(())
    }

    #[payable]
    #[private]
    #[handle_result]
    fn user_funding_creator(
        &mut self,
        user_id: UserAccountId,
        creator_account_id: CreatorAccountId,
        nft_rank: UserNFTRank,
        ft_token_id: FTAccountId,
    ) -> Result<(), MetaDaoError> {
        if env::attached_deposit() < parse_near!("0.01 N") {
            return Err(MetaDaoError::UserDidNotAttachEnoughFunds);
        }

        if !self.is_epoch_on {
            return Err(MetaDaoError::EpochIsOff);
        }

        if !self.in_funding {
            return Err(MetaDaoError::NotInFundingPeriod);
        }

        if self.creators_per_epoch_set.get(&self.epoch).is_none() {
            return Err(MetaDaoError::CreatorIsNotRegistered);
        }

        let creators_metadata = self
            .creators_metadata
            .get(&self.epoch)
            .ok_or(MetaDaoError::InvalidCurrentEpoch)?;
        let creator_metadata = creators_metadata
            .get(&creator_account_id)
            .ok_or(MetaDaoError::CreatorIsNotRegistered)?;

        let amount = creator_metadata
            .nft_rank(nft_rank)
            .get_amount_from_nft_rank(&ft_token_id)?;

        let funded_token_amount = FundedTokenAmount {
            creator_id: creator_account_id.clone(),
            ft_token_id: ft_token_id.clone(),
            amount,
        };

        let mut user_funds = self
            .user_funds
            .get(&self.epoch)
            .ok_or(MetaDaoError::InvalidCurrentEpoch)?;

        match user_funds.get(&user_id) {
            None => {
                user_funds.insert(&user_id, &vec![funded_token_amount]);
            }
            Some(mut funds) => {
                funds.push(funded_token_amount);
                user_funds.insert(&user_id, &funds);
            }
        }

        self.user_funds.insert(&self.epoch, &user_funds);

        let obtained_token_amount = ObtainedTokenAmounts {
            user_id,
            ft_token_id,
            amount,
        };

        let mut creator_fundings = self
            .creator_funding
            .get(&self.epoch)
            .ok_or(MetaDaoError::InvalidCurrentEpoch)?;

        match creator_fundings.get(&creator_account_id) {
            None => {
                let amounts = vec![obtained_token_amount];
                creator_fundings.insert(&creator_account_id, &amounts);
            }
            Some(mut amounts) => {
                amounts.push(obtained_token_amount);
                creator_fundings.insert(&creator_account_id, &amounts);
            }
        }

        self.creator_funding.insert(&self.epoch, &creator_fundings);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{
        testing_env, AccountId, Gas, MockedBlockchain, PromiseResult, RuntimeFeesConfig, VMConfig,
        VMContext,
    };
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
    fn test_new_works() {
        let admin: AccountId = "admin.near".to_string().try_into().unwrap();
        let contract = MetaDaoContract::new(admin.clone());

        assert_eq!(contract.epoch, Epoch(0u16));

        assert!(!contract.is_epoch_on);
        assert!(!contract.in_registration);
        assert!(!contract.in_funding);

        assert_eq!(contract.admin, admin);
        assert!(contract.creator_funding.is_empty());
        assert!(contract.user_funds.is_empty());

        assert!(contract.creators_per_epoch_set.is_empty());
        assert!(contract.creators_metadata.is_empty());

        assert!(contract.allowed_fungible_tokens_funding.is_empty());
    }

    #[test]
    fn test_set_funding_works() {
        let admin: AccountId = accounts(1);
        let storage = 1u128;

        let context = get_context_with_storage(storage);
        testing_env!(context);

        let mut contract = MetaDaoContract::new(admin.clone());

        contract.is_epoch_on = true;
        contract.in_registration = true;

        contract.set_funding().unwrap();

        assert!(contract.in_funding);
        assert!(!contract.in_registration);
        assert!(contract.is_epoch_on);
    }

    #[test]
    fn test_set_funding_fails_if_not_admin() {
        let admin: AccountId = "admin.near".to_string().try_into().unwrap();
        let storage = 1u128;

        let context = get_context_with_storage(storage);
        testing_env!(context);

        let mut contract = MetaDaoContract::new(admin.clone());

        contract.is_epoch_on = true;
        contract.in_registration = true;

        assert!(contract
            .set_funding()
            .unwrap_err()
            .to_string()
            .contains("Invalid Admin call"));
    }

    #[test]
    fn test_set_funding_fails_if_epoch_is_off() {
        let admin = accounts(1);
        let storage = 1u128;

        let context = get_context_with_storage(storage);
        testing_env!(context);

        let mut contract = MetaDaoContract::new(admin.clone());

        contract.is_epoch_on = false;
        contract.in_registration = true;

        assert!(contract
            .set_funding()
            .unwrap_err()
            .to_string()
            .contains("Currently, epoch is off"));
    }

    #[test]
    fn test_set_funding_fails_if_registration_off() {
        let admin = accounts(1);
        let storage = 1u128;

        let context = get_context_with_storage(storage);
        testing_env!(context);

        let mut contract = MetaDaoContract::new(admin.clone());

        contract.is_epoch_on = true;
        contract.in_registration = false;

        assert!(contract
            .set_funding()
            .unwrap_err()
            .to_string()
            .contains("Not in Registration period"));
    }

    #[test]
    fn test_set_funding_fails_if_already_in_funding() {
        let admin = accounts(1);
        let storage = 1u128;

        let context = get_context_with_storage(storage);
        testing_env!(context);

        let mut contract = MetaDaoContract::new(admin.clone());

        contract.is_epoch_on = true;
        contract.in_registration = true;
        contract.in_funding = true;

        assert!(contract
            .set_funding()
            .unwrap_err()
            .to_string()
            .contains("Already in funding period"));
    }

    #[test]
    fn test_set_registration_works() {
        let admin = accounts(1);
        let storage = 1u128;

        let context = get_context_with_storage(storage);
        testing_env!(context);

        let mut contract = MetaDaoContract::new(admin.clone());

        contract.is_epoch_on = true;

        contract.set_Registration().unwrap();

        assert!(contract.in_registration);
        assert!(contract.is_epoch_on);
        assert!(!contract.in_funding);
    }

    #[test]
    fn test_set_registration_fails_if_not_admin() {
        let admin: AccountId = "admin.near".to_string().try_into().unwrap();
        let storage = 1u128;

        let context = get_context_with_storage(storage);
        testing_env!(context);

        let mut contract = MetaDaoContract::new(admin.clone());

        contract.is_epoch_on = true;
        contract.in_registration = true;

        assert!(contract
            .set_Registration()
            .unwrap_err()
            .to_string()
            .contains("Invalid Admin call"));
    }

    #[test]
    fn test_set_registration_fails_if_epoch_is_off() {
        let admin = accounts(1);
        let storage = 1u128;

        let context = get_context_with_storage(storage);
        testing_env!(context);

        let mut contract = MetaDaoContract::new(admin.clone());

        contract.is_epoch_on = false;
        contract.in_registration = true;

        assert!(contract
            .set_Registration()
            .unwrap_err()
            .to_string()
            .contains("Currently, epoch is off"));
    }

    #[test]
    fn test_set_registration_fails_if_registration_on() {
        let admin = accounts(1);
        let storage = 1u128;

        let context = get_context_with_storage(storage);
        testing_env!(context);

        let mut contract = MetaDaoContract::new(admin.clone());

        contract.is_epoch_on = true;
        contract.in_registration = true;
        contract.in_funding = false;

        assert!(contract
            .set_Registration()
            .unwrap_err()
            .to_string()
            .contains("Already in Registration period"));
    }

    #[test]
    fn test_set_registration_fails_if_already_in_funding() {
        let admin = accounts(1);
        let storage = 1u128;

        let context = get_context_with_storage(storage);
        testing_env!(context);

        let mut contract = MetaDaoContract::new(admin.clone());

        contract.is_epoch_on = true;
        contract.in_registration = true;
        contract.in_funding = true;

        assert!(contract
            .set_Registration()
            .unwrap_err()
            .to_string()
            .contains("Already in funding period"));
    }

    #[test]
    fn test_end_epoch_works() {
        let admin = accounts(1);
        let storage = 1u128;

        let context = get_context_with_storage(storage);
        testing_env!(context);

        let mut contract = MetaDaoContract::new(admin.clone());

        contract.is_epoch_on = true;

        contract.end_epoch().unwrap();

        assert!(!contract.is_epoch_on);
        assert!(!contract.in_funding);
        assert!(!contract.in_registration);
    }

    #[test]
    fn test_end_epoch_fails_if_not_call_by_admin() {
        let admin: AccountId = "admin.near".to_string().try_into().unwrap();
        let storage = 1u128;

        let context = get_context_with_storage(storage);
        testing_env!(context);

        let mut contract = MetaDaoContract::new(admin.clone());

        assert!(contract
            .end_epoch()
            .unwrap_err()
            .to_string()
            .contains("Invalid Admin call"));
    }

    #[test]
    fn test_end_epoch_fails_if_epoch_off() {
        let admin: AccountId = accounts(1);
        let storage = 1u128;

        let context = get_context_with_storage(storage);
        testing_env!(context);

        let mut contract = MetaDaoContract::new(admin.clone());

        contract.is_epoch_on = false;

        assert!(contract
            .end_epoch()
            .unwrap_err()
            .to_string()
            .contains("Currently, epoch is off"));
    }

    #[test]
    fn test_end_epoch_fails_if_registration_on() {
        let admin: AccountId = accounts(1);
        let storage = 1u128;

        let context = get_context_with_storage(storage);
        testing_env!(context);

        let mut contract = MetaDaoContract::new(admin.clone());

        contract.is_epoch_on = true;
        contract.in_registration = true;

        assert!(contract
            .end_epoch()
            .unwrap_err()
            .to_string()
            .contains("Already in Registration"));
    }

    #[test]
    fn test_end_epoch_fails_if_funding_on() {
        let admin: AccountId = accounts(1);
        let storage = 1u128;

        let context = get_context_with_storage(storage);
        testing_env!(context);

        let mut contract = MetaDaoContract::new(admin.clone());

        contract.is_epoch_on = true;
        contract.in_registration = false;
        contract.in_funding = true;

        assert!(contract
            .end_epoch()
            .unwrap_err()
            .to_string()
            .contains("Already in funding"));
    }

    #[test]
    fn test_create_epoch_works() {
        let admin: AccountId = accounts(1);
        let storage = 1u128;

        let context = get_context_with_storage(storage);
        testing_env!(context);

        let mut contract = MetaDaoContract::new(admin.clone());
        let allowed_ft_accounts: Vec<AccountId> = vec![
            "wrap.near".to_string().try_into().unwrap(),
            "usn".to_string().try_into().unwrap(),
        ];

        contract
            .create_new_epoch(Some(allowed_ft_accounts))
            .unwrap();

        assert_eq!(contract.epoch, Epoch(1u16));
        assert!(contract.is_epoch_on);
        assert!(!contract.in_funding);
        assert!(!contract.in_registration);

        let epoch = Epoch(1u16);

        assert!(contract.creator_funding.get(&epoch).unwrap().is_empty());
        assert!(contract.creators_metadata.get(&epoch).unwrap().is_empty());
        assert!(contract.user_funds.get(&epoch).unwrap().is_empty());

        assert!(contract
            .creators_per_epoch_set
            .get(&epoch)
            .unwrap()
            .is_empty());
        assert_eq!(
            contract
                .allowed_fungible_tokens_funding
                .get(&epoch)
                .unwrap()
                .len(),
            2u64
        );
        assert!(contract
            .allowed_fungible_tokens_funding
            .get(&epoch)
            .unwrap()
            .contains(&AccountId::try_from(String::from("wrap.near")).unwrap()));
        assert!(contract
            .allowed_fungible_tokens_funding
            .get(&epoch)
            .unwrap()
            .contains(&AccountId::try_from(String::from("usn")).unwrap()));
    }

    #[test]
    fn test_create_new_epoch_fails_if_not_admin_call() {
        let admin: AccountId = "admin.near".to_string().try_into().unwrap();
        let storage = 1u128;

        let context = get_context_with_storage(storage);
        testing_env!(context);

        let mut contract = MetaDaoContract::new(admin.clone());
        let allowed_ft_accounts: Vec<AccountId> = vec![
            "wrap.near".to_string().try_into().unwrap(),
            "usn".to_string().try_into().unwrap(),
        ];

        contract
            .create_new_epoch(Some(allowed_ft_accounts))
            .unwrap_err()
            .to_string()
            .contains("Invalid Admin call");
    }

    #[test]
    fn test_create_new_epoch_fails_if_epoch_on() {
        let admin: AccountId = accounts(1);
        let storage = 1u128;

        let context = get_context_with_storage(storage);
        testing_env!(context);

        let mut contract = MetaDaoContract::new(admin.clone());
        let allowed_ft_accounts: Vec<AccountId> = vec![
            "wrap.near".to_string().try_into().unwrap(),
            "usn".to_string().try_into().unwrap(),
        ];

        contract.is_epoch_on = true;

        contract
            .create_new_epoch(Some(allowed_ft_accounts))
            .unwrap_err()
            .to_string()
            .contains("Unable to create a new epoch, while previous epoch is still ongoing");
    }

    #[test]
    fn test_user_funding_creator_works() {
        let admin: AccountId = accounts(1);
        let storage = parse_near!("0.01 N");

        let context = get_context_with_storage(storage);
        testing_env!(context);

        let mut contract = MetaDaoContract::new(admin.clone());
        let allowed_ft_accounts: Vec<AccountId> = vec![
            "wrap.near".to_string().try_into().unwrap(),
            "usn".to_string().try_into().unwrap(),
        ];

        contract
            .create_new_epoch(Some(allowed_ft_accounts))
            .unwrap();

        contract.in_funding = true;

        let user_id: AccountId = "user.near".to_string().try_into().unwrap();
        let creator_account_id = "creator.near".to_string().try_into().unwrap();
        let nft_rank = UserNFTRank::Common;
        let ft_token_id = "wrap.near".to_string().try_into().unwrap();

        contract
            .user_funding_creator(user_id, creator_account_id, nft_rank, ft_token_id)
            .unwrap();

        // TODO: continue test
    }
}
