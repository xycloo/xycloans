use soroban_sdk::{Address, Env, Symbol};

const XYCLOAN: Symbol = Symbol::short("XYCLOAN");

pub(crate) fn fees_deposited(env: &Env, amount: i128) {
    let topics = (XYCLOAN, Symbol::new(env, "deposit_fees"));
    env.events().publish(topics, amount);
}

pub(crate) fn deposited(env: &Env, from: Address, amount: i128) {
    let topics = (XYCLOAN, Symbol::short("deposit"));
    env.events().publish(topics, (from, amount));
}

pub(crate) fn matured_withdrawn(env: &Env, addr: Address) {
    let topics = (XYCLOAN, Symbol::new(env, "withdraw_matured"));
    env.events().publish(topics, addr);
}

pub(crate) fn matured_updated(env: &Env, addr: Address) {
    let topics = (XYCLOAN, Symbol::new(env, "update_fee_rewards"));
    env.events().publish(topics, addr);
}

pub(crate) fn withdrawn(env: &Env, from: Address, amount: i128) {
    let topics = (XYCLOAN, Symbol::short("withdraw"));
    env.events().publish(topics, (from, amount));
}
