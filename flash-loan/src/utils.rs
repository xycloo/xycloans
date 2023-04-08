use soroban_sdk::{Address, BytesN, Env, IntoVal, RawVal, Symbol};

use crate::{
    token,
    types::{DataKey, Error},
};

pub fn is_initialized(e: &Env) -> bool {
    e.storage().has(&DataKey::TokenId)
}

// data helpers
pub fn set_token(e: &Env, id: BytesN<32>) {
    e.storage().set(&DataKey::TokenId, &id);
}

fn compute_fee(amount: &i128) -> i128 {
    amount / 2000 // 0.05%, still TBD
}

pub fn get_token_id(e: &Env) -> BytesN<32> {
    // safe since we only reach this unwrap when the contract is initialized
    e.storage().get(&DataKey::TokenId).unwrap().unwrap()
}

pub fn _get_token_balance(e: &Env) -> i128 {
    let token_id: BytesN<32> = get_token_id(e);
    let client = token::Client::new(e, &token_id);

    client.balance(&get_contract_addr(e))
}

pub fn transfer(e: &Env, client: &token::Client, to: &Address, amount: &i128) {
    //    let token_id: BytesN<32> = get_token_id(e);
    //    let client = token::Client::new(e, &token_id);

    client.transfer(&get_contract_addr(e), to, amount);
}

pub fn xfer_from_to_fl(
    e: &Env,
    client: &token::Client,
    from: &Address,
    amount: &i128,
) -> Result<(), Error> {
    //    let token_id: BytesN<32> = get_token_id(e);
    //    let client = token::Client::new(e, &token_id);

    let res = client.try_transfer_from(&get_contract_addr(e), from, &get_contract_addr(e), amount);

    if let Ok(Ok(_)) = res {
        Ok(())
    } else {
        Err(Error::LoanNotRepaid)
    }
}

pub fn try_repay(
    e: &Env,
    client: &token::Client,
    receiver_id: &Address,
    amount: &i128,
) -> Result<(), Error> {
    let fees = compute_fee(amount);

    xfer_from_to_fl(e, client, receiver_id, &(amount + fees))?;
    transfer(e, client, &get_lp(e), &fees);

    Ok(())
}

pub fn invoke_receiver(e: &Env, id: &BytesN<32>) {
    e.invoke_contract::<RawVal>(id, &Symbol::short("exec_op"), ().into_val(e));
}

pub fn get_contract_addr(e: &Env) -> Address {
    e.current_contract_address()
}

pub fn _has_lp(e: &Env) -> bool {
    e.storage().has(&DataKey::LP)
}

pub fn set_lp(e: &Env, id: Address) {
    e.storage().set(&DataKey::LP, &id);
}

pub fn get_lp(e: &Env) -> Address {
    e.storage().get(&DataKey::LP).unwrap().unwrap()
}

pub fn _remove_lp(e: &Env) {
    e.storage().remove(&DataKey::LP)
}
