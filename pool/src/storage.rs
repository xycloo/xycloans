use soroban_sdk::{Address, Env};

use crate::{
    types::{DataKey, Error},
    INSTANCE_LEDGER_LIFE, PERSISTENT_LEDGER_LIFE,
};

// User specific state.

pub(crate) fn write_balance(e: &Env, addr: Address, balance: i128) {
    let key = DataKey::Balance(addr);
    e.storage().persistent().set(&key, &balance);
    e.storage().persistent().bump(&key, PERSISTENT_LEDGER_LIFE, PERSISTENT_LEDGER_LIFE);
}

pub(crate) fn read_balance(e: &Env, addr: Address) -> i128 {
    let key = DataKey::Balance(addr);
    
    if let Some(balance) = e.storage().persistent().get(&key) {
        e.storage().persistent().bump(&key, PERSISTENT_LEDGER_LIFE, PERSISTENT_LEDGER_LIFE);
        balance
    } else {
        0
    }
}

pub(crate) fn write_fee_per_share_particular(e: &Env, addr: Address, amount: i128) {
    let key = DataKey::FeePerShareParticular(addr);
    e.storage().persistent().set(&key, &amount);
    e.storage().persistent().bump(&key, PERSISTENT_LEDGER_LIFE, PERSISTENT_LEDGER_LIFE);
}

pub(crate) fn read_fee_per_share_particular(e: &Env, addr: Address) -> i128 {
    let key = DataKey::FeePerShareParticular(addr);
    
    if let Some(particular) = e.storage().persistent().get(&key) {
        e.storage().persistent().bump(&key, PERSISTENT_LEDGER_LIFE, PERSISTENT_LEDGER_LIFE);
        particular
    } else {
        0
    }
}

pub(crate) fn write_matured_fees_particular(e: &Env, addr: Address, amount: i128) {
    let key = DataKey::MaturedFeesParticular(addr);
    e.storage().persistent().set(&key, &amount);
    e.storage().persistent().bump(&key, PERSISTENT_LEDGER_LIFE, PERSISTENT_LEDGER_LIFE);
}

pub(crate) fn read_matured_fees_particular(e: &Env, addr: Address) -> i128 {
    let key = DataKey::MaturedFeesParticular(addr);

    if let Some(matured) = e.storage().persistent().get(&key) {
        e.storage().persistent().bump(&key, PERSISTENT_LEDGER_LIFE, PERSISTENT_LEDGER_LIFE);
        matured    
    } else {
        0
    }
}

// INSTANCE

// instance bumps are for every call on the contract and better controlled directly
// within the exported function's block.
pub(crate) fn bump_instance(env: &Env) {
    env.storage().instance().bump(INSTANCE_LEDGER_LIFE, PERSISTENT_LEDGER_LIFE);
}

pub(crate) fn put_tot_supply(e: &Env, supply: i128) {
    let key = DataKey::TotSupply;
    e.storage().instance().set(&key, &supply);
}

pub(crate) fn get_tot_supply(e: &Env) -> i128 {
    let key = DataKey::TotSupply;
    e.storage().instance().get(&key).unwrap_or(0)
}

pub(crate) fn put_fee_per_share_universal(e: &Env, last_recorded: i128) {
    let key = DataKey::FeePerShareUniversal;
    e.storage().instance().set(&key, &last_recorded);
}

pub(crate) fn get_fee_per_share_universal(e: &Env) -> i128 {
    let key = DataKey::FeePerShareUniversal;
    e.storage().instance().get(&key).unwrap_or(0)
}

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
        return Err(Error::NotInitialized);
    }
}

// shouldn't be needed because of state expiration

pub(crate) fn _remove_matured_fees_particular(e: &Env, addr: Address) {
    let key = DataKey::MaturedFeesParticular(addr);
    e.storage().persistent().remove(&key)
}

pub(crate) fn _remove_fee_per_share_particular(e: &Env, addr: Address) {
    let key = DataKey::FeePerShareParticular(addr);
    e.storage().persistent().remove(&key)
}

pub(crate) fn _remove_balance(e: &Env, addr: Address) {
    let key = DataKey::Balance(addr);
    e.storage().persistent().remove(&key)
}
