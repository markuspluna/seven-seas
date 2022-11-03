use std::println;

use crate::sea::DataKey;
use soroban_sdk::{BigInt, Env};

pub fn get_total_shares(e: &Env) -> BigInt {
    e.data().get_unchecked(DataKey::ShareTotal).unwrap()
}

pub fn get_rate(e: &Env) -> BigInt {
    e.data().get_unchecked(DataKey::Rate).unwrap()
}

pub fn get_decimals(e: &Env) -> BigInt {
    e.data().get_unchecked(DataKey::Decimals).unwrap()
}

pub fn decimals_in_int(e: &Env) -> BigInt {
    return BigInt::from_i64(e, 1000000000000000000);
}

pub fn get_index(e: &Env) -> BigInt {
    e.data().get_unchecked(DataKey::Index).unwrap()
}

pub fn get_last_block(e: &Env) -> u32 {
    e.data().get_unchecked(DataKey::LastBlock).unwrap()
}

pub fn get_new_index(e: &Env) -> BigInt {
    let block_now = e.ledger().sequence();
    println!("{}", block_now - get_last_block(e));
    println!("{}", get_index(e));

    return get_index(e) + get_rate(e) * BigInt::from_u32(&e, block_now - get_last_block(e));
}

pub fn set_total_shares(e: &Env, amount: BigInt) {
    e.data().set(DataKey::ShareTotal, amount)
}

pub fn set_index(e: &Env, index: BigInt) {
    e.data().set(DataKey::Index, index)
}

pub fn set_last_block(e: &Env) {
    let block_now = e.ledger().sequence();
    e.data().set(DataKey::LastBlock, block_now)
}

pub fn set_decimals(e: &Env) {
    e.data().set(DataKey::Decimals, BigInt::from_i32(e, 18))
}
