use soroban_bag::{
    sea::{Sea, SeaClient},
    token,
};

use rand::{thread_rng, RngCore};
use soroban_auth::Identifier;
use soroban_sdk::{AccountId, BytesN, Env, IntoVal};

pub fn generate_contract_id(e: &Env) -> BytesN<32> {
    let mut id: [u8; 32] = Default::default();
    thread_rng().fill_bytes(&mut id);
    BytesN::from_array(e, &id)
}

pub fn create_usdc_token_contract(
    e: &Env,
    contract_id: &BytesN<32>,
    admin: &AccountId,
) -> token::Client {
    e.register_contract_token(contract_id);

    let token = token::Client::new(e, contract_id);
    token.init(
        &Identifier::Account(admin.clone()),
        &token::TokenMetadata {
            name: "USDC".into_val(e),
            symbol: "USDC".into_val(e),
            decimals: 7,
        },
    );
    token
}

pub fn create_base_token_contract(
    e: &Env,
    contract_id: &BytesN<32>,
    admin: &AccountId,
) -> token::Client {
    e.register_contract_token(contract_id);

    let token = token::Client::new(e, contract_id);
    token.init(
        &Identifier::Account(admin.clone()),
        &token::TokenMetadata {
            name: "shell".into_val(e),
            symbol: "SHL".into_val(e),
            decimals: 7,
        },
    );
    token
}

pub fn create_share_token_contract(
    e: &Env,
    contract_id: &BytesN<32>,
    admin: &AccountId,
) -> token::Client {
    e.register_contract_token(contract_id);

    let token = token::Client::new(e, contract_id);
    token.init(
        &Identifier::Account(admin.clone()),
        &token::TokenMetadata {
            name: "seashell".into_val(e),
            symbol: "SSHL".into_val(e),
            decimals: 7,
        },
    );
    token
}

pub fn create_sea_contract(e: &Env, contract_id: &BytesN<32>) -> SeaClient {
    e.register_contract(contract_id, Sea {});
    return SeaClient::new(e, contract_id);
}
pub const SCALER: i64 = 10000000;
