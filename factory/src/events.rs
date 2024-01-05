use soroban_sdk::{Env, Address, symbol_short};

pub(crate) fn deployed_pool(env: &Env, contract: &Address) {
    let topics = (symbol_short!("deployed"), );
    env.events().publish(topics, contract);
}