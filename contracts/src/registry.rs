use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde::{Deserialize, Serialize};

use crate::consts::CREATOR_REGISTRY_STORAGE_COST;
use crate::{
    error::MetaDaoError,
    nft::{
        CreatorNFTCopies, CreatorNFTDescription, CreatorNFTExtra, CreatorNFTMedia,
        CreatorNFTRanking, CreatorNFTReference, CreatorNFTTitle,
    },
    *,
};

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct CreatorMetadata {
    nft_ranks: Vec<CreatorNFTRanking>,
    titles: Vec<CreatorNFTTitle>,
    descriptions: Vec<CreatorNFTDescription>,
    medias: Vec<CreatorNFTMedia>,
    copies: Vec<CreatorNFTCopies>,
    extras: Vec<CreatorNFTExtra>,
    references: Vec<CreatorNFTReference>,
}

impl CreatorMetadata {
    pub fn nft_rank(&self, user_nft_rank: UserNFTRank) -> CreatorNFTRanking {
        match user_nft_rank {
            UserNFTRank::Common => self.nft_ranks[0].clone(),
            UserNFTRank::Uncommon => self.nft_ranks[1].clone(),
            UserNFTRank::Rare => self.nft_ranks[2].clone(),
        }
    }

    pub fn get_nft_ranks(&self) -> Vec<CreatorNFTRanking> {
        self.nft_ranks.clone()
    }

    pub fn get_titles(&self) -> Vec<CreatorNFTTitle> {
        self.titles.clone()
    }

    pub fn get_descriptions(&self) -> Vec<CreatorNFTDescription> {
        self.descriptions.clone()
    }

    pub fn get_medias(&self) -> Vec<CreatorNFTMedia> {
        self.medias.clone()
    }

    pub fn get_copies(&self) -> Vec<CreatorNFTCopies> {
        self.copies.clone()
    }

    pub fn get_extras(&self) -> Vec<CreatorNFTExtra> {
        self.extras.clone()
    }

    pub fn get_references(&self) -> Vec<CreatorNFTReference> {
        self.references.clone()
    }

    pub fn get_token_metadata(&self, nft_rank: UserNFTRank) -> Result<TokenMetadata, MetaDaoError> {
        let index = match nft_rank {
            UserNFTRank::Common => 0_usize,
            UserNFTRank::Uncommon => 1_usize,
            UserNFTRank::Rare => 2_usize,
        };

        let title = Some(String::from(self.get_titles()[index].get_title()));
        let description = Some(String::from(
            self.get_descriptions()[index].get_description(),
        ));
        let media = Some(String::from(self.get_medias()[index].get_media()));
        let media_hash = env::sha256(media.clone().unwrap().as_bytes());
        let media_hash = Some(Base64VecU8::from(media_hash));
        let copies = Some(self.get_copies()[index].get_copies());
        let issued_at = Some(format!("block_timestamp: {}", env::block_timestamp()));
        let extra = Some(String::from(self.get_extras()[index].get_extra()));
        let reference = self.get_references()[index].get_reference();
        let reference_hash = if reference.is_none() {
            Some(Base64VecU8::from(env::sha256(
                reference.clone().unwrap().as_bytes(),
            )))
        } else {
            None
        };

        let token_metadata = TokenMetadata {
            title,
            description,
            media,
            media_hash,
            copies,
            issued_at,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra,
            reference,
            reference_hash,
        };

        Ok(token_metadata)
    }
}

#[near_bindgen]
impl MetaDaoContract {
    #[payable]
    #[handle_result]
    pub fn creator_registration(&mut self, metadata: CreatorMetadata) -> Result<(), MetaDaoError> {
        let creator_account_id = env::predecessor_account_id();

        if env::attached_deposit()
            < (CREATOR_REGISTRY_STORAGE_COST as u128) * env::STORAGE_PRICE_PER_BYTE
        {
            return Err(MetaDaoError::UncoveredStorageCosts);
        }

        if !self.is_epoch_on {
            return Err(MetaDaoError::EpochIsOff);
        }

        if !self.in_registration {
            return Err(MetaDaoError::NotInRegistrationPeriod);
        }

        let mut creator_funding = self
            .creator_funding
            .get(&self.epoch)
            .ok_or(MetaDaoError::EpochIsOff)?;

        let mut creators_metadata = self
            .creators_metadata
            .get(&self.epoch)
            .ok_or(MetaDaoError::EpochIsOff)?;

        let mut creators_per_epoch = self
            .creators_per_epoch_set
            .get(&self.epoch)
            .ok_or(MetaDaoError::EpochIsOff)?;

        creator_funding.insert(&creator_account_id, &vec![]);
        creators_metadata.insert(&creator_account_id, &metadata);
        creators_per_epoch.insert(&creator_account_id);

        self.creator_funding.insert(&self.epoch, &creator_funding);
        self.creators_metadata
            .insert(&self.epoch, &creators_metadata);
        self.creators_per_epoch_set
            .insert(&self.epoch, &creators_per_epoch);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

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
            .signer_account_id(accounts(0))
            .predecessor_account_id(accounts(0))
            .prepaid_gas(Gas(300 * 10u64.pow(16)))
            .attached_deposit(storage)
            .build()
    }

    fn get_registry_metadata() -> CreatorMetadata {
        CreatorMetadata {
            nft_ranks: vec![
                CreatorNFTRanking::Common(HashMap::<FTAccountId, u128>::from_iter([(
                    "ft_account_id.near".to_string().try_into().unwrap(),
                    100_u128,
                )])),
                CreatorNFTRanking::Uncommon(HashMap::<FTAccountId, u128>::from_iter([(
                    "ft_account_id.near".to_string().try_into().unwrap(),
                    250_u128,
                )])),
                CreatorNFTRanking::Rare(HashMap::<FTAccountId, u128>::from_iter([(
                    "ft_account_id.near".to_string().try_into().unwrap(),
                    500_u128,
                )])),
            ],
            titles: vec![
                CreatorNFTTitle::Common("common".to_string()),
                CreatorNFTTitle::Uncommon("uncommon".to_string()),
                CreatorNFTTitle::Rare("rare".to_string()),
            ],
            descriptions: vec![
                CreatorNFTDescription::Common("common".to_string()),
                CreatorNFTDescription::Uncommon("uncommon".to_string()),
                CreatorNFTDescription::Rare("rare".to_string()),
            ],
            medias: vec![
                CreatorNFTMedia::Common("media_common".to_string()),
                CreatorNFTMedia::Uncommon("media_uncommon".to_string()),
                CreatorNFTMedia::Rare("media_rare".to_string()),
            ],
            copies: vec![
                CreatorNFTCopies::Common(100_u64),
                CreatorNFTCopies::Uncommon(50_u64),
                CreatorNFTCopies::Rare(5_u64),
            ],
            extras: vec![
                CreatorNFTExtra::Common("extra_common".to_string()),
                CreatorNFTExtra::Uncommon("extra_uncommon".to_string()),
                CreatorNFTExtra::Rare("extra_rare".to_string()),
            ],
            references: vec![
                CreatorNFTReference::Common(None),
                CreatorNFTReference::Uncommon(None),
                CreatorNFTReference::Rare(None),
            ],
        }
    }

    #[test]
    fn it_works_registry() {
        let admin: AccountId = accounts(0);
        let storage = (CREATOR_REGISTRY_STORAGE_COST as u128) * env::STORAGE_PRICE_PER_BYTE;

        let context = get_context_with_storage(storage);
        testing_env!(context);

        let mut contract = MetaDaoContract::new(admin.clone());
        let allowed_ft_accounts: Vec<AccountId> = vec![
            "wrap.near".to_string().try_into().unwrap(),
            "usn".to_string().try_into().unwrap(),
        ];

        let mut protocol_fee = UnorderedMap::<FTAccountId, f64>::new(b"test_protocol_fee".to_vec());

        protocol_fee.insert(&"wrap.near".to_string().try_into().unwrap(), &0.05);
        protocol_fee.insert(&"usn".to_string().try_into().unwrap(), &0.03);

        contract
            .create_new_epoch(Some(allowed_ft_accounts), protocol_fee)
            .unwrap();

        contract.is_epoch_on = true;

        contract.set_registration().unwrap();

        let creator_account_id = accounts(0);

        let metadata = get_registry_metadata();

        contract.creator_registration(metadata.clone()).unwrap();

        let creator_funding = contract
            .creator_funding
            .get(&contract.epoch)
            .ok_or(MetaDaoError::EpochIsOff)
            .unwrap();

        let creators_metadata = contract
            .creators_metadata
            .get(&contract.epoch)
            .ok_or(MetaDaoError::EpochIsOff)
            .unwrap();

        let creators_per_epoch = contract
            .creators_per_epoch_set
            .get(&contract.epoch)
            .ok_or(MetaDaoError::EpochIsOff)
            .unwrap();

        assert_eq!(creator_funding.get(&creator_account_id).unwrap(), vec![]);
        assert_eq!(
            creators_metadata.get(&creator_account_id).unwrap(),
            metadata
        );
        assert_eq!(creators_per_epoch.len(), 1);
        assert!(creators_per_epoch.contains(&creator_account_id));
    }

    #[test]
    fn it_fails_creator_registry_if_not_enough_funds_for_storage() {
        let admin: AccountId = accounts(0);
        let storage = (CREATOR_REGISTRY_STORAGE_COST as u128) * env::STORAGE_PRICE_PER_BYTE - 1u128;

        let context = get_context_with_storage(storage);
        testing_env!(context);

        let mut contract = MetaDaoContract::new(admin.clone());
        let allowed_ft_accounts: Vec<AccountId> = vec![
            "wrap.near".to_string().try_into().unwrap(),
            "usn".to_string().try_into().unwrap(),
        ];

        let mut protocol_fee = UnorderedMap::<FTAccountId, f64>::new(b"test_protocol_fee".to_vec());

        protocol_fee.insert(&"wrap.near".to_string().try_into().unwrap(), &0.05);
        protocol_fee.insert(&"usn".to_string().try_into().unwrap(), &0.03);

        contract
            .create_new_epoch(Some(allowed_ft_accounts), protocol_fee)
            .unwrap();

        contract.is_epoch_on = true;

        contract.set_registration().unwrap();

        let metadata = get_registry_metadata();

        contract.is_epoch_on = false;

        assert!(contract
            .creator_registration(metadata.clone())
            .unwrap_err()
            .to_string()
            .contains("Uncovered storage costs"));
    }

    #[test]
    fn it_fails_creator_registry_if_epoch_is_on() {
        let admin: AccountId = accounts(0);
        let storage = (CREATOR_REGISTRY_STORAGE_COST as u128) * env::STORAGE_PRICE_PER_BYTE - 1u128;

        let context = get_context_with_storage(storage);
        testing_env!(context);

        let mut contract = MetaDaoContract::new(admin.clone());
        let allowed_ft_accounts: Vec<AccountId> = vec![
            "wrap.near".to_string().try_into().unwrap(),
            "usn".to_string().try_into().unwrap(),
        ];

        let mut protocol_fee = UnorderedMap::<FTAccountId, f64>::new(b"test_protocol_fee".to_vec());

        protocol_fee.insert(&"wrap.near".to_string().try_into().unwrap(), &0.05);
        protocol_fee.insert(&"usn".to_string().try_into().unwrap(), &0.03);

        contract
            .create_new_epoch(Some(allowed_ft_accounts), protocol_fee)
            .unwrap();

        contract.is_epoch_on = true;

        contract.set_registration().unwrap();

        let metadata = get_registry_metadata();

        contract.is_epoch_on = false;

        assert!(contract
            .creator_registration(metadata.clone())
            .unwrap_err()
            .to_string()
            .contains("Currently, epoch is off"));
    }

    #[test]
    fn it_fails_creator_registry_if_not_in_registration() {
        let admin: AccountId = accounts(0);
        let storage = (CREATOR_REGISTRY_STORAGE_COST as u128) * env::STORAGE_PRICE_PER_BYTE - 1u128;

        let context = get_context_with_storage(storage);
        testing_env!(context);

        let mut contract = MetaDaoContract::new(admin.clone());
        let allowed_ft_accounts: Vec<AccountId> = vec![
            "wrap.near".to_string().try_into().unwrap(),
            "usn".to_string().try_into().unwrap(),
        ];

        let mut protocol_fee = UnorderedMap::<FTAccountId, f64>::new(b"test_protocol_fee".to_vec());

        protocol_fee.insert(&"wrap.near".to_string().try_into().unwrap(), &0.05);
        protocol_fee.insert(&"usn".to_string().try_into().unwrap(), &0.03);

        contract
            .create_new_epoch(Some(allowed_ft_accounts), protocol_fee)
            .unwrap();

        contract.is_epoch_on = true;

        let metadata = get_registry_metadata();

        assert!(contract
            .creator_registration(metadata.clone())
            .unwrap_err()
            .to_string()
            .contains("Not in Registration period"));
    }
}
