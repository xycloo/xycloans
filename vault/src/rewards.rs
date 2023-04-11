use core::ops::AddAssign;

use crate::{
    math::{compute_fee_earned, compute_fee_per_share},
    storage::*,
    token_utility::{get_token_client, transfer},
    types::Error,
};
use soroban_sdk::{Address, Env};

pub fn update_rewards(e: &Env, addr: Address) {
    // loading storage variables
    let total_supply = get_tot_supply(e);
    //    let collected_last_recorded = get_collected_last_recorded(e);
    let fee_per_share_universal = get_fee_per_share_universal(e);

    // computing the new universal fee per share in light of the collected interest
    //    let adjusted_fee_per_share_universal = compute_fee_per_share(
    //        fee_per_share_universal,
    //        collected_last_recorded,
    //        total_supply,
    //    );
    //    put_fee_per_share_universal(e, adjusted_fee_per_share_universal);

    let lender_fees = compute_fee_earned(
        read_balance(e, addr.clone()),
        fee_per_share_universal,
        read_fee_per_share_particular(e, addr.clone()),
    );
    write_fee_per_share_particular(e, addr.clone(), fee_per_share_universal);
    let mut matured = read_matured_fees_particular(e, addr.clone());
    matured.add_assign(lender_fees);
    write_matured_fees_particular(e, addr, matured);
    //    put_collected_last_recorded(e, 0);
}

pub fn update_fee_per_share_universal(e: &Env, collected: i128) {
    //    let collected_last_recorded = get_collected_last_recorded(e);
    let fee_per_share_universal = get_fee_per_share_universal(e);
    let total_supply = get_tot_supply(e);

    // computing the new universal fee per share in light of the collected interest
    let adjusted_fee_per_share_universal =
        compute_fee_per_share(fee_per_share_universal, collected, total_supply);
    put_fee_per_share_universal(e, adjusted_fee_per_share_universal);
    //    put_collected_last_recorded(e, 0);
}

pub fn pay_matured(e: &Env, addr: Address) {
    let token_client = get_token_client(e);

    // collect all the fees matured by the lender `addr`
    let matured = read_matured_fees_particular(e, addr.clone());

    // transfer the matured yield to `addr` and update the particular matured fees storage slot
    transfer(e, &token_client, &addr, matured);
    write_matured_fees_particular(e, addr, 0);
}