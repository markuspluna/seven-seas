#![cfg(test)]

use soroban_auth::{Identifier, Signature};
use soroban_sdk::{
    testutils::{Accounts, Ledger, LedgerInfo},
    BigInt, Env,
};

mod helper;
use helper::{
    create_base_token_contract, create_sea_contract, create_share_token_contract,
    generate_contract_id, SCALER,
};
extern crate std;
#[test]
fn test_withdraw_happy_path() {
    let e = Env::default();
    e.ledger().set(LedgerInfo {
        timestamp: 100,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    let withdraw_amount_i64 = 123456789;
    let withdraw_amount = BigInt::from_i64(&e, withdraw_amount_i64);

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
    share_token_client.with_source_account(&token_admin).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &user1_id,
        &withdraw_amount,
    );
    assert_eq!(share_token_client.balance(&user1_id), withdraw_amount);

    // deploy and init sea
    let rate = BigInt::from_i64(&e, 5); //equivalent to 0.000005
    let sea_contract_id = generate_contract_id(&e);
    let sea_id = Identifier::Contract(sea_contract_id.clone());
    let sea_client = create_sea_contract(&e, &sea_contract_id);
    sea_client.with_source_account(&token_admin).initialize(
        &share_token_contract_id,
        &base_token_contract_id,
        &rate,
        &BigInt::from_i64(&e, 10000000),
    );

    // transfer admin priviliges
    share_token_client
        .with_source_account(&token_admin)
        .set_admin(&Signature::Invoker, &BigInt::zero(&e), &sea_id);
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

    let expected_withdrawal =
        (rate.clone() * BigInt::from_i32(&e, 10000) + SCALER) * withdraw_amount.clone() / SCALER;

    // withdraw
    sea_client
        .with_source_account(&user1_acct)
        .dredge(&withdraw_amount);

    assert_eq!(expected_withdrawal, base_token_client.balance(&user1_id));
}
