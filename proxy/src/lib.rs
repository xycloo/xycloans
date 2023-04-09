#![no_std]

mod contract;
mod storage;
mod types;

mod vault {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_fl_vault.wasm");
}

mod flash_loan {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_flash_loan.wasm");
}
