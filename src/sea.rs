use crate::{
    admin::{check_admin, write_administrator},
    data_management::{
        get_base_token_client, get_decimals, get_last_voyage_id, get_new_index,
        get_share_token_client, get_total_shares, get_user_voyage, get_voyage, remove_user_voyage,
        set_base_token, set_decimals, set_index, set_last_block, set_last_voyage_id, set_rate,
        set_share_token, set_total_shares, set_user_voyage, set_voyage, SCALER,
    },
};
use soroban_auth::{Identifier, Signature};
use soroban_sdk::{contractimpl, contracttype, BigInt, BytesN, Env};

// ****** Contract Storage *****

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    BaseToken,             // address of the shell token
    ShareToken,            // address of the seashell token
    ShareTotal,            // total shares issued
    Rate,                  // per-block rebase rate for seashell tokens
    Index,                 // rebase index for seashell tokens
    LastBlock,             // last block the index was updated at
    Admin,                 // admin address
    Decimals,              // decimals
    Voyages(i32),          // struct of voyage information
    LastVoyage,            //stores the id of the last voyage
    UserVoyage(VoyageKey), // struct of user voyages
}

fn burn_shares(e: &Env, from: Identifier, amount: BigInt) {
    let total = get_total_shares(e);
    let share_token_client = get_share_token_client(&e);
    share_token_client.burn(&Signature::Invoker, &BigInt::zero(&e), &from, &amount);
    set_total_shares(e, total - amount);
}

fn mint_shares(e: &Env, to: Identifier, amount: BigInt) {
    let total = get_total_shares(e);
    let share_token_client = get_share_token_client(&e);
    share_token_client.mint(&Signature::Invoker, &BigInt::zero(&e), &to, &amount);
    set_total_shares(e, total + amount);
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
pub struct Sea;
pub trait SeaTrait {
    // initialize the contract with caller as the admin - rate is the raw integer, rate_scaler scales the rate to its decimal form
    fn initialize(
        e: Env,
        share_token_id: BytesN<32>,
        base_token_id: BytesN<32>,
        rate: BigInt,
        rate_scaler: BigInt,
    );

    /******** User functions ********/
    // stake base tokens for share tokens
    fn sink(e: Env, amount: BigInt);

    // unstake share tokens for base tokens
    fn dredge(e: Env, amount: BigInt);

    // fund a voyage
    fn voyage(e: Env, voyage_id: i32, num_voyages: BigInt);

    // redeem a finished voyage
    fn end_voyage(e: Env, voyage_id: i32);

    /******** Read Functions *********/
    fn decimals(e: Env) -> BigInt;

    //returns information about a voyage
    fn get_voyage(e: Env, voyage_id: i32) -> VoyageInfo;

    //returns amount of voyages a user has for a specific voyage id
    fn get_u_vygs(e: Env, user_id: Identifier, voyage_id: i32) -> BigInt;

    /******** Admin functions ********/
    // create a new voyage (basically OHM bonds, but fun name)
    fn new_voyage(e: Env, voyage_asset: BytesN<32>, price: BigInt, available_voyages: BigInt);

    // transfers contract holdings
    fn xfer_held(e: Env, token_id: BytesN<32>, to: Identifier, amount: BigInt);

    // set the rebase rate
    fn set_rate(e: Env, rate: BigInt);

    // set a new admin
    fn set_admin(e: Env, new_admin: Identifier);
}

// ****** Contract ******

#[contractimpl]
impl SeaTrait for Sea {
    fn initialize(
        e: Env,
        share_token_id: BytesN<32>,
        base_token_id: BytesN<32>,
        rate: BigInt,
        rate_scaler: BigInt,
    ) {
        if e.data().has(DataKey::BaseToken) {
            panic!("contract already initialized");
        }
        //check that sea contract is the admin for base tokens and share token
        /*** Note - currently not possible as you can't read token admins TODO: file issue */

        set_base_token(&e, base_token_id);
        set_share_token(&e, share_token_id);
        set_total_shares(&e, BigInt::zero(&e));
        set_rate(&e, rate, rate_scaler);
        set_index(&e, BigInt::from_i64(&e, SCALER));
        set_last_block(&e);
        set_decimals(&e);
        write_administrator(&e, Identifier::from(e.invoker()));
    }

    fn sink(e: Env, amount: BigInt) {
        let new_index = get_new_index(&e);
        set_index(&e, new_index.clone());
        set_last_block(&e);
        burn_token(&e, Identifier::from(e.invoker()), amount.clone());
        let mint_amount = amount * BigInt::from_i64(&e, SCALER) / new_index.clone();
        mint_shares(&e, Identifier::from(e.invoker()), mint_amount);
    }

    fn dredge(e: Env, amount: BigInt) {
        let user = Identifier::from(e.invoker());
        let new_index = get_new_index(&e);
        set_index(&e, new_index.clone());
        set_last_block(&e);
        burn_shares(&e, user.clone(), amount.clone());
        let mint_amount = amount * new_index / BigInt::from_i64(&e, SCALER);
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

        //NOTE: we use invoker as the user_id here because we fetch the voyage info with the invoker in end_voyage() and the sig_id does not equal the invoker id
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

    /******** Read functions *********/
    fn decimals(e: Env) -> BigInt {
        return get_decimals(&e);
    }

    fn get_voyage(e: Env, voyage_id: i32) -> VoyageInfo {
        return get_voyage(&e, voyage_id);
    }

    fn get_u_vygs(e: Env, user_id: Identifier, voyage_id: i32) -> BigInt {
        return get_user_voyage(&e, user_id, voyage_id);
    }

    /******** Admin functions ********/
    fn new_voyage(e: Env, vyg_asset: BytesN<32>, price: BigInt, max_vygs: BigInt) {
        check_admin(&e, &Signature::Invoker);
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
        check_admin(&e, &Signature::Invoker);
        transfer(&e, token_id, to, amount);
    }

    // set the rebase rate
    fn set_rate(e: Env, rate: BigInt) {
        //check that invoker is admin
        check_admin(&e, &Signature::Invoker);
        let new_index = get_new_index(&e);
        set_index(&e, new_index);
        set_last_block(&e);
        set_rate(&e, rate, BigInt::from_i64(&e, SCALER));
    }

    // set a new admin
    fn set_admin(e: Env, new_admin: Identifier) {
        //check that invoker is admin
        check_admin(&e, &Signature::Invoker);
        write_administrator(&e, new_admin);
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
