use core::ops::{AddAssign, MulAssign};

use soroban_sdk::{unwrap::UnwrapOptimized, vec, Address, BytesN, ConversionError, Env, Vec};

use crate::{
    token,
    types::{BatchKey, BatchObj, DataKey, Error},
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

pub fn write_balance(e: &Env, addr: Address, balance: i128) {
    let key = DataKey::Balance(addr);
    e.storage().set(&key, &balance);
}

pub fn read_balance(e: &Env, addr: Address) -> i128 {
    let key = DataKey::Balance(addr);
    e.storage().get(&key).unwrap_or(Ok(0)).unwrap()
}

pub fn write_fee_per_share_particular(e: &Env, addr: Address, amount: i128) {
    let key = DataKey::FeePerShareParticular(addr);
    e.storage().set(&key, &amount);
}

pub fn read_fee_per_share_particular(e: &Env, addr: Address) -> i128 {
    let key = DataKey::FeePerShareParticular(addr);
    e.storage().get(&key).unwrap_or(Ok(0)).unwrap()
}

pub fn write_matured_fees_particular(e: &Env, addr: Address, amount: i128) {
    let key = DataKey::MaturedFeesParticular(addr);
    e.storage().set(&key, &amount);
}

pub fn read_matured_fees_particular(e: &Env, addr: Address) -> i128 {
    let key = DataKey::MaturedFeesParticular(addr);
    e.storage().get(&key).unwrap_or(Ok(0)).unwrap()
}

pub fn put_collected_last_recorded(e: &Env, last_recorded: i128) {
    let key = DataKey::CollectedLastRecorded;
    e.storage().set(&key, &last_recorded);
}

pub fn get_collected_last_recorded(e: &Env) -> i128 {
    let key = DataKey::CollectedLastRecorded;
    e.storage().get(&key).unwrap_or(Ok(0)).unwrap()
}

pub fn put_fee_per_share_universal(e: &Env, last_recorded: i128) {
    let key = DataKey::FeePerShareUniversal;
    e.storage().set(&key, &last_recorded);
}

pub fn get_fee_per_share_universal(e: &Env) -> i128 {
    let key = DataKey::FeePerShareUniversal;
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

pub fn get_token_balance(e: &Env, client: &token::Client) -> i128 {
    client.balance(&get_contract_addr(e)) + client.balance(&get_flash_loan(e))
}

pub fn read_flash_loan_balance(e: &Env, client: &token::Client) -> i128 {
    client.balance(&get_flash_loan(e))
}

pub fn transfer(e: &Env, client: &token::Client, to: &Address, amount: i128) {
    client.transfer(&get_contract_addr(e), to, &amount);
}

pub fn _transfer_in_vault(e: &Env, from: &Address, amount: &i128) {
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

pub fn mint_shares(e: &Env, to: Address, shares: i128) {
    let tot_supply = get_tot_supply(e);
    put_tot_supply(e, tot_supply + shares);

    let mut balance = read_balance(e, to.clone());
    balance.add_assign(shares);
    write_balance(e, to, balance);
}

pub fn burn_shares(e: &Env, to: Address, shares: i128, batch_n: i128) {
    let tot_supply = get_tot_supply(e);
    let key = DataKey::Batch(BatchKey(to, batch_n));

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

pub fn auth_admin(e: &Env, admin: Address) -> Result<(), Error> {
    if read_admin(e) != admin {
        return Err(Error::InvalidAdminAuth);
    }
    admin.require_auth();
    Ok(())
}

pub fn get_batch(e: &Env, id: Address, batch_n: i128) -> Option<Result<BatchObj, ConversionError>> {
    let key = DataKey::Batch(BatchKey(id, batch_n));
    e.storage().get(&key)
}

pub fn get_initial_deposit(e: &Env, id: Address) -> i128 {
    e.storage().get(&DataKey::InitialDep(id)).unwrap().unwrap()
}

pub fn set_initial_deposit(e: &Env, id: Address, amount: i128) {
    e.storage().set(&DataKey::InitialDep(id), &amount)
}

pub fn transfer_into_flash_loan(e: &Env, client: &token::Client, from: &Address, amount: &i128) {
    client.transfer(from, &get_flash_loan(e), amount);
}

pub fn get_token_client(e: &Env) -> token::Client {
    token::Client::new(e, &get_token_id(e))
}

// NEW REWARDING SYSTEM
