use soroban_sdk::{vec, Address, BytesN, Env, Vec};

use crate::{
    token,
    types::{BatchKey, BatchObj, DataKey},
};

pub fn get_contract_addr(e: &Env) -> Address {
    e.current_contract_address()
}

fn put_tot_supply(e: &Env, supply: i128) {
    let key = DataKey::TotSupply;
    e.storage().set(&key, &supply);
}

pub fn get_tot_supply(e: &Env) -> i128 {
    let key = DataKey::TotSupply;
    e.storage().get(&key).unwrap_or(Ok(0)).unwrap()
}

pub fn put_token_id(e: &Env, token_id: BytesN<32>) {
    let key = DataKey::TokenId;
    e.storage().set(&key, &token_id);
}

pub fn get_token_id(e: &Env) -> BytesN<32> {
    let key = DataKey::TokenId;
    e.storage().get(&key).unwrap().unwrap()
}

pub fn put_flash_loan(e: &Env, id: Address) {
    let key = DataKey::FlashLoan;
    e.storage().set(&key, &id);
}

pub fn get_flash_loan(e: &Env) -> Address {
    let key = DataKey::FlashLoan;
    e.storage().get(&key).unwrap().unwrap()
}

pub fn put_flash_loan_bytes(e: &Env, id: BytesN<32>) {
    let key = DataKey::FlashLoanB;
    e.storage().set(&key, &id);
}

pub fn get_flash_loan_bytes(e: &Env) -> BytesN<32> {
    let key = DataKey::FlashLoanB;
    e.storage().get(&key).unwrap().unwrap()
}

pub fn get_token_balance(e: &Env) -> i128 {
    let contract_id = get_token_id(e);
    let client = token::Client::new(e, &contract_id);

    client.balance(&get_contract_addr(e)) + client.balance(&get_flash_loan(e))
}

pub fn transfer(e: &Env, to: &Address, amount: i128) {
    let client = token::Client::new(e, &get_token_id(e));
    client.transfer(&get_contract_addr(e), to, &amount);
}

pub fn transfer_in_vault(e: &Env, from: &Address, amount: &i128) {
    let client = token::Client::new(e, &get_token_id(e));
    let vault_addr = get_contract_addr(e);

    client.transfer(from, &vault_addr, amount);
}

pub fn has_administrator(e: &Env) -> bool {
    let key = DataKey::Admin;
    e.storage().has(&key)
}

pub fn read_admin(e: &Env) -> Address {
    let key = DataKey::Admin;
    e.storage().get_unchecked(&key).unwrap()
}

pub fn write_administrator(e: &Env, id: Address) {
    let key = DataKey::Admin;
    e.storage().set(&key, &id);
}

pub fn mint_shares(e: &Env, to: Address, shares: i128, deposit: i128) -> i128 {
    let tot_supply = get_tot_supply(e);
    put_tot_supply(e, tot_supply + shares);

    let n = get_increment(e, to.clone());
    let key = DataKey::Batch(BatchKey(to.clone(), n));

    let val = BatchObj {
        init_s: shares,
        deposit,
        curr_s: shares,
    };

    put_increment(e, to, n + 1);
    e.storage().set(&key, &val);

    n
}

pub fn get_user_batches(e: &Env, id: Address) -> Vec<i128> {
    let n = get_increment(e, id);
    let mut batches = vec![e];
    for i in 0..n {
        batches.push_back(i)
    }

    batches
}

pub fn burn_shares(e: &Env, to: Address, shares: i128, batch_n: i128) {
    let tot_supply = get_tot_supply(e);
    let key = DataKey::Batch(BatchKey(to.clone(), batch_n));

    let mut batch: BatchObj = e.storage().get(&key).unwrap().unwrap();
    batch.curr_s -= shares;
    put_tot_supply(e, tot_supply - shares);

    if batch.curr_s == 0 {
        e.storage().remove(&key); // if there are 0 shares remove the batch
    } else {
        e.storage().set(&key, &batch);
    }
}

pub fn put_increment(e: &Env, id: Address, n: i128) {
    e.storage().set(&DataKey::Increment(id), &n);
}

pub fn get_increment(e: &Env, id: Address) -> i128 {
    e.storage()
        .get(&DataKey::Increment(id))
        .unwrap_or(Ok(0))
        .unwrap()
}
