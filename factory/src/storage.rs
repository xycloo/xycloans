use soroban_sdk::{unwrap::UnwrapOptimized, Address, BytesN, Env};

use crate::types::{DataKey, Error};

pub(crate) fn set_admin(env: &Env, admin: Address) {
    env.storage().instance().set(&DataKey::Admin, &admin);
}

pub(crate) fn read_admin(env: &Env) -> Result<Address, Error> {
    if let Some(admin_id) = env.storage().instance().get(&DataKey::Admin) {
        Ok(admin_id)
    } else {
        Err(Error::NotInitialized)
    }
}

pub(crate) fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

pub(crate) fn set_pool(env: &Env, token_address: Address, pool_address: Address) {
    let key = &DataKey::Pool(token_address);
    env.storage().persistent().set(key, &pool_address);
}

pub(crate) fn read_pool(env: &Env, token_address: Address) -> Result<Address, Error> {
    let key = &DataKey::Pool(token_address);
    if let Some(vault_address) = env.storage().persistent().get(key) {
        Ok(vault_address)
    } else {
        Err(Error::VaultDoesntExist)
    }
}

pub(crate) fn read_pool_hash(env: &Env) -> BytesN<32> {
    env.storage()
        .instance()
        .get(&DataKey::PoolHash)
        .unwrap_optimized()
}

pub(crate) fn write_pool_hash(env: &Env, hash: &BytesN<32>) {
    env.storage().instance().set(&DataKey::PoolHash, hash)
}

/*

Deprecated from 0.2.0

pub(crate) fn vault_withdraw_matured_fees(
    env: &Env,
    provider: Address,
    token_address: Address,
) -> Result<(), Error> {
    let vault_client = vault::Client::new(env, &read_vault(env, token_address)?);

    vault_client.withdraw_matured(&provider);

    Ok(())
}

pub(crate) fn flash_loan_borrow(
    env: &Env,
    token_address: Address,
    amount: i128,
    receiver_address: Address,
) -> Result<(), Error> {
    let flash_loan_client = flash_loan::Client::new(env, &read_flash_loan(env, token_address)?);
    flash_loan_client.borrow(&receiver_address, &amount);
    Ok(())
}



*/
