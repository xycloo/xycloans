use soroban_sdk::{symbol_short, Address, Env, IntoVal};


mod moderc3156 {
    use soroban_sdk::contractimport;
    contractimport!(file = "../target/wasm32-unknown-unknown/release/moderc3156.wasm");
}

pub(crate) fn invoke_receiver(e: &Env, id: &Address) {
    e.invoke_contract::<()>(id, &symbol_short!("exec_op"), ().into_val(e));
}

pub(crate) fn invoke_receiver_moderc3156(e: &Env, id: &Address, token: &Address, amount: &i128, fee: &i128) {
    moderc3156::Client::new(e, &id).exec_op(&e.current_contract_address(), token, amount, fee)
}
