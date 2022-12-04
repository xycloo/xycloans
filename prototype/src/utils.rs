use soroban_auth::{Identifier, Signature};
use soroban_sdk::{bigint, symbol, BigInt, BytesN, Env, IntoVal, RawVal};

use crate::{
    token,
    types::{DataKey, Error},
};

pub fn is_initialized(e: &Env) -> bool {
    e.data().has(DataKey::TokenId)
}

// data helpers
pub fn set_token(e: &Env, id: BytesN<32>) {
    e.data().set(DataKey::TokenId, id);
}

pub fn get_vault(e: &Env) -> BytesN<32> {
    e.data().get(DataKey::VaultId).unwrap().unwrap()
}

pub fn set_vault(e: &Env, id: BytesN<32>) {
    e.data().set(DataKey::VaultId, id);
}

fn compute_fee(e: &Env, amount: &BigInt) -> BigInt {
    bigint!(e, 5) * amount / 10000 // 0.05%, still TBD
}

pub fn get_token_id(e: &Env) -> BytesN<32> {
    // safe since we only reach this unwrap when the contract is initialized
    e.data().get(DataKey::TokenId).unwrap().unwrap()
}

pub fn _get_token_balance(e: &Env, id: &Identifier) -> BigInt {
    let token_id: BytesN<32> = get_token_id(e);
    let client = token::Client::new(e, token_id);

    client.balance(id)
}

pub fn vault_xfer(e: &Env, to: &Identifier, amount: &BigInt) -> Result<(), Error> {
    let token_id: BytesN<32> = get_token_id(e);
    let client = token::Client::new(e, token_id);

    let xfer_result = client.try_xfer(&Signature::Invoker, &BigInt::zero(e), to, amount);

    // TODO: more explicit handling
    match xfer_result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(_)) => Err(Error::GenericLend),
        Err(_) => Err(Error::GenericLend),
    }
}

pub fn try_repay(e: &Env, receiver_id: &Identifier, amount: &BigInt) -> Result<(), Error> {
    let fees = compute_fee(e, amount);

    xfer_in_pool(e, receiver_id, &(amount + &fees))?;
    vault_xfer(e, &Identifier::Contract(get_vault(e)), &fees)?;

    Ok(())
}

pub fn xfer_in_pool(e: &Env, from: &Identifier, amount: &BigInt) -> Result<(), Error> {
    let token_id: BytesN<32> = get_token_id(e);
    let client = token::Client::new(e, token_id);

    let xfer_from_result = client.try_xfer_from(
        &Signature::Invoker,
        &BigInt::zero(e),
        from,
        &Identifier::Contract(e.current_contract()),
        amount,
    );

    // TODO: more explicit handling
    match xfer_from_result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(_)) => Err(Error::GenericRepay),
        Err(_) => Err(Error::GenericRepay),
    }
}

pub fn invoke_receiver(e: &Env, id: &BytesN<32>) {
    e.invoke_contract::<RawVal>(id, &symbol!("exec_op"), ().into_val(e));
}

pub fn get_nonce(e: &Env, id: Identifier) -> BigInt {
    let key = DataKey::Nonce(id);
    e.data()
        .get(key)
        .unwrap_or_else(|| Ok(BigInt::zero(&e)))
        .unwrap()
}

pub fn get_contract_id(e: &Env) -> Identifier {
    Identifier::Contract(e.get_current_contract())
}

pub fn get_deposit(e: &Env, id: Identifier) -> BigInt {
    let key = DataKey::Deposit(id);
    e.data()
        .get(key)
        .unwrap_or_else(|| Ok(BigInt::zero(&e)))
        .unwrap()
}

pub fn set_deposit(e: &Env, id: Identifier, amount: BigInt) {
    let current_deposit = get_deposit(e, id.clone());
    let key = DataKey::Deposit(id);

    e.data().set(key, current_deposit + amount);
}

pub fn remove_deposit(e: &Env, id: Identifier) {
    let key = DataKey::Deposit(id);
    e.data().remove(key);
}
