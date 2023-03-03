use soroban_sdk::{Address, BytesN, Env};

use crate::{
    flash_loan,
    types::{DataKey, Error},
    vault,
};

pub fn set_admin(env: &Env, admin: Address) {
    env.storage().set(&DataKey::Admin, &admin);
}

pub fn get_admin(env: &Env) -> Result<Address, Error> {
    if let Some(Ok(admin_id)) = env.storage().get(&DataKey::Admin) {
        Ok(admin_id)
    } else {
        Err(Error::NotInitialized)
    }
}

pub fn has_admin(env: &Env) -> bool {
    env.storage().has(&DataKey::Admin)
}

pub fn check_admin(env: &Env, admin: &Address) -> Result<(), Error> {
    if admin != &get_admin(env)? {
        return Err(Error::NotAdmin);
    }

    Ok(())
}

pub fn set_vault(env: &Env, token_contract_id: BytesN<32>, vault_contract_id: BytesN<32>) {
    let key = &DataKey::Vault(token_contract_id);
    env.storage().set(key, &vault_contract_id);
}

pub fn get_vault(env: &Env, token_contract_id: BytesN<32>) -> Result<BytesN<32>, Error> {
    let key = &DataKey::Vault(token_contract_id);
    if let Some(Ok(vault_contract_id)) = env.storage().get(key) {
        Ok(vault_contract_id)
    } else {
        Err(Error::VaultDoesntExist)
    }
}

pub fn set_flash_loan(
    env: &Env,
    token_contract_id: BytesN<32>,
    flash_loan_contract_id: BytesN<32>,
) {
    let key = &DataKey::FlashLoan(token_contract_id);
    env.storage().set(key, &flash_loan_contract_id);
}

pub fn get_flash_loan(env: &Env, token_contract_id: BytesN<32>) -> Result<BytesN<32>, Error> {
    let key = &DataKey::FlashLoan(token_contract_id);
    if let Some(Ok(flash_loan_contract_id)) = env.storage().get(key) {
        Ok(flash_loan_contract_id)
    } else {
        Err(Error::FlashLoanDoesntExist)
    }
}

pub fn vault_deposit(
    env: &Env,
    provider: Address,
    token_contract_id: BytesN<32>,
    amount: i128,
) -> Result<(), Error> {
    let vault_client = vault::Client::new(env, &get_vault(env, token_contract_id)?);
    vault_client.deposit(&provider, &amount);

    Ok(())
}

pub fn vault_withdraw_fees(
    env: &Env,
    provider: Address,
    token_contract_id: BytesN<32>,
    batch_n: i128,
    shares: i128,
) -> Result<(), Error> {
    let vault_client = vault::Client::new(env, &get_vault(env, token_contract_id)?);
    vault_client.fee_withd(&provider, &batch_n, &shares);
    Ok(())
}

pub fn flash_loan_borrow(
    env: &Env,
    token_contract_id: BytesN<32>,
    amount: i128,
    receiver_contract_id: BytesN<32>,
    receiver_address: Address,
) -> Result<(), Error> {
    let flash_loan_client = flash_loan::Client::new(env, &get_flash_loan(env, token_contract_id)?);
    flash_loan_client.borrow(&receiver_address, &receiver_contract_id, &amount);
    Ok(())
}
