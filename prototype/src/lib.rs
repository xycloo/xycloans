#![no_std]
mod token {
    soroban_sdk::contractimport!(file = "./soroban_token_spec.wasm");
}
mod contract;
mod types;
mod utils;

#[cfg(test)]
mod test;
