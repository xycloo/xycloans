use crate::storage::{get_tot_supply, put_tot_supply, read_balance, write_balance};
use core::ops::SubAssign;
use soroban_sdk::{Address, Env};

pub(crate) fn mint_shares(e: &Env, to: Address, shares: i128) {
    // add to total supply
    put_tot_supply(e, get_tot_supply(e) + shares);

    // add to user balance
    write_balance(e, to.clone(), read_balance(e, to) + shares);
}

// needs to be rewritten
pub(crate) fn burn_shares(e: &Env, to: Address, shares: i128) {
    // update the total supply
    let tot_supply = get_tot_supply(e);
    put_tot_supply(e, tot_supply - shares);

    let mut balance = read_balance(e, to.clone());

    // update addr's balance
    balance.sub_assign(shares);
    write_balance(e, to, balance);
}
