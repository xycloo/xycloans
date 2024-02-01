use soroban_sdk::{symbol_short, Address, Env};

pub(crate) fn deposited(env: &Env, from: Address, amount: i128) {
    let topics = (symbol_short!("deposit"), from);
    env.events().publish(topics, amount);
}

pub(crate) fn matured_withdrawn(env: &Env, addr: Address, withdrawn: i128) {
    let topics = (symbol_short!("collect"), addr);
    env.events().publish(topics, withdrawn);
}

pub(crate) fn new_fees(env: &Env, addr: Address, matured: i128) {
    let topics = (symbol_short!("newfee"), addr);
    env.events().publish(topics, matured);
}

pub(crate) fn withdrawn(env: &Env, from: Address, amount: i128) {
    let topics = (symbol_short!("withdrawn"), from);
    env.events().publish(topics, amount);
}

pub(crate) fn loan_successful(env: &Env, receiver_contract: Address, amount: i128) {
    let topics = (symbol_short!("borrow"), receiver_contract);
    env.events().publish(topics, amount);
}
