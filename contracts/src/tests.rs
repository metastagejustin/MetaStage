// use std::collections::HashMap;

// use anyhow::*;
// use near_primitives::transaction;
// use near_sdk::ONE_YOCTO;
// use near_sdk::{env::STORAGE_PRICE_PER_BYTE, json_types::U128};
// use near_units::{parse_gas, parse_near};
// use workspaces::{network::Sandbox, Account, AccountId, Contract, Worker};

// use crate::*;

// pub async fn init() -> anyhow::Result<(
//     Contract,
//     Account,
//     Contract,
//     Contract,
//     Account,
//     Account,
//     Account,
//     Worker<Sandbox>,
// )> {
//     // get worker
//     let worker = &workspaces::sandbox().await?;

//     // get accounts registered on the fungible token contracts
//     let owner = worker.root_account().unwrap();

//     let admin = owner
//         .create_subaccount("admin")
//         .initial_balance(parse_near!("10,000,000 N"))
//         .transact()
//         .await?
//         .into_result()
//         .unwrap();

//     // get alpha fungible token contract
//     let alpha_ft_contract = worker
//         .dev_deploy(&include_bytes!("../res/fungible_token.wasm").to_vec())
//         .await?;

//     // initial balance for fungible tokens
//     let initial_balance = parse_near!("1,000,000,000,000 N").to_string();

//     let alice = owner
//         .create_subaccount("alice")
//         .initial_balance(parse_near!("10,000,000 N"))
//         .transact()
//         .await?
//         .into_result()
//         .unwrap();

//     // get alpha fungible token contract
//     let res = alice
//         .call(alpha_ft_contract.id(), "new_default_meta")
//         .args_json(serde_json::json!({
//             "owner_id": owner.id(),
//             "total_supply": initial_balance
//         }))
//         .gas(300_000_000_000_000)
//         .transact()
//         .await?;

//     // result is successful
//     assert!(res.is_success());

//     // get beta fungible token contract
//     let beta_ft_contract = worker
//         .dev_deploy(&include_bytes!("../res/fungible_token.wasm").to_vec())
//         .await?;

//     let bob = owner
//         .create_subaccount("bob")
//         .initial_balance(parse_near!("10,000,000 N"))
//         .transact()
//         .await?
//         .into_result()
//         .unwrap();

//     // get beta fungible token contract
//     let res = bob
//         .call(beta_ft_contract.id(), "new_default_meta")
//         .args_json(serde_json::json!({
//             "owner_id": owner.id(),
//             "total_supply": initial_balance
//         }))
//         .gas(300_000_000_000_000)
//         .transact()
//         .await?;

//     // result is successful
//     assert!(res.is_success());

//     // get concentrated liquidity contract
//     let metadao_contract = worker
//         .dev_deploy(&workspaces::compile_project("./").await?)
//         .await?;

//     let charlie = owner
//         .create_subaccount("charlie")
//         .initial_balance(parse_near!("10,000,000 N"))
//         .transact()
//         .await?
//         .into_result()
//         .unwrap();

//     let res = charlie
//         .call(metadao_contract.id(), "new")
//         .args_json(serde_json::json!({
//             "admin": admin.id()
//         }))
//         .gas(300_000_000_000_000)
//         .transact()
//         .await?;

//     assert!(res.is_success());

//     // register alice as user in both alpha and beta ft contracts
//     register_and_transfer_funds_to_account_id(&owner, &alpha_ft_contract, alice.id()).await?;
//     register_and_transfer_funds_to_account_id(&owner, &beta_ft_contract, alice.id()).await?;

//     // register our contract in both alpha and beta ft contracts
//     register_and_transfer_funds_to_account_id(&owner, &alpha_ft_contract, metadao_contract.id())
//         .await?;
//     register_and_transfer_funds_to_account_id(&owner, &beta_ft_contract, metadao_contract.id())
//         .await?;

//     // register our contract in both alpha and beta ft contracts
//     register_and_transfer_funds_to_account_id(&owner, &alpha_ft_contract, charlie.id()).await?;
//     register_and_transfer_funds_to_account_id(&owner, &beta_ft_contract, charlie.id()).await?;

//     Ok((
//         metadao_contract,
//         admin,
//         alpha_ft_contract,
//         beta_ft_contract,
//         alice,
//         bob,
//         charlie,
//         worker.clone(),
//     ))
// }

// pub async fn register_and_transfer_funds_to_account_id(
//     owner: &Account,
//     contract: &Contract,
//     account_id: &AccountId,
// ) -> anyhow::Result<()> {
//     // register user
//     let res = owner
//         .call(contract.id(), "storage_deposit")
//         .args_json(serde_json::json!({ "account_id": account_id }))
//         .gas(300_000_000_000_000)
//         .deposit(parse_near!("1 N"))
//         .transact()
//         .await?;
//     assert!(res.is_success());

//     // send funds to user
//     // after registering alice account, we have to send tokens to it
//     let res = owner
//         .call(contract.id(), "ft_transfer")
//         .args_json(serde_json::json!({
//             "receiver_id": account_id,
//             "amount": parse_near!("1,000,000,000 N").to_string().as_str()
//         }))
//         .deposit(1)
//         .transact()
//         .await?;
//     assert!(res.is_success());

//     Ok(())
// }

// pub async fn ft_transfer_call(
//     ft_contract: &Contract,
//     user: &Account,
//     receiver: &Contract,
//     amount: u128,
//     msg: &str,
// ) -> anyhow::Result<()> {
//     let res = user
//         .call(ft_contract.id(), "ft_transfer")
//         .args_json(serde_json::json!({
//             "receiver_id": receiver.id(),
//             "amount": amount.to_string().as_str(),
//             "msg": msg
//         }))
//         .gas(parse_gas!("100 Tgas") as u64)
//         .deposit(ONE_YOCTO)
//         .transact()
//         .await?;

//     assert!(res.is_success());

//     let res = ft_contract
//         .as_account()
//         .call(receiver.id(), "ft_on_transfer")
//         .args_json(serde_json::json!({
//             "sender_id": user.id(),
//             "amount": amount.to_string().as_str(),
//             "msg": msg
//         }))
//         .gas(parse_gas!("300 Tgas") as u64)
//         .transact()
//         .await?;

//     assert!(res.is_success());

//     Ok(())
// }

// #[tokio::test]
// async fn on_epoch_start() -> anyhow::Result<()> {
//     let (metadao_contract, admin, alpha_ft_contract, beta_ft_contract, alice, bob, charlie, worker) =
//         init().await?;

//     let allowed_ft_account_ids = vec![*alpha_ft_contract.id(), *beta_ft_contract.id()];
//     let mut protocol_fee = HashMap::<FTAccountId, f64>::new();

//     protocol_fee.insert(alpha_ft_contract.id().clone().into(), 0.001);
//     protocol_fee.insert(beta_ft_contract.id().clone().into(), 0.001);

//     let res = admin
//         .call(metadao_contract, "create_new_epoch")
//         .args_json(serde_json::json!({
//             "allowed_ft_account_ids": Some(allowed_ft_account_ids),
//             "protocol_fee": protocol_fee
//         }))
//         .gas(300_000_000_000_000)
//         .transact()
//         .await?;

//     Ok(())
// }
