#![no_std]
mod token {
    soroban_sdk::contractimport!(file = "./soroban_token_spec.wasm");
}
mod interface;
mod types;

#[cfg(test)]
mod test;
