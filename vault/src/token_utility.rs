use soroban_sdk::{Address, Env};

use crate::{
    storage::{get_flash_loan, get_token_id},
    token,
};

pub fn transfer(e: &Env, client: &token::Client, to: &Address, amount: i128) {
    client.transfer(&e.current_contract_address(), to, &amount);
}

pub fn transfer_into_flash_loan(e: &Env, client: &token::Client, from: &Address, amount: &i128) {
    client.transfer(from, &get_flash_loan(e), amount);
}

pub fn get_token_client(e: &Env) -> token::Client {
    token::Client::new(e, &get_token_id(e))
}

pub fn read_flash_loan_balance(e: &Env, client: &token::Client) -> i128 {
    client.balance(&get_flash_loan(e))
}
