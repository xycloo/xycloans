use soroban_sdk::{Address, Env};

use crate::types::DataKey;

pub(crate) fn put_tot_supply(e: &Env, supply: i128) {
    let key = DataKey::TotSupply;
    e.storage().set(&key, &supply);
}

pub(crate) fn get_tot_supply(e: &Env) -> i128 {
    let key = DataKey::TotSupply;
    e.storage().get(&key).unwrap_or(Ok(0)).unwrap()
}

pub(crate) fn write_total_deposited(e: &Env, amount: i128) {
    let key = DataKey::TotalDeposited;
    e.storage().set(&key, &amount);
}

pub(crate) fn read_total_deposited(e: &Env) -> i128 {
    let key = DataKey::TotalDeposited;
    e.storage().get(&key).unwrap_or(Ok(0)).unwrap()
}

pub(crate) fn write_balance(e: &Env, addr: Address, balance: i128) {
    let key = DataKey::Balance(addr);
    e.storage().set(&key, &balance);
}

pub(crate) fn read_balance(e: &Env, addr: Address) -> i128 {
    let key = DataKey::Balance(addr);
    e.storage().get(&key).unwrap_or(Ok(0)).unwrap()
}

pub(crate) fn remove_balance(e: &Env, addr: Address) {
    let key = DataKey::Balance(addr);
    e.storage().remove(&key)
}

pub(crate) fn write_fee_per_share_particular(e: &Env, addr: Address, amount: i128) {
    let key = DataKey::FeePerShareParticular(addr);
    e.storage().set(&key, &amount);
}

pub(crate) fn read_fee_per_share_particular(e: &Env, addr: Address) -> i128 {
    let key = DataKey::FeePerShareParticular(addr);
    e.storage().get(&key).unwrap_or(Ok(0)).unwrap()
}

pub(crate) fn remove_fee_per_share_particular(e: &Env, addr: Address) {
    let key = DataKey::FeePerShareParticular(addr);
    e.storage().remove(&key)
}

pub(crate) fn write_matured_fees_particular(e: &Env, addr: Address, amount: i128) {
    let key = DataKey::MaturedFeesParticular(addr);
    e.storage().set(&key, &amount);
}

pub(crate) fn read_matured_fees_particular(e: &Env, addr: Address) -> i128 {
    let key = DataKey::MaturedFeesParticular(addr);
    e.storage().get(&key).unwrap_or(Ok(0)).unwrap()
}

pub(crate) fn remove_matured_fees_particular(e: &Env, addr: Address) {
    let key = DataKey::MaturedFeesParticular(addr);
    e.storage().remove(&key)
}

pub(crate) fn put_fee_per_share_universal(e: &Env, last_recorded: i128) {
    let key = DataKey::FeePerShareUniversal;
    e.storage().set(&key, &last_recorded);
}

pub(crate) fn get_fee_per_share_universal(e: &Env) -> i128 {
    let key = DataKey::FeePerShareUniversal;
    e.storage().get(&key).unwrap_or(Ok(0)).unwrap()
}

pub(crate) fn put_token_id(e: &Env, token_id: Address) {
    let key = DataKey::TokenId;
    e.storage().set(&key, &token_id);
}

pub(crate) fn get_token_id(e: &Env) -> Address {
    let key = DataKey::TokenId;
    e.storage().get(&key).unwrap().unwrap()
}

pub(crate) fn put_flash_loan(e: &Env, id: Address) {
    let key = DataKey::FlashLoan;
    e.storage().set(&key, &id);
}

pub(crate) fn get_flash_loan(e: &Env) -> Address {
    let key = DataKey::FlashLoan;
    e.storage().get(&key).unwrap().unwrap()
}

pub(crate) fn has_administrator(e: &Env) -> bool {
    let key = DataKey::Admin;
    e.storage().has(&key)
}

pub(crate) fn read_admin(e: &Env) -> Address {
    let key = DataKey::Admin;
    e.storage().get_unchecked(&key).unwrap()
}

pub(crate) fn write_administrator(e: &Env, id: Address) {
    let key = DataKey::Admin;
    e.storage().set(&key, &id);
}

pub(crate) fn auth_admin(e: &Env) {
    read_admin(e).require_auth();
}
