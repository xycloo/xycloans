#![no_std]

mod contract;
mod storage;
mod types;

mod token {
    soroban_sdk::contractimport!(file = "../soroban_token_spec.wasm");
}

mod flash_loan {
    soroban_sdk::contractimport!(file = "../target/wasm32-unknown-unknown/release/flash_loan.wasm");
}
