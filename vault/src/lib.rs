#![no_std]

mod balance;
mod contract;
mod math;
mod rewards;
mod storage;
mod token_utility;
mod types;

mod token {
    soroban_sdk::contractimport!(file = "../soroban_token_spec.wasm");
}

mod flash_loan {
    soroban_sdk::contractimport!(
        file = "../target/wasm32-unknown-unknown/release/xycloans_flash_loan.wasm"
    );
}
