use soroban_sdk::{Address, Env, unwrap::UnwrapOptimized};

use crate::types::{DataKey, Error};

pub(crate) fn put_tot_supply(e: &Env, supply: i128) {
    let key = DataKey::TotSupply;
    e.storage().instance().set(&key, &supply);
}

pub(crate) fn get_tot_supply(e: &Env) -> i128 {
    let key = DataKey::TotSupply;
    e.storage().instance().get(&key).unwrap_or(0)
}

/*
Currently deprecated functions

pub(crate) fn write_total_deposited(e: &Env, amount: i128) {
    let key = DataKey::TotalDeposited;
    e.storage().set(&key, &amount);
}

pub(crate) fn read_total_deposited(e: &Env) -> i128 {
    let key = DataKey::TotalDeposited;
    e.storage().get(&key).unwrap_or(Ok(0)).unwrap()
}

 */

/*
Probably deprecated-forever functions.
Since release 0.2.0 xycLoans vaults don't require auth from the proxy anymore

pub(crate) fn auth_admin(e: &Env) {
    read_admin(e).require_auth();
}
*/

pub(crate) fn write_balance(e: &Env, addr: Address, balance: i128) {
    let key = DataKey::Balance(addr);
    e.storage().persistent().set(&key, &balance);
}

pub(crate) fn read_balance(e: &Env, addr: Address) -> i128 {
    let key = DataKey::Balance(addr);
    e.storage().persistent().get(&key).unwrap_or(0)
}

// shouldn't be needed given state expiration
pub(crate) fn remove_balance(e: &Env, addr: Address) {
    let key = DataKey::Balance(addr);
    e.storage().persistent().remove(&key)
}

pub(crate) fn write_fee_per_share_particular(e: &Env, addr: Address, amount: i128) {
    let key = DataKey::FeePerShareParticular(addr);
    e.storage().persistent().set(&key, &amount);
}

pub(crate) fn read_fee_per_share_particular(e: &Env, addr: Address) -> i128 {
    let key = DataKey::FeePerShareParticular(addr);
    e.storage().persistent().get(&key).unwrap_or(0)
}

// shouldn't be needed because of state expiration
pub(crate) fn remove_fee_per_share_particular(e: &Env, addr: Address) {
    let key = DataKey::FeePerShareParticular(addr);
    e.storage().persistent().remove(&key)
}

pub(crate) fn write_matured_fees_particular(e: &Env, addr: Address, amount: i128) {
    let key = DataKey::MaturedFeesParticular(addr);
    e.storage().persistent().set(&key, &amount);
}

pub(crate) fn read_matured_fees_particular(e: &Env, addr: Address) -> i128 {
    let key = DataKey::MaturedFeesParticular(addr);
    e.storage().persistent().get(&key).unwrap_or(0)
}

// shouldn't be needed because of state expiration
pub(crate) fn remove_matured_fees_particular(e: &Env, addr: Address) {
    let key = DataKey::MaturedFeesParticular(addr);
    e.storage().persistent().remove(&key)
}

pub(crate) fn put_fee_per_share_universal(e: &Env, last_recorded: i128) {
    let key = DataKey::FeePerShareUniversal;
    e.storage().instance().set(&key, &last_recorded);
}

pub(crate) fn get_fee_per_share_universal(e: &Env) -> i128 {
    let key = DataKey::FeePerShareUniversal;
    e.storage().instance().get(&key).unwrap_or(0)
}

// INSTANCE

pub(crate) fn has_token_id(e: &Env) -> bool {
    let key = DataKey::TokenId;
    e.storage().instance().has(&key)
}

pub(crate) fn put_token_id(e: &Env, token_id: Address) {
    let key = DataKey::TokenId;
    e.storage().instance().set(&key, &token_id);
}

pub(crate) fn get_token_id(e: &Env) -> Result<Address, Error> {
    let key = DataKey::TokenId;

    if let Some(token) = e.storage().instance().get(&key) {
        Ok(token)
    } else {
        return Err(Error::NotInitialized)
    }
}


// These functions do not currently serve any purpuse in the current implementation.
// They probably will if governance doesn't happen within the contract

pub(crate) fn has_administrator(e: &Env) -> bool {
    let key = DataKey::Admin;
    e.storage().instance().has(&key)
}

pub(crate) fn write_administrator(e: &Env, id: Address) {
    let key = DataKey::Admin;
    e.storage().instance().set(&key, &id);
}

pub(crate) fn read_admin(e: &Env) -> Address {
    let key = DataKey::Admin;
    e.storage().instance().get(&key).unwrap()
}
