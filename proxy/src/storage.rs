use soroban_sdk::{Address, Env};

use crate::{
    flash_loan,
    types::{DataKey, Error},
    vault,
};

pub(crate) fn set_admin(env: &Env, admin: Address) {
    env.storage().set(&DataKey::Admin, &admin);
}

pub(crate) fn get_admin(env: &Env) -> Result<Address, Error> {
    if let Some(Ok(admin_id)) = env.storage().get(&DataKey::Admin) {
        Ok(admin_id)
    } else {
        Err(Error::NotInitialized)
    }
}

pub(crate) fn has_admin(env: &Env) -> bool {
    env.storage().has(&DataKey::Admin)
}

pub(crate) fn set_vault(env: &Env, token_contract_id: Address, vault_contract_id: Address) {
    let key = &DataKey::Vault(token_contract_id);
    env.storage().set(key, &vault_contract_id);
}

pub(crate) fn get_vault(env: &Env, token_contract_id: Address) -> Result<Address, Error> {
    let key = &DataKey::Vault(token_contract_id);
    if let Some(Ok(vault_contract_id)) = env.storage().get(key) {
        Ok(vault_contract_id)
    } else {
        Err(Error::VaultDoesntExist)
    }
}

pub(crate) fn set_flash_loan(
    env: &Env,
    token_contract_id: Address,
    flash_loan_contract_id: Address,
) {
    let key = &DataKey::FlashLoan(token_contract_id);
    env.storage().set(key, &flash_loan_contract_id);
}

pub(crate) fn get_flash_loan(env: &Env, token_contract_id: Address) -> Result<Address, Error> {
    let key = &DataKey::FlashLoan(token_contract_id);
    if let Some(Ok(flash_loan_contract_id)) = env.storage().get(key) {
        Ok(flash_loan_contract_id)
    } else {
        Err(Error::FlashLoanDoesntExist)
    }
}

pub(crate) fn vault_withdraw_matured_fees(
    env: &Env,
    provider: Address,
    token_contract_id: Address,
) -> Result<(), Error> {
    let vault_client = vault::Client::new(env, &get_vault(env, token_contract_id)?);

    vault_client.withdraw_matured(&provider);

    Ok(())
}

pub(crate) fn flash_loan_borrow(
    env: &Env,
    token_contract_id: Address,
    amount: i128,
    receiver_address: Address,
) -> Result<(), Error> {
    let flash_loan_client = flash_loan::Client::new(env, &get_flash_loan(env, token_contract_id)?);
    flash_loan_client.borrow(&receiver_address, &amount);
    Ok(())
}
