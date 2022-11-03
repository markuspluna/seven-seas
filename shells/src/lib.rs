#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

mod accounting;
mod auth;

pub mod sea;
pub mod token {
    soroban_sdk::contractimport!(file = "./soroban_token_spec.wasm");
}
