use soroban_auth::{Identifier, Signature};
use soroban_sdk::{symbol, BytesN, Env, IntoVal, RawVal};

use crate::{
    token,
    types::{DataKey, Error},
};

pub fn is_initialized(e: &Env) -> bool {
    e.storage().has(DataKey::TokenId)
}

// data helpers
pub fn set_token(e: &Env, id: BytesN<32>) {
    e.storage().set(DataKey::TokenId, id);
}

fn compute_fee(amount: &i128) -> i128 {
    5 * amount / 10000 // 0.05%, still TBD
}

pub fn get_token_id(e: &Env) -> BytesN<32> {
    // safe since we only reach this unwrap when the contract is initialized
    e.storage().get(DataKey::TokenId).unwrap().unwrap()
}

pub fn get_token_balance(e: &Env) -> i128 {
    let token_id: BytesN<32> = get_token_id(e);
    let client = token::Client::new(e, token_id);

    client.balance(&get_contract_id(e))
}

pub fn vault_xfer(e: &Env, to: &Identifier, amount: &i128) -> Result<(), Error> {
    let token_id: BytesN<32> = get_token_id(e);
    let client = token::Client::new(e, token_id);

    let xfer_result = client.try_xfer(&Signature::Invoker, &0, to, amount);

    // TODO: more explicit handling
    match xfer_result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(_)) => Err(Error::GenericLend),
        Err(_) => Err(Error::GenericLend),
    }
}

pub fn try_repay(e: &Env, receiver_id: &Identifier, amount: &i128) -> Result<(), Error> {
    let fees = compute_fee(amount);

    xfer_in_pool(e, receiver_id, &(amount + fees))?;
    vault_xfer(e, &get_lp(e), &fees)?;

    Ok(())
}

pub fn xfer_in_pool(e: &Env, from: &Identifier, amount: &i128) -> Result<(), Error> {
    let token_id: BytesN<32> = get_token_id(e);
    let client = token::Client::new(e, token_id);

    let xfer_from_result = client.try_xfer_from(
        &Signature::Invoker,
        &0,
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

pub fn get_nonce(e: &Env, id: Identifier) -> i128 {
    let key = DataKey::Nonce(id);
    e.storage().get(key).unwrap_or(Ok(0)).unwrap()
}

pub fn get_contract_id(e: &Env) -> Identifier {
    Identifier::Contract(e.get_current_contract())
}

pub fn has_lp(e: &Env) -> bool {
    e.storage().has(DataKey::LP)
}

pub fn set_lp(e: &Env, id: Identifier) {
    e.storage().set(DataKey::LP, id);
}

pub fn get_lp(e: &Env) -> Identifier {
    e.storage().get(DataKey::LP).unwrap().unwrap()
}

pub fn remove_lp(e: &Env) {
    e.storage().remove(DataKey::LP)
}
