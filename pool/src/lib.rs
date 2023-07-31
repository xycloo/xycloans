#![no_std]

mod balance;
mod contract;
pub mod math;
mod rewards;
mod storage;
mod token_utility;
mod types;
mod events;
mod execution;

mod flash_loan {
    soroban_sdk::contractimport!(
        file = "../target/wasm32-unknown-unknown/release/xycloans_flash_loan.wasm"
    );
}

pub fn compute_fee(amount: &i128) -> i128 {
    amount / 2000 // 0.05%, still TBD
}
