#![no_std]

mod balance;
mod contract;
mod math;
mod rewards;
mod storage;
mod token_utility;
mod types;

mod flash_loan {
    soroban_sdk::contractimport!(
        file = "../target/wasm32-unknown-unknown/release/xycloans_flash_loan.wasm"
    );
}
