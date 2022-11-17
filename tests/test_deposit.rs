#![cfg(test)]

use soroban_auth::{Identifier, Signature};
use soroban_sdk::{
    testutils::{Accounts, Ledger, LedgerInfo},
    BigInt, Env,
};

mod helper;
use helper::{create_base_token_contract, create_sea_contract, generate_contract_id};
extern crate std;

#[test]
fn test_deposit_happy_path() {
    let e = Env::default();
    e.ledger().set(LedgerInfo {
        timestamp: 100,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    let deposit_amount_i64 = 123456789;
    let deposit_amount = BigInt::from_i64(&e, deposit_amount_i64);

    // deploy token contracts
    let token_admin = e.accounts().generate_and_create();
    let base_token_contract_id = generate_contract_id(&e);
    let base_token_client = create_base_token_contract(&e, &base_token_contract_id, &token_admin);

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
    let rate = BigInt::from_i64(&e, 500);
    let target_raid_interval: u32 = 1800;
    let sea_contract_id = generate_contract_id(&e);
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

    //pass 10000 blocks
    e.ledger().set(LedgerInfo {
        timestamp: 100,
        protocol_version: 1,
        sequence_number: 10010,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    // deposit
    sea_client
        .with_source_account(&user1_acct)
        .bury(&deposit_amount);

    let expected_bury_amount = BigInt::from_i32(&e, 122842576);
    assert_eq!(base_token_client.balance(&user1_id), BigInt::zero(&e));
    assert_eq!(base_token_client.balance(&sea_id), BigInt::zero(&e));
    assert_eq!(sea_client.get_buried(&user1_id), expected_bury_amount);
}
