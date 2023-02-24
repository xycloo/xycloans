use soroban_sdk::{Address, BytesN, Env, Vec};

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
    client.xfer(&get_contract_addr(e), to, &amount);
}

pub fn transfer_in_vault(e: &Env, from: &Address, amount: &i128) {
    let client = token::Client::new(e, &get_token_id(e));
    let vault_addr = get_contract_addr(e);

    client.xfer_from(&vault_addr, from, &vault_addr, amount);
}

pub fn has_administrator(e: &Env) -> bool {
    let key = DataKey::Admin;
    e.storage().has(&key)
}

pub fn _read_administrator(e: &Env) -> Address {
    let key = DataKey::Admin;
    e.storage().get_unchecked(&key).unwrap()
}

pub fn write_administrator(e: &Env, id: Address) {
    let key = DataKey::Admin;
    e.storage().set(&key, &id);
}

pub fn mint_shares(e: &Env, to: Address, shares: i128, deposit: i128) -> u64 {
    let tot_supply = get_tot_supply(e);
    put_tot_supply(e, tot_supply + shares);

    let ts = e.ledger().timestamp();
    let key = DataKey::Batch(BatchKey(to.clone(), ts));

    let val = BatchObj {
        init_s: shares,
        deposit,
        curr_s: shares,
    };

    add_user_batch(e, to, ts);
    e.storage().set(&key, &val);

    ts
}

pub fn get_user_batches(e: &Env, id: Address) -> Vec<u64> {
    let key = DataKey::Batches(id);
    e.storage()
        .get(&key)
        .unwrap_or_else(|| Ok(Vec::new(e)))
        .unwrap()
}

pub fn add_user_batch(e: &Env, id: Address, batch_ts: u64) {
    let mut batches = get_user_batches(e, id.clone());
    batches.push_front(batch_ts);

    let key = DataKey::Batches(id);
    e.storage().set(&key, &batches);
}

pub fn remove_user_batch(e: &Env, id: Address, batch_ts: u64) {
    let mut batches = get_user_batches(e, id.clone());
    let batch_idx = batches.iter().position(|x| x.unwrap() == batch_ts).unwrap();

    batches.remove(batch_idx as u32);

    let key = DataKey::Batches(id);
    e.storage().set(&key, &batches);
}

pub fn burn_shares(e: &Env, to: Address, shares: i128, batch_ts: u64) {
    let tot_supply = get_tot_supply(e);
    let key = DataKey::Batch(BatchKey(to.clone(), batch_ts));

    let mut batch: BatchObj = e.storage().get(&key).unwrap().unwrap();
    batch.curr_s -= shares;
    put_tot_supply(e, tot_supply - shares);

    if batch.curr_s == 0 {
        e.storage().remove(&key); // if there are 0 shares remove the batch
        remove_user_batch(e, to, batch_ts);
    } else {
        e.storage().set(&key, &batch);
    }
}
