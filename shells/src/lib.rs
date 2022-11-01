#![no_std]
use soroban_auth::{Identifier, Signature, verify};
use soroban_sdk::{contractimpl, symbol, vec, Env, Symbol, Vec};


// ****** Contract Storage *****

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    BaseToken(Identifier) , // address of the shell token
    ShareToken(Identifier), // address of the seashell token
    TotalShares(i64), // total shares issued
    Rate(i64), // rebase rate for seashell tokens
    Index(i64), // rebase index for seashell tokens
    Admin(Identifier) // admin address
}

fn get_contract_id(e: &Env) -> Identifier {
    Identifier::Contract(e.get_current_contract().into())
}

fn get_base_token(e: &Env) -> BytesN<32> {
    e.contract_data().get_unchecked(DataKey::BaseToken).unwrap()
}

 fn get_base_token_client(e: &Env) -> crate::token::Client {
    let id = get_base_token(e);
    crate::token::Client::new(e, id)
 }

fn get_share_token(e: &Env) -> BytesN<32> {
    e.contract_data().get_unchecked(DataKey::ShareToken).unwrap()
}

 fn get_share_token_client(e: &Env) -> crate::token::Client {
    let id = get_share_token(e);
    crate::token::Client::new(e, id)
 }

fn get_total_shares(e: &Env) -> BigInt {
    e.contract_data()
        .get_unchecked(DataKey::TotalShares)
        .unwrap()
}

fn get_rate(e: &Env) -> BigInt {
    e.contract_data().get_unchecked(DataKey::Rate).unwrap()
}

fn get_index(e: &Env) -> BigInt {
    e.contract_data().get_unchecked(DataKey::Index).unwrap()
}

fn get_admin(e: &Env) -> BytesN<32> {
    e.contract_data().get_unchecked(DataKey::Admin).unwrap()
}

fn get_contract_id(e: &Env) -> Identifier {
    Identifier::Contract(e.get_current_contract().into())
}



fn put_base_token(e: &Env, contract_id: BytesN<32>) {
    e.contract_data().set(DataKey::BaseToken, contract_id);
}

fn put_share_token(e: &Env, contract_id: BytesN<32>) {
    e.contract_data().set(DataKey::ShareToken, contract_id);
}

fn put_total_shares(e: &Env, amount: BigInt) {
    e.contract_data().set(DataKey::TotalShares, amount)
}

fn put_rate(e: &Env, rate: BigInt) {
    e.contract_data().set(DataKey::Rate, rate)
}

fn put_index(e: &Env, index: BigInt) {
    e.contract_data().set(DataKey::Index, index)
}

fn put_admin(e: &Host, id: Identifier) -> Result<(), HostError> {
    let key = DataKey::Admin;
    e.contract_data().set(DataKey::Admin, id.try_into_val(e)?)?;
}

fn burn_shares(e: &Env, amount: BigInt) {
    let total = get_total_shares(e);
    let share_contract_id = get_token_share(e);
    token::burn(
        e,
        &share_contract_id,
        &Authorization::Contract,
        &get_contract_id(e),
        &amount,
    );
    put_total_shares(e, total - amount);
}

fn mint_shares(e: &Env, to: Identifier, amount: BigInt) {
    let total = get_total_shares(e);
    let share_contract_id = get_token_share(e);
    token::mint(
        e,
        &share_contract_id,
        &Authorization::Contract,
        &to,
        &amount,
    );
    put_total_shares(e, total + amount);
}

fn burn_token(e: &Env, amount: BigInt) {
    let share_contract_id = get_base_token(e);
    token::burn(
        e,
        &share_contract_id,
        &Authorization::Contract,
        &get_contract_id(e),
        &amount,
    );
}

fn mint_token(e: &Env, to: Identifier, amount: BigInt) {
    let share_contract_id = get_base_token(e);
    token::mint(
        e,
        &share_contract_id,
        &Authorization::Contract,
        &to,
        &amount,
    );
}

fn transfer(e: &Env, contract_id: BytesN<32>, to: Identifier, amount: BigInt) {
    token::Client::new(&e, contract_id).xfer(&Signature::Invoker, &BigInt::zero(&e), &to, &amount);
}


pub trait SeaTrait {
    // initialize the contract with an admin
    fn initialize(e: Env, admin: Identifier);
    
    // stake tokens for shares
    fn stake(e: Env, amount: BigInt);

    // unstake shares for tokens
    fn unstake(e: Env, amount: BigInt)>;

    // transfers contract holdings
    fn transfer(e: Env,contract_id: BytesN<32>, to: Identifier, amount: BigInt, out_b: BigInt);

    fn withdraw(e: Env, to: Identifier);
}

pub struct Sea;
// ****** Contract ******

#[contractimpl]
impl SeaTrait for Sea {
    pub fn initialize(e: Env,share_token_id: BytesN<32>,base_token_id:BytesN<32>,rate:BigInt) {
        if e.data().has(DataKey::BaseToken) {
            panic!("contract already initialized");
        }

        //set tokens
        put_base_token(&e,base_token_id)
        put_share_token(&e,share_token_id)

        //set contract is admin for both tokens - contract will panic if initializer is not token admin
        let base_token_client = get_base_token_client(&e)
        let share_token_client = get_share_token_client(&e)
        let contract_id = get_contract_id(&e)
        base_token_client.set_admin(&e,&Signature::Invoker,&BigInt::zero(&env),contract_id)
        share_token_client.set_admin(&e,&Signature::Invoker,&BigInt::zero(&env),contract_id)

        //set remaining data
        put_total_shares(&e,BigInt::zero(&env))
        put_rate(&env,rate)
        put_index(&env,BigInt::from_i64(&e,1))
        put_admin(&env,Identifier::from(e.invoker()))
    }
    // stake tokens 
    pub fn Sink(env: Env, to: Symbol) -> Vec<Symbol> {
        vec![&env, symbol!("Hello"), to]
    }
}

// ****** Helpers *****



#[cfg(test)]
mod test;