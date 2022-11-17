use core::u32::MAX;

use crate::{
    captain::{check_captain, write_captain},
    data_management::{
        get_base_token_client, get_decimals, get_last_raid_block, get_last_voyage_id,
        get_new_index, get_target_raid_interval, get_total_buried, get_user_buried,
        get_user_voyage, get_voyage, remove_user_voyage, set_base_token, set_decimals, set_index,
        set_last_block, set_last_raid, set_last_voyage_id, set_rate, set_target_raid_interval,
        set_total_buried, set_user_buried, set_user_voyage, set_voyage, SCALER,
    },
};
use soroban_auth::{Identifier, Signature};
use soroban_sdk::{contractimpl, contracttype, symbol, BigInt, BytesN, Env, RawVal};

// ****** Contract Storage *****

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    BaseToken,              // address of the doubloon token
    UserBuried(Identifier), // bigint storing the amount of doubloons buried by a user
    TtlBuried,              // total doubloons buried
    Rate,                   // per-100-block rebase rate for buried_doubloon tokens
    Index,                  // rebase index for doubloon tokens
    LastBlock,              // last block the index was updated at
    Admin,                  // admin address
    Decimals,               // decimals for both the index, rate, and buried_doubloon tokens
    Voyages(i32),           // struct of voyage information
    LastVoyage,             // stores the id of the last voyage
    UserVoyage(VoyageKey),  // struct of user voyages
    LastRaid,               // stores the block the last raid was performed on
    TgtRaidInt,             // stores the target raid interval (num blocks between raids)
}

fn subtract_buried(e: &Env, from: Identifier, amount: BigInt) {
    let current = get_user_buried(&e, from.clone());
    if current < amount {
        panic!("not enough buried doubloons to unearth");
    }
    set_user_buried(&e, from, current - amount.clone());
    let total = get_total_buried(e);
    set_total_buried(e, total - amount);
}

fn add_buried(e: &Env, to: Identifier, amount: BigInt) {
    let current = get_user_buried(&e, to.clone());
    set_user_buried(&e, to, current + amount.clone());
    let total = get_total_buried(e);
    set_total_buried(e, total + amount);
}

fn burn_token(e: &Env, from: Identifier, amount: BigInt) {
    let base_token_client = get_base_token_client(&e);
    base_token_client.burn(&Signature::Invoker, &BigInt::zero(&e), &from, &amount)
}

fn mint_token(e: &Env, to: Identifier, amount: BigInt) {
    let base_token_client = get_base_token_client(&e);
    base_token_client.mint(&Signature::Invoker, &BigInt::zero(&e), &to, &amount)
}

fn transfer(e: &Env, contract_id: BytesN<32>, to: Identifier, amount: BigInt) {
    crate::token::Client::new(&e, contract_id).xfer(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &to,
        &amount,
    );
}

fn get_contract_id(e: &Env) -> Identifier {
    Identifier::Contract(e.get_current_contract().into())
}

const WEEK_IN_BLOCKS: u32 = 100_800;
pub struct PiratesBay;
pub trait PiratesBayTrait {
    // initialize the contract with caller as the admin - rate is the raw integer
    fn initialize(e: Env, base_token_id: BytesN<32>, rate: BigInt, target_raid_interval: u32);

    /******** User functions ********/
    // stake base tokens for share tokens
    fn bury(e: Env, amount: BigInt);

    // unstake share tokens for base tokens
    fn unearth(e: Env, amount: BigInt);

    // fund a voyage
    fn voyage(e: Env, voyage_id: i32, num_voyages: BigInt);

    // redeem a finished voyage
    fn end_voyage(e: Env, voyage_id: i32);

    // raid a users voyage
    fn raid(e: Env, voyage_id: i32, user_id: Identifier);

    /******** Read Functions *********/
    fn decimals(e: Env) -> BigInt;

    //returns amount of buried doubloons user owns
    fn get_buried(e: Env, user_id: Identifier) -> BigInt;

    //returns information about a voyage
    fn get_voyage(e: Env, voyage_id: i32) -> VoyageInfo;

    //returns amount of voyages a user has for a specific voyage id
    fn get_u_vygs(e: Env, user_id: Identifier, voyage_id: i32) -> BigInt;

    //gets the block the last raid was performed
    fn get_l_raid(e: Env) -> u32;

    /******** Admin functions ********/
    // create a new voyage (basically OHM bonds, but fun name)
    fn new_voyage(e: Env, voyage_asset: BytesN<32>, price: BigInt, available_voyages: BigInt);

    // transfers contract holdings
    fn xfer_held(e: Env, token_id: BytesN<32>, to: Identifier, amount: BigInt);

    // set the rebase rate
    fn set_rate(e: Env, rate: BigInt);

    // set a new captain
    fn set_capn(e: Env, new_admin: Identifier);

    // set the target raid interval
    fn set_tgt_ri(e: Env, tgt_raid_int: u32);
}

// ****** Contract ******

#[contractimpl]
impl PiratesBayTrait for PiratesBay {
    fn initialize(e: Env, base_token_id: BytesN<32>, rate: BigInt, target_raid_interval: u32) {
        if e.data().has(DataKey::BaseToken) {
            panic!("contract already initialized");
        }
        //check if PiratesBay contract is the admin for base tokens and share token
        /*** Note - currently not possible as you can't read token admins TODO: file issue */

        set_base_token(&e, base_token_id);
        set_total_buried(&e, BigInt::zero(&e));
        set_rate(&e, rate);
        set_index(&e, BigInt::from_i64(&e, SCALER * SCALER));
        set_last_block(&e);
        set_decimals(&e);
        set_target_raid_interval(&e, target_raid_interval);
        write_captain(&e, Identifier::from(e.invoker()));
    }

    fn bury(e: Env, amount: BigInt) {
        let user_id = Identifier::from(e.invoker());
        let new_index = get_new_index(&e);
        set_index(&e, new_index.clone());
        set_last_block(&e);
        burn_token(&e, user_id.clone(), amount.clone());
        let bury_amount = amount * BigInt::from_i64(&e, SCALER * SCALER) / new_index.clone();
        add_buried(&e, user_id, bury_amount);
    }

    fn unearth(e: Env, amount: BigInt) {
        let user = Identifier::from(e.invoker());
        let new_index = get_new_index(&e);
        set_index(&e, new_index.clone());
        set_last_block(&e);
        subtract_buried(&e, user.clone(), amount.clone());
        let mint_amount = amount * new_index / BigInt::from_i64(&e, SCALER * SCALER);
        mint_token(&e, user, mint_amount);
    }

    /// Requires approval for `transfer_from` before running
    fn voyage(e: Env, voyage_id: i32, num_voyages: BigInt) {
        let mut voyage_info = get_voyage(&e, voyage_id);

        //check that the voyage is still available
        if voyage_info.expiration.clone() < e.ledger().sequence().into() {
            panic!("voyage no longer available");
        }
        if voyage_info.max_vygs.clone() - voyage_info.n_embarked.clone() < num_voyages.clone() {
            panic!("not enough voyage available");
        }
        let transfer_amount = voyage_info.price.clone() * num_voyages.clone();
        let user_id = Identifier::from(e.invoker());

        let voyage_asset_client = crate::token::Client::new(&e, voyage_info.vyg_asset.clone());

        voyage_asset_client.xfer_from(
            &Signature::Invoker,
            &BigInt::zero(&e),
            &user_id,
            &get_contract_id(&e),
            &transfer_amount,
        );

        set_user_voyage(&e, user_id, voyage_id, num_voyages.clone());

        //update voyage info
        voyage_info.n_embarked = voyage_info.n_embarked.clone() + num_voyages.clone();

        set_voyage(&e, voyage_id, voyage_info);
    }

    fn end_voyage(e: Env, voyage_id: i32) {
        let user_id = Identifier::from(e.invoker());
        let user_voyage_amt = get_user_voyage(&e, user_id.clone(), voyage_id);
        // NOTE: this may not be necessary, could just let contract panic from null result, but think this is clearer
        if user_voyage_amt == BigInt::zero(&e) {
            panic!("user has no voyages for this voyage id");
        }
        // user gets shells equal to the number of voyages they finished
        mint_token(&e, user_id.clone(), user_voyage_amt);
        remove_user_voyage(&e, user_id, voyage_id);
    }

    fn raid(e: Env, voyage_id: i32, voyager_id: Identifier) {
        // waiting on PRNG pull request https://github.com/stellar/rs-soroban-env/pull/544
        panic!("Not Implemented");

        let user_voyage_amt = get_user_voyage(&e, voyager_id.clone(), voyage_id);
        let raider_id = Identifier::from(e.invoker());
        // NOTE: this may not be necessary, could just let contract panic from null result, but think this is clearer
        if user_voyage_amt == BigInt::zero(&e) {
            panic!("user has no voyages for this voyage id");
        }
        // calculate the amount of shells required to perform the raid
        let raid_cost = user_voyage_amt / BigInt::from_i64(&e, 100);
        // burn the shells
        burn_token(&e, raider_id, raid_cost);

        // calculate probability of raid - scaled by how long it has been since the last raid - we target 1 raid per 600 blocks - probability cant be greater than 1.25%
        // scale by 10000 to avoid floating point math
        let current_block: u32 = e.ledger().sequence().into();
        let mut raid_probability: u32 =
            (current_block - get_last_raid_block(&e)) * 10000 / get_target_raid_interval(&e);
        if raid_probability > 12500 {
            raid_probability = 12500;
        }
        // calculate the max random number for a successful raid based on raid probability
        let max_ok_PRNG = MAX / 1000000 * raid_probability;
        // check if the raid was successful
        // TODO: Use PRNG to determine if raid was successful, waiting on this PR https://github.com/stellar/rs-soroban-env/pull/544
        // let prng_u32: u32 = e.prng_next_u32(RawVal::from_bool(true));
        // if prng_u32 < max_ok_PRNG {
        //     // raid was successful, user loses all their voyages, raider gets shells
        //     remove_user_voyage(&e, voyager_id, voyage_id);
        //     mint_token(&e, raider_id.clone(), user_voyage_amt);
        //     e.events().publish(
        //         (symbol!("raid_won"), voyage_id, voyager_id, current_block),
        //         true,
        //     );
        // } else {
        //     e.events().publish(
        //         (symbol!("raid_won"), voyage_id, voyager_id, current_block),
        //         false,
        //     );
        // }
        //update last raid block
        set_last_raid(&e);
    }

    /******** Read functions *********/
    fn decimals(e: Env) -> BigInt {
        return get_decimals(&e);
    }

    fn get_buried(e: Env, user_id: Identifier) -> BigInt {
        return get_user_buried(&e, user_id);
    }

    fn get_voyage(e: Env, voyage_id: i32) -> VoyageInfo {
        return get_voyage(&e, voyage_id);
    }

    fn get_u_vygs(e: Env, user_id: Identifier, voyage_id: i32) -> BigInt {
        return get_user_voyage(&e, user_id, voyage_id);
    }

    fn get_l_raid(e: Env) -> u32 {
        return get_last_raid_block(&e);
    }

    /******** Admin functions ********/
    fn new_voyage(e: Env, vyg_asset: BytesN<32>, price: BigInt, max_vygs: BigInt) {
        check_captain(&e, &Signature::Invoker);
        let voyage_id = get_last_voyage_id(&e) + 1;
        let voyage_info = VoyageInfo {
            vyg_asset,
            price,
            max_vygs,
            n_embarked: BigInt::zero(&e),
            expiration: e.ledger().sequence() + WEEK_IN_BLOCKS,
        };
        set_voyage(&e, voyage_id, voyage_info);
        set_last_voyage_id(&e, voyage_id);
    }

    // transfers contract holdings
    fn xfer_held(e: Env, token_id: BytesN<32>, to: Identifier, amount: BigInt) {
        //check that invoker is admin
        check_captain(&e, &Signature::Invoker);
        transfer(&e, token_id, to, amount);
    }

    fn set_rate(e: Env, rate: BigInt) {
        //check that invoker is admin
        check_captain(&e, &Signature::Invoker);
        let new_index = get_new_index(&e);
        set_index(&e, new_index);
        set_last_block(&e);
        set_rate(&e, rate);
    }

    fn set_tgt_ri(e: Env, interval: u32) {
        //check that invoker is admin
        check_captain(&e, &Signature::Invoker);
        set_target_raid_interval(&e, interval);
    }

    fn set_capn(e: Env, new_admin: Identifier) {
        //check that invoker is admin
        check_captain(&e, &Signature::Invoker);
        write_captain(&e, new_admin);
    }
}

/****** Objects *******/
#[derive(Clone)]
#[contracttype]
pub struct VoyageInfo {
    pub vyg_asset: BytesN<32>, //asset being used to fund the voyage
    pub price: BigInt,         //price of shells in voyage asset
    pub max_vygs: BigInt,      //max number of voyages that can be entered for shells
    pub n_embarked: BigInt,    //number of voyages that have been embarked on
    pub expiration: u32,       //block the voyage expires on
}

#[derive(Clone)]
#[contracttype]
// unsure why I had to use pub here but on on voyage info TODO - ask mootz
pub struct VoyageKey {
    pub user_id: Identifier, //user public key
    pub voyage_id: i32,      //id of the voyage entered by the user
}
