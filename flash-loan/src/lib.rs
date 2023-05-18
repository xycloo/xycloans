#![no_std]
//mod token {
//    soroban_sdk::contractimport!(file = "../soroban_token_spec.wasm");
//}

mod vault {
    soroban_sdk::contractimport!(file = "../contracts-wasm/xycloans_fl_vault.wasm");
}

mod contract;
mod execution;
mod storage;
mod token_utility;
mod types;
