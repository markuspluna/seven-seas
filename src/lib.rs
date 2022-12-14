#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

mod captain;
mod data_management;

pub mod seven_seas;
pub mod token {
    soroban_sdk::contractimport!(file = "./soroban_token_spec.wasm");
}
