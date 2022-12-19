#![no_std]
mod token {
    soroban_sdk::contractimport!(file = "../soroban_token_spec.wasm");
}

/*mod vault {
    soroban_sdk::contractimport!(
        file = "../target/wasm32-unknown-unknown/release/soroban_vault.wasm"
    );
}*/

mod contract;
mod types;
mod utils;

//#[cfg(test)]
//mod test;
