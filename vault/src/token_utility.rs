use crate::storage::{get_flash_loan, get_token_id};
use soroban_sdk::{token, Address, Env};

pub(crate) fn transfer(e: &Env, client: &token::Client, to: &Address, amount: i128) {
    client.transfer(&e.current_contract_address(), to, &amount);
}

pub(crate) fn transfer_into_flash_loan(
    e: &Env,
    client: &token::Client,
    from: &Address,
    amount: &i128,
) {
    client.transfer(from, &get_flash_loan(e), amount);
}

pub(crate) fn get_token_client(e: &Env) -> token::Client {
    token::Client::new(e, &get_token_id(e))
}


