#![no_std]

mod vault {
    soroban_sdk::contractimport!(
        file = "../target/wasm32-unknown-unknown/release/xycloans_vault_interface.wasm"
    );
}

mod contract;
mod events;
mod execution;
mod storage;
mod token_utility;
mod types;
