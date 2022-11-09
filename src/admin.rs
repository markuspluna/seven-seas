use crate::sea::DataKey;
use soroban_auth::{Identifier, Signature};
use soroban_sdk::Env;

fn read_administrator(e: &Env) -> Identifier {
    let key = DataKey::Admin;
    e.data().get_unchecked(key).unwrap()
}

pub fn write_administrator(e: &Env, id: Identifier) {
    let key = DataKey::Admin;
    e.data().set(key, id);
}

pub fn check_admin(e: &Env, auth: &Signature) {
    let auth_id = auth.identifier(&e);
    if auth_id != read_administrator(&e) {
        panic!("not authorized by admin")
    }
}
