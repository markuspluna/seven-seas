// REVIEW: Not needed
use core::ops::{Shl, Shr, ShrAssign};

use crate::{
    accounting::{
        decimals_in_int, get_decimals, get_new_index, get_total_shares, set_decimals, set_index,
        set_last_block, set_total_shares,
    },
    auth::{check_admin, write_administrator},
};
use soroban_auth::{Identifier, Signature};
use soroban_sdk::{contractimpl, contracttype, BigInt, BytesN, Env};

// ****** Contract Storage *****

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    BaseToken,  // address of the shell token
    ShareToken, // address of the seashell token
    ShareTotal, // total shares issued
    Rate,       // per-block rebase rate for seashell tokens
    Index,      // rebase index for seashell tokens
    LastBlock,  // last block the index was updated at
    Admin,      // admin address
    Decimals,   // decimals
}

// REVIEW: Bit confusing due to some helpers listed here and some separated into files
//         Consider cleaning up homes for these 

fn get_base_token(e: &Env) -> BytesN<32> {
    e.data().get_unchecked(DataKey::BaseToken).unwrap()
}

fn get_base_token_client(e: &Env) -> crate::token::Client {
    let id = get_base_token(e);
    crate::token::Client::new(e, id)
}

fn get_share_token(e: &Env) -> BytesN<32> {
    e.data().get_unchecked(DataKey::ShareToken).unwrap()
}

fn get_share_token_client(e: &Env) -> crate::token::Client {
    let id = get_share_token(e);
    crate::token::Client::new(e, id)
}

fn set_base_token(e: &Env, contract_id: BytesN<32>) {
    e.data().set(DataKey::BaseToken, contract_id);
}

fn set_share_token(e: &Env, contract_id: BytesN<32>) {
    e.data().set(DataKey::ShareToken, contract_id);
}

fn set_rate(e: &Env, rate: BigInt) {
    e.data().set(DataKey::Rate, rate)
}

// REVIEW: Picky: (but somewhat important for contracts)
//         Consider having consistent variable orders to avoid whoopsies
fn burn_shares(e: &Env, amount: BigInt, from: Identifier) {
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

fn burn_token(e: &Env, amount: BigInt, from: Identifier) {
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
// REVIEW: Run `cargo fmt` to format your project (I think it will enforce spacing?)
pub struct Sea;
pub trait SeaTrait {
    // initialize the contract with caller as the admin
    fn initialize(e: Env, share_token_id: BytesN<32>, base_token_id: BytesN<32>, rate: BigInt);

    // REVIEW: this is a bit of a misnomer? Nothing in here is private
    /******** Public functions ********/
    // stake base tokens for share tokens
    fn sink(e: Env, amount: BigInt);

    // unstake share tokens for base tokens
    fn dredge(e: Env, amount: BigInt);

    /******** Read Functions *********/
    fn decimals(e: Env) -> BigInt;
    /******** Admin functions ********/
    // transfers contract holdings
    fn xfer_held(e: Env, token_id: BytesN<32>, to: Identifier, amount: BigInt);

    // REVIEW: Picky: Recommend conforming to rust naming conventions (set_rate)
    // set the rebase rate
    fn setRate(e: Env, rate: BigInt);

    // set a new admin
    fn setAdmin(e: Env, newAdmin: Identifier);
}

// ****** Contract ******

#[contractimpl]
impl SeaTrait for Sea {
    fn initialize(e: Env, share_token_id: BytesN<32>, base_token_id: BytesN<32>, rate: BigInt) {
        if e.data().has(DataKey::BaseToken) {
            panic!("contract already initialized");
        }
        //check that sea contract is the admin for base tokens and share token
        /*** Note - currently not possible as you can't read token admins TODO: file issue */

        //set tokens
        set_base_token(&e, base_token_id);
        set_share_token(&e, share_token_id);

        //set remaining data
        set_total_shares(&e, BigInt::zero(&e));
        set_rate(&e, rate); // REVIEW: Should be noted this has to contain `decimals` decimals
        set_index(&e, decimals_in_int(&e));
        set_last_block(&e);
        set_decimals(&e);
        write_administrator(&e, Identifier::from(e.invoker()));
    }

    // REVIEW: Function comments should be applied to the trait
    //         In-line comments should be for anything that is non-obvious
    //         It's a fair argument that mine was overcommented, but due to context I added
    //         lots of extra info regarding authentication

    // stake base tokens for share tokens
    fn sink(e: Env, amount: BigInt) {
        //get new index
        let new_index = get_new_index(&e);
        //update stored index
        set_index(&e, new_index.clone());
        //update stored lastblock
        set_last_block(&e);

        //burn base tokens from caller, token contract will panic if balance is insufficient
        // REVIEW: I've never actually read Ohm contracts. Do they actually burn the token when you "stake"?
        burn_token(&e, amount.clone(), Identifier::from(e.invoker()));
        //mint share share tokens to caller
        let mint_amount = amount * decimals_in_int(&e) / new_index.clone();
        std::println!("new_index: {}", new_index); // REVIEW: clear out the debugs
        mint_shares(&e, Identifier::from(e.invoker()), mint_amount);
    }

    // unstake share tokens for base tokens
    fn dredge(e: Env, amount: BigInt) {
        //get new index
        // REVIEW: it feels like the three of these can be consolidated to a `update_index` fn
        let new_index = get_new_index(&e);
        std::println!("new_index: {}", new_index);

        //update stored index
        set_index(&e, new_index.clone());
        //update stored lastblock
        set_last_block(&e);

        //burn share tokens from caller, token contract will panic if balance is insufficient
        burn_shares(&e, amount.clone(), Identifier::from(e.invoker()));
        //mint base tokens to caller
        std::println!("amount {}", amount);
        // REVIEW: The writeup / README should include a note on this math
        let mint_amount = amount * new_index / decimals_in_int(&e);
        std::println!("amount {}", mint_amount);

        mint_token(&e, Identifier::from(e.invoker()), mint_amount);
    }

    /******** Read functions *********/
    fn decimals(e: Env) -> BigInt {
        return get_decimals(&e);
    }

    /******** Admin functions ********/
    // REVIEW: Not sure I follow what the contract will holding or why this method is needed
    // transfers contract holdings
    fn xfer_held(e: Env, token_id: BytesN<32>, to: Identifier, amount: BigInt) {
        //check that invoker is admin
        check_admin(&e, &Signature::Invoker);
        // transfer tokens
        transfer(&e, token_id, to, amount);
    }

    // set the rebase rate
    fn setRate(e: Env, rate: BigInt) {
        //check that invoker is admin
        check_admin(&e, &Signature::Invoker);
        //get new index
        let new_index = get_new_index(&e);
        //update stored index
        set_index(&e, new_index);
        //update stored lastblock
        set_last_block(&e);
        //set new rate
        set_rate(&e, rate);
    }

    // set a new admin
    fn setAdmin(e: Env, new_admin: Identifier) {
        //check that invoker is admin
        check_admin(&e, &Signature::Invoker);
        //set new admin
        write_administrator(&e, new_admin);
    }
}
