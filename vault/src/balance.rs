use core::ops::AddAssign;

use soroban_sdk::{Address, Env};

use crate::storage::{get_tot_supply, put_tot_supply, read_balance, write_balance};

pub fn mint_shares(e: &Env, to: Address, shares: i128) {
    let tot_supply = get_tot_supply(e);
    put_tot_supply(e, tot_supply + shares);

    let mut balance = read_balance(e, to.clone());
    balance.add_assign(shares);
    write_balance(e, to, balance);
}

// needs to be rewritten
pub fn burn_shares(e: &Env, to: Address, shares: i128, batch_n: i128) {
    let tot_supply = get_tot_supply(e);
    /*    let key = DataKey::Batch(BatchKey(to, batch_n));

    let mut batch: BatchObj = e.storage().get(&key).unwrap().unwrap();
    batch.curr_s -= shares;
    put_tot_supply(e, tot_supply - shares);

    if batch.curr_s == 0 {
        e.storage().remove(&key); // if there are 0 shares remove the batch
    } else {
        e.storage().set(&key, &batch);
    }*/
}
