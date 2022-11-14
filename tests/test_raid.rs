#![cfg(test)]

use soroban_auth::{Identifier, Signature};
use soroban_sdk::{
    testutils::{Accounts, Ledger, LedgerInfo},
    BigInt, Env,
};
mod helper;
use helper::{
    create_base_token_contract, create_sea_contract, create_share_token_contract,
    create_usdc_token_contract, generate_contract_id,
};
extern crate std;

#[test]
#[should_panic(expected = "Not Implemented")]
fn test_raid_happy_path() {
    let e = Env::default();

    //set ledger sequence so we can estimate voyage expiration
    e.ledger().set(LedgerInfo {
        timestamp: 100,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    let max_vygs = BigInt::from_i64(&e, 1000);
    let vyg_price = BigInt::from_i64(&e, 10);
    let user_num_voyages = BigInt::from_i64(&e, 100);
    let user_usdc_spend = vyg_price.clone() * user_num_voyages.clone();

    // deploy token contracts
    let token_admin = e.accounts().generate_and_create();
    let usdc_token_contract_id = generate_contract_id(&e);
    let base_token_contract_id = generate_contract_id(&e);
    let share_token_contract_id = generate_contract_id(&e);
    let usdc_token_client = create_usdc_token_contract(&e, &usdc_token_contract_id, &token_admin);
    let base_token_client = create_base_token_contract(&e, &base_token_contract_id, &token_admin);
    let share_token_client =
        create_share_token_contract(&e, &share_token_contract_id, &token_admin);
    // setup env
    let user1_acct = e.accounts().generate_and_create();
    let user1_id = Identifier::Account(user1_acct.clone());
    usdc_token_client.with_source_account(&token_admin).mint(
        &soroban_auth::Signature::Invoker,
        &BigInt::zero(&e),
        &user1_id,
        &user_usdc_spend,
    );
    let user2_acct = e.accounts().generate_and_create();
    let user2_id = Identifier::Account(user2_acct.clone());
    base_token_client.with_source_account(&token_admin).mint(
        &soroban_auth::Signature::Invoker,
        &BigInt::zero(&e),
        &user2_id,
        &BigInt::from_i64(&e, 1),
    );
    assert_eq!(usdc_token_client.balance(&user1_id), user_usdc_spend);
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

    // create voyage
    sea_client.with_source_account(&token_admin).new_voyage(
        &usdc_token_contract_id,
        &vyg_price,
        &max_vygs,
    );

    // enter voyage with user
    usdc_token_client.with_source_account(&user1_acct).approve(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &sea_id,
        &user_usdc_spend,
    );
    let expected_id: i32 = 1;
    sea_client
        .with_source_account(&user1_acct)
        .voyage(&expected_id, &user_num_voyages);

    //let time pass
    let new_block: u32 = 10 + 900;
    e.ledger().set(LedgerInfo {
        timestamp: 100,
        protocol_version: 1,
        sequence_number: new_block,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    //TODO: add testing for raiding success, we need to wait for the PRNG PR - https://github.com/stellar/rs-soroban-env/pull/544
    //Currently we just check that the function emits a not implemented panic
    sea_client.raid(&expected_id, &user1_id);
}
