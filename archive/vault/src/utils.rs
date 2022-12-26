use soroban_auth::{Identifier, Signature};
use soroban_sdk::{
    bigint, panic_with_error, symbol, Address, BigInt, BytesN, Env, IntoVal, RawVal,
};

use crate::{
    token,
    types::{DataKey, VaultError},
};

pub fn get_contract_id(e: &Env) -> Identifier {
    Identifier::Contract(e.get_current_contract())
}

pub fn put_max_supply(e: &Env, supply: BigInt) {
    let key = DataKey::MaxSupply;
    e.data().set(key, supply);
}

pub fn get_max_supply(e: &Env) -> BigInt {
    let key = DataKey::MaxSupply;
    e.data()
        .get(key)
        .unwrap_or_else(|| Ok(BigInt::zero(e)))
        .unwrap()
}

pub fn put_tot_supply(e: &Env, supply: BigInt) {
    let key = DataKey::TotSupply;
    e.data().set(key, supply);
}

pub fn get_tot_supply(e: &Env) -> BigInt {
    let key = DataKey::TotSupply;
    e.data()
        .get(key)
        .unwrap_or_else(|| Ok(BigInt::zero(e)))
        .unwrap()
}

pub fn put_id_balance(e: &Env, id: Identifier, amount: BigInt) {
    let key = DataKey::Balance(id);
    e.data().set(key, amount);
}

pub fn get_id_balance(e: &Env, id: Identifier) -> BigInt {
    let key = DataKey::Balance(id);
    e.data()
        .get(key)
        .unwrap_or_else(|| Ok(BigInt::zero(e)))
        .unwrap()
}

pub fn put_token_id(e: &Env, token_id: BytesN<32>) {
    let key = DataKey::TokenId;
    e.data().set(key, token_id);
}

pub fn get_token_id(e: &Env) -> BytesN<32> {
    let key = DataKey::TokenId;
    e.data().get(key).unwrap().unwrap()
}

pub fn get_token_balance(e: &Env) -> BigInt {
    let contract_id = get_token_id(e);
    token::Client::new(e, contract_id).balance(&get_contract_id(e))
}

pub fn transfer(e: &Env, to: Identifier, amount: BigInt) {
    let client = token::Client::new(e, get_token_id(e));
    client.xfer(
        &Signature::Invoker,
        &client.nonce(&Signature::Invoker.identifier(e)),
        &to,
        &amount,
    );
}

pub fn transfer_in_vault(e: &Env, from: Identifier, amount: BigInt) {
    let client = token::Client::new(e, get_token_id(e));
    let vault_id = get_contract_id(e);

    client.xfer_from(
        &Signature::Invoker,
        &BigInt::zero(e),
        &from,
        &vault_id,
        &amount,
    )
}

pub fn has_administrator(e: &Env) -> bool {
    let key = DataKey::Admin;
    e.data().has(key)
}

pub fn read_administrator(e: &Env) -> Identifier {
    let key = DataKey::Admin;
    e.data().get_unchecked(key).unwrap()
}

pub fn check_administrator(e: &Env) -> bool {
    read_administrator(e) == identifier(e.invoker())
}

pub fn identifier(addr: Address) -> Identifier {
    match addr {
        Address::Account(id) => Identifier::Account(id),
        Address::Contract(id) => Identifier::Contract(id),
    }
}

pub fn write_administrator(e: &Env, id: Identifier) {
    let key = DataKey::Admin;
    e.data().set(key, id);
}

pub fn read_nonce(e: &Env, id: &Identifier) -> BigInt {
    let key = DataKey::Nonce(id.clone());
    e.data()
        .get(key)
        .unwrap_or_else(|| Ok(BigInt::zero(e)))
        .unwrap()
}

pub fn mint_shares(e: &Env, to: Identifier, shares: BigInt) {
    let tot_supply = get_tot_supply(e);
    let id_balance = get_id_balance(e, to.clone());

    put_tot_supply(e, tot_supply + shares.clone());
    put_id_balance(e, to, id_balance + shares);
}

pub fn burn_shares(e: &Env, to: Identifier, shares: BigInt) {
    let tot_supply = get_tot_supply(e);
    let id_balance = get_id_balance(e, to.clone());

    if shares > id_balance {
        panic!("not enough vault shares")
    }

    put_tot_supply(e, tot_supply - shares.clone());
    put_id_balance(e, to, id_balance - shares);
}

pub fn assert_supply(e: &Env, shares: &BigInt) -> bool {
    let max_supply = get_max_supply(e);
    let tot_supply = get_tot_supply(e);

    max_supply >= tot_supply + shares
}
