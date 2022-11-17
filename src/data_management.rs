use crate::{
    pirates_bay::{DataKey, VoyageInfo, VoyageKey},
    token::Identifier,
};
use soroban_sdk::{BigInt, BytesN, Env};

pub const SCALER: i64 = 10000000;

/******** Read functions */
pub fn get_user_buried(e: &Env, user: Identifier) -> BigInt {
    e.data()
        .get(DataKey::UserBuried(user.clone()))
        .unwrap_or(Ok(BigInt::zero(&e)))
        .unwrap()
}

pub fn get_total_buried(e: &Env) -> BigInt {
    e.data().get_unchecked(DataKey::TtlBuried).unwrap()
}

pub fn get_rate(e: &Env) -> BigInt {
    e.data().get_unchecked(DataKey::Rate).unwrap()
}

pub fn get_decimals(e: &Env) -> BigInt {
    e.data().get_unchecked(DataKey::Decimals).unwrap()
}
pub fn get_index(e: &Env) -> BigInt {
    e.data().get_unchecked(DataKey::Index).unwrap()
}

pub fn get_new_index(e: &Env) -> BigInt {
    let block_now = e.ledger().sequence();
    //multiply by 1000 to avoid decimals
    return get_index(e)
        + get_rate(e)
            * BigInt::from_u32(&e, block_now - get_last_block(e))
            * BigInt::from_i64(e, SCALER)
            / 100;
}

pub fn get_last_block(e: &Env) -> u32 {
    e.data().get_unchecked(DataKey::LastBlock).unwrap()
}

pub fn get_base_token(e: &Env) -> BytesN<32> {
    e.data().get_unchecked(DataKey::BaseToken).unwrap()
}

pub fn get_base_token_client(e: &Env) -> crate::token::Client {
    let id = get_base_token(e);
    crate::token::Client::new(e, id)
}

pub fn get_voyage(e: &Env, voyage_id: i32) -> VoyageInfo {
    e.data().get_unchecked(DataKey::Voyages(voyage_id)).unwrap()
}

pub fn get_last_voyage_id(e: &Env) -> i32 {
    e.data().get(DataKey::LastVoyage).unwrap_or(Ok(0)).unwrap()
}

pub fn get_user_voyage(e: &Env, user: Identifier, voyage: i32) -> BigInt {
    let voyage_key = VoyageKey {
        user_id: user.clone(),
        voyage_id: voyage,
    };
    let data: BigInt = e
        .data()
        .get(DataKey::UserVoyage(voyage_key))
        .unwrap_or(Ok(BigInt::zero(&e)))
        .unwrap();
    return data;
}

pub fn get_last_raid_block(e: &Env) -> u32 {
    e.data().get(DataKey::LastRaid).unwrap_or(Ok(0)).unwrap()
}

pub fn get_target_raid_interval(e: &Env) -> u32 {
    e.data().get_unchecked(DataKey::TgtRaidInt).unwrap()
}

/******** Write Functions */
pub fn set_user_buried(e: &Env, user_id: Identifier, amount: BigInt) {
    let key = DataKey::UserBuried(user_id.clone());
    e.data().set(key, amount);
}

pub fn set_total_buried(e: &Env, amount: BigInt) {
    e.data().set(DataKey::TtlBuried, amount)
}

pub fn set_index(e: &Env, index: BigInt) {
    e.data().set(DataKey::Index, index)
}

pub fn set_last_block(e: &Env) {
    let block_now = e.ledger().sequence();
    e.data().set(DataKey::LastBlock, block_now)
}

pub fn set_decimals(e: &Env) {
    e.data().set(DataKey::Decimals, BigInt::from_i32(e, 7))
}

pub fn set_base_token(e: &Env, contract_id: BytesN<32>) {
    e.data().set(DataKey::BaseToken, contract_id);
}

pub fn set_rate(e: &Env, rate: BigInt) {
    e.data().set(DataKey::Rate, rate);
}

pub fn set_voyage(e: &Env, voyage_id: i32, voyage: VoyageInfo) {
    e.data().set(DataKey::Voyages(voyage_id), voyage)
}

pub fn set_user_voyage(e: &Env, user: Identifier, voyage_id: i32, amount: BigInt) {
    let voyage_key = VoyageKey {
        user_id: user.clone(),
        voyage_id: voyage_id,
    };

    e.data().set(DataKey::UserVoyage(voyage_key), amount)
}

pub fn remove_user_voyage(e: &Env, user: Identifier, voyage_id: i32) {
    let voyage_key = VoyageKey {
        user_id: user,
        voyage_id: voyage_id,
    };
    e.data().remove(DataKey::UserVoyage(voyage_key));
}

pub fn set_last_voyage_id(e: &Env, voyage_id: i32) {
    e.data().set(DataKey::LastVoyage, voyage_id)
}

pub fn set_last_raid(e: &Env) {
    let block_now = e.ledger().sequence();
    e.data().set(DataKey::LastRaid, block_now)
}

pub fn set_target_raid_interval(e: &Env, interval: u32) {
    e.data().set(DataKey::TgtRaidInt, interval)
}
