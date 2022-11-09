#![cfg(test)]

use soroban_sdk::{
    testutils::{Accounts, Ledger, LedgerInfo},
    BigInt, Env,
};
mod helper;
use helper::{create_sea_contract, generate_contract_id};
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
    let share_token_contract_id = generate_contract_id(&e);

    // deploy and init sea
    let rate = BigInt::from_i64(&e, 5);
    let sea_contract_id = generate_contract_id(&e);
    let sea_client = create_sea_contract(&e, &sea_contract_id);
    sea_client.with_source_account(&token_admin).initialize(
        &share_token_contract_id,
        &base_token_contract_id,
        &rate,
        &BigInt::from_i64(&e, 10000000),
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
