#![no_std]

mod contract;
mod storage;
mod types;
mod events;

mod pool {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_pool.wasm");
}
