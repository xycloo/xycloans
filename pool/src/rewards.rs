use crate::{
    events, math::{compute_fee_earned, compute_fee_per_share, I128WithDust}, storage::*, token_utility::{get_token_client, transfer}, types::Error
};
use core::ops::AddAssign;
use soroban_sdk::{Address, Env};

pub(crate) fn update_rewards(e: &Env, addr: Address) {
    let fee_per_share_universal = get_fee_per_share_universal(e);
    let lender_fees = compute_fee_earned(
        read_balance(e, addr.clone()),
        fee_per_share_universal,
        read_fee_per_share_particular(e, addr.clone()),
    );

    write_fee_per_share_particular(e, addr.clone(), fee_per_share_universal);
    
    let mut matured = read_matured_fees_particular(e, addr.clone());
    matured.add_assign(lender_fees);
    
    write_matured_fees_particular(e, addr.clone(), matured);
    events::new_fees(e, addr, lender_fees);
}

pub(crate) fn update_fee_per_share_universal(e: &Env, collected: i128) {
    let fee_per_share_universal = get_fee_per_share_universal(e);
    let total_supply = get_tot_supply(e);
    
    // computing the new universal fee per share in light of the collected interest
    let (adjusted_fee_per_share_universal, dust): I128WithDust =
        compute_fee_per_share(fee_per_share_universal, collected + read_dust(e), total_supply);

    if dust > 0 {
        write_dust(e, dust);
    }

    put_fee_per_share_universal(e, adjusted_fee_per_share_universal);
}

pub(crate) fn pay_matured(e: &Env, addr: Address) -> Result<i128, Error> {
    let token_client = get_token_client(e);

    // collect all the fees matured by the lender `addr`
    let matured = read_matured_fees_particular(e, addr.clone());

    if matured == 0 {
        return Err(Error::NoFeesMatured);
    }

    // transfer the matured yield to `addr` and update the particular matured fees storage slot
    transfer(e, &token_client, &addr, &matured);
    write_matured_fees_particular(e, addr, 0);

    Ok(matured)
}
