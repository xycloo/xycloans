use soroban_sdk::{unwrap::UnwrapOptimized, Address, BytesN, Env};

use crate::types::{DataKey, Error};

pub(crate) fn set_admin(env: &Env, admin: Address) {
    env.storage().set(&DataKey::Admin, &admin);
}

pub(crate) fn read_admin(env: &Env) -> Result<Address, Error> {
    if let Some(Ok(admin_id)) = env.storage().get(&DataKey::Admin) {
        Ok(admin_id)
    } else {
        Err(Error::NotInitialized)
    }
}

pub(crate) fn has_admin(env: &Env) -> bool {
    env.storage().has(&DataKey::Admin)
}

pub(crate) fn set_vault(env: &Env, token_address: Address, vault_address: Address) {
    let key = &DataKey::Vault(token_address);
    env.storage().set(key, &vault_address);
}

pub(crate) fn read_vault(env: &Env, token_address: Address) -> Result<Address, Error> {
    let key = &DataKey::Vault(token_address);
    if let Some(Ok(vault_address)) = env.storage().get(key) {
        Ok(vault_address)
    } else {
        Err(Error::VaultDoesntExist)
    }
}

pub(crate) fn set_flash_loan(env: &Env, token_address: Address, flash_loan_address: Address) {
    let key = &DataKey::FlashLoan(token_address);
    env.storage().set(key, &flash_loan_address);
}

pub(crate) fn read_flash_loan(env: &Env, token_address: Address) -> Result<Address, Error> {
    let key = &DataKey::FlashLoan(token_address);
    if let Some(Ok(flash_loan_address)) = env.storage().get(key) {
        Ok(flash_loan_address)
    } else {
        Err(Error::FlashLoanDoesntExist)
    }
}

pub(crate) fn read_flash_loan_hash(env: &Env) -> BytesN<32> {
    env.storage()
        .get(&DataKey::FlashLoanHash)
        .unwrap_optimized() // safe, only called after the admin is read, thus after the factory was initialized
        .unwrap_optimized()
}

pub(crate) fn read_vault_hash(env: &Env) -> BytesN<32> {
    env.storage()
        .get(&DataKey::VaultHash)
        .unwrap_optimized() // safe, only called after the admin is read, thus after the factory was initialized
        .unwrap_optimized()
}

pub(crate) fn write_flash_loan_hash(env: &Env, hash: &BytesN<32>) {
    env.storage().set(&DataKey::FlashLoanHash, hash)
}

pub(crate) fn write_vault_hash(env: &Env, hash: &BytesN<32>) {
    env.storage().set(&DataKey::VaultHash, hash)
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
