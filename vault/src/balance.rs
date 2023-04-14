use core::ops::{AddAssign, SubAssign};

use soroban_sdk::{Address, Env};

use crate::storage::{
    get_tot_supply, put_tot_supply, read_balance, remove_balance, remove_fee_per_share_particular,
    remove_matured_fees_particular, write_balance,
};

pub fn mint_shares(e: &Env, to: Address, shares: i128) {
    let tot_supply = get_tot_supply(e);
    put_tot_supply(e, tot_supply + shares);

    let mut balance = read_balance(e, to.clone());
    balance.add_assign(shares);
    write_balance(e, to, balance);
}

// needs to be rewritten
pub fn burn_shares(e: &Env, to: Address, shares: i128) {
    // update the total supply
    let tot_supply = get_tot_supply(e);
    put_tot_supply(e, tot_supply - shares);

    let mut balance = read_balance(e, to.clone());

    // if addr is burning all the shares then remove every particular data associated with addr
    if balance == shares {
        remove_balance(e, to.clone());
        remove_fee_per_share_particular(e, to.clone());
        remove_matured_fees_particular(e, to.clone());
    }
    {
        // update addr's balance
        balance.sub_assign(shares);
        write_balance(e, to, balance);
    }

    /*    let key = DataKey::Batch(BatchKey(to, batch_n));

    let mut batch: BatchObj = e.storage().get(&key).unwrap().unwrap();
    batch.curr_s -= shares;


    if batch.curr_s == 0 {
        e.storage().remove(&key); // if there are 0 shares remove the batch
    } else {
        e.storage().set(&key, &batch);
    }*/
}
