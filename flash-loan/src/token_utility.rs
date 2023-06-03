use soroban_sdk::{token, Address, Env};

use crate::{storage::get_lp, types::Error, vault};

fn compute_fee(amount: &i128) -> i128 {
    amount / 2000 // 0.05%, still TBD
}

pub fn transfer(e: &Env, client: &token::Client, to: &Address, amount: &i128) {
    client.transfer(&e.current_contract_address(), to, amount);
}

pub fn xfer_from_to_fl(
    e: &Env,
    client: &token::Client,
    from: &Address,
    amount: &i128,
) -> Result<(), Error> {
    // catch the result of the `xfer_from` operation
    let res = client.try_transfer_from(
        &e.current_contract_address(),
        from,
        &e.current_contract_address(),
        amount,
    );

    // if the xfer failed, then the receiver contract hasn't paid back the debt + fees
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

    // xfer back the lent capital + fees from the receiver contract to the flash loan
    xfer_from_to_fl(e, client, receiver_id, &(amount + fees))?;

    let lp = get_lp(e);
    transfer(e, client, &lp, &fees);

    // deposit fees into the vault
    //    let vault_contract_id = lp.contract_id().unwrap(); // safe since we require lp to be a contract upon initialization
    vault::Client::new(e, &lp).deposit_fees(&fees);

    Ok(())
}
