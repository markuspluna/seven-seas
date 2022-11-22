#![cfg(test)]

use soroban_auth::{Identifier, Signature};
use soroban_sdk::{
    testutils::{Accounts, Ledger, LedgerInfo},
    BigInt, Env,
};
mod helper;
use helper::{
    create_base_token_contract, create_sea_contract, create_usdc_token_contract,
    generate_contract_id, SCALER,
};
extern crate std;
#[test]
fn test_create_voyage() {
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

    // deploy token contracts
    let token_admin = e.accounts().generate_and_create();
    let usdc_token_contract_id = generate_contract_id(&e);
    let base_token_contract_id = generate_contract_id(&e);

    // deploy and init sea
    let rate = BigInt::from_i64(&e, 5);
    let target_raid_interval: u32 = 1800;
    let sea_contract_id = generate_contract_id(&e);
    let sea_client = create_sea_contract(&e, &sea_contract_id);
    sea_client.with_source_account(&token_admin).initialize(
        &base_token_contract_id,
        &rate,
        &target_raid_interval,
    );

    // create voyage
    sea_client.with_source_account(&token_admin).new_voyage(
        &usdc_token_contract_id,
        &vyg_price,
        &max_vygs,
    );

    //check that voyage was created
    let expected_id: i32 = 1;
    let expected_expiration: u32 = 10 + 100800;
    let usdc_voyage = sea_client.get_voyage(&expected_id);
    assert_eq!(usdc_voyage.max_vygs, max_vygs);
    assert_eq!(usdc_voyage.vyg_asset, usdc_token_contract_id);
    assert_eq!(usdc_voyage.price, vyg_price);
    assert_eq!(usdc_voyage.expiration, expected_expiration);
    assert_eq!(usdc_voyage.n_embarked, BigInt::zero(&e));
}

#[test]
fn test_voyage() {
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
    let usdc_token_client = create_usdc_token_contract(&e, &usdc_token_contract_id, &token_admin);

    // setup env
    let user1_acct = e.accounts().generate_and_create();
    let user1_id = Identifier::Account(user1_acct.clone());
    usdc_token_client.with_source_account(&token_admin).mint(
        &soroban_auth::Signature::Invoker,
        &BigInt::zero(&e),
        &user1_id,
        &user_usdc_spend,
    );
    assert_eq!(usdc_token_client.balance(&user1_id), user_usdc_spend);

    // deploy and init sea
    let rate = BigInt::from_i64(&e, 5);
    let target_raid_interval: u32 = 1800;
    let sea_contract_id = generate_contract_id(&e);
    let sea_id = Identifier::Contract(sea_contract_id.clone());
    let sea_client = create_sea_contract(&e, &sea_contract_id);
    sea_client.with_source_account(&token_admin).initialize(
        &base_token_contract_id,
        &rate,
        &target_raid_interval,
    );

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

    //check that user entered voyage
    let num_user_voyages = sea_client.get_u_vygs(&user1_id, &expected_id);
    assert_eq!(num_user_voyages, user_num_voyages);
    //check that usdc was transferred
    assert_eq!(usdc_token_client.balance(&user1_id), BigInt::zero(&e));
    assert_eq!(usdc_token_client.balance(&sea_id), user_usdc_spend.clone());
    //check that voyages were tracked
    let voyage_info = sea_client.get_voyage(&expected_id);
    assert_eq!(voyage_info.n_embarked, user_num_voyages);
}

#[test]
fn test_end_voyage() {
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
    let usdc_token_client = create_usdc_token_contract(&e, &usdc_token_contract_id, &token_admin);
    let base_token_client = create_base_token_contract(&e, &base_token_contract_id, &token_admin);

    // setup env
    let user1_acct = e.accounts().generate_and_create();
    let user1_id = Identifier::Account(user1_acct.clone());
    usdc_token_client.with_source_account(&token_admin).mint(
        &soroban_auth::Signature::Invoker,
        &BigInt::zero(&e),
        &user1_id,
        &user_usdc_spend,
    );
    assert_eq!(usdc_token_client.balance(&user1_id), user_usdc_spend);
    // deploy and init sea
    let rate = BigInt::from_i64(&e, 5);
    let sea_contract_id = generate_contract_id(&e);
    let target_raid_interval: u32 = 1800;
    let sea_id = Identifier::Contract(sea_contract_id.clone());
    let sea_client = create_sea_contract(&e, &sea_contract_id);
    sea_client.with_source_account(&token_admin).initialize(
        &base_token_contract_id,
        &rate,
        &target_raid_interval,
    );

    // transfer admin priviliges
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

    //let voyage expire
    let expected_expiration: u32 = 10 + 100800;
    e.ledger().set(LedgerInfo {
        timestamp: 100,
        protocol_version: 1,
        sequence_number: expected_expiration,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    //end voyage
    sea_client
        .with_source_account(&user1_acct)
        .end_voyage(&expected_id);

    //check that user received shells
    let expected_shells = user_num_voyages.clone() * BigInt::from_i64(&e, SCALER);
    assert_eq!(base_token_client.balance(&user1_id), expected_shells);
    //check that user no longer has an outstanding voyage
    let remaining_vygs = sea_client.get_u_vygs(&user1_id, &expected_id);
    assert_eq!(remaining_vygs, 0);
}
