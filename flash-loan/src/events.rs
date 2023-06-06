use soroban_sdk::{Address, Env, Symbol};

const XYCLOAN: Symbol = Symbol::short("XYCLOAN");

pub(crate) fn loan_successful(env: &Env, receiver_contract: Address, amount: i128) {
    let topics = (XYCLOAN, Symbol::short("borrow"));
    env.events().publish(topics, (receiver_contract, amount));
}

pub(crate) fn withdraw(env: &Env, amount: i128, to: Address) {
    let topics = (XYCLOAN, Symbol::short("withdraw"));
    env.events().publish(topics, (amount, to));
}
