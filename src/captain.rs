use crate::seven_seas::DataKey;
use soroban_auth::{Identifier, Signature};
use soroban_sdk::Env;

fn read_captain(e: &Env) -> Identifier {
    let key = DataKey::Admin;
    e.data().get_unchecked(key).unwrap()
}

pub fn write_captain(e: &Env, id: Identifier) {
    let key = DataKey::Admin;
    e.data().set(key, id);
}

pub fn check_captain(e: &Env, auth: &Signature) {
    let auth_id = auth.identifier(&e);
    if auth_id != read_captain(&e) {
        panic!("not authorized by captain")
    }
}
