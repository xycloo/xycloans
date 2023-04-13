use soroban_sdk::{Address, Env};

use crate::{storage::get_lp, token, types::Error, vault};

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
    // catch the result of the `transfer_from` operation
    let res = client.try_transfer_from(
        &e.current_contract_address(),
        from,
        &e.current_contract_address(),
        amount,
    );

    // if the transfer failed, then the receiver contract hasn't paid back the debt + fees
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

    // transfer back the lent capital + fees from the receiver contract to the flash loan
    xfer_from_to_fl(e, client, receiver_id, &(amount + fees))?;

    // deposit fees into the vault
    let vault_contract_id = get_lp(e).contract_id().unwrap(); // safe since we require lp to be a contract upon initialization
    vault::Client::new(e, &vault_contract_id).deposit_fees(&e.current_contract_address(), &fees);

    Ok(())
}
