use soroban_auth::{Identifier, Signature};
use soroban_sdk::{bigint, symbol, BigInt, BytesN, Env, IntoVal, RawVal};

use crate::{
    token,
    types::{DataKey, Error},
};

fn compute_fee(e: &Env, amount: &BigInt) -> BigInt {
    bigint!(e, 5) * amount / 10000 // 0.05%, still TBD
}

pub fn get_token_id(e: &Env) -> BytesN<32> {
    // safe since we only reach this unwrap when the contract is initialized
    e.data().get(DataKey::TokenId).unwrap().unwrap()
}

pub fn get_token_balance(e: &Env, id: &Identifier) -> BigInt {
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
    let token_id: BytesN<32> = get_token_id(e);
    let client = token::Client::new(e, token_id);

    let fees = compute_fee(e, amount);
    let total_amount = amount + fees;
    let xfer_from_result = client.try_xfer_from(
        &Signature::Invoker,
        &BigInt::zero(e),
        receiver_id,
        &Identifier::Contract(e.current_contract()),
        &total_amount,
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
