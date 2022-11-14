#![cfg(test)]

use soroban_auth::{Identifier, Signature};
use soroban_sdk::{testutils::Accounts, BigInt, Env};

mod helper;
use helper::{
    create_base_token_contract, create_sea_contract, create_share_token_contract,
    generate_contract_id,
};
extern crate std;

#[test]
fn test_deposit_happy_path() {
    let e = Env::default();

    let deposit_amount_i64 = 123456789;
    let deposit_amount = BigInt::from_i64(&e, deposit_amount_i64);

    // deploy token contracts
    let token_admin = e.accounts().generate_and_create();
    let base_token_contract_id = generate_contract_id(&e);
    let base_token_client = create_base_token_contract(&e, &base_token_contract_id, &token_admin);
    let share_token_contract_id = generate_contract_id(&e);
    let share_token_client =
        create_share_token_contract(&e, &share_token_contract_id, &token_admin);

    // setup env
    let user1_acct = e.accounts().generate_and_create();
    let user1_id = Identifier::Account(user1_acct.clone());
    base_token_client.with_source_account(&token_admin).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &user1_id,
        &deposit_amount,
    );
    assert_eq!(base_token_client.balance(&user1_id), deposit_amount);

    // deploy and init sea
    let rate = BigInt::from_i64(&e, 5);
    let target_raid_interval: u32 = 1800;
    let sea_contract_id = generate_contract_id(&e);
    let sea_id = Identifier::Contract(sea_contract_id.clone());
    let sea_client = create_sea_contract(&e, &sea_contract_id);
    sea_client.with_source_account(&token_admin).initialize(
        &share_token_contract_id,
        &base_token_contract_id,
        &rate,
        &BigInt::from_i64(&e, 10000000),
        &target_raid_interval,
    );

    // transfer admin priviliges
    share_token_client
        .with_source_account(&token_admin)
        .set_admin(&Signature::Invoker, &BigInt::zero(&e), &sea_id);
    base_token_client
        .with_source_account(&token_admin)
        .set_admin(&Signature::Invoker, &BigInt::zero(&e), &sea_id);

    // deposit
    sea_client
        .with_source_account(&user1_acct)
        .sink(&deposit_amount);

    assert_eq!(base_token_client.balance(&user1_id), BigInt::zero(&e));
    assert_eq!(base_token_client.balance(&sea_id), BigInt::zero(&e));
    assert_eq!(share_token_client.balance(&user1_id), deposit_amount);
}
