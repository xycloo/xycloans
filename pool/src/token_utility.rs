use crate::{
    rewards::update_fee_per_share_universal, storage::get_token_id,
    types::Error,
};
use soroban_sdk::{token, Address, Env};

pub(crate) fn transfer(e: &Env, client: &token::Client, to: &Address, amount: &i128) {
    client.transfer(&e.current_contract_address(), to, amount);
}

pub(crate) fn get_token_client(e: &Env) -> token::Client {
    token::Client::new(
        e,
        &get_token_id(e).unwrap(), // safe
                                   // only called when
                                   // execution already
                                   // knows that the contract
                                   // is initialized
    )
}

pub(crate) fn transfer_in_pool(env: &Env, client: &token::Client, from: &Address, amount: &i128) {
    client.transfer(from, &env.current_contract_address(), amount);
}

pub(crate) fn transfer_from_to_pool(
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
pub(crate) fn try_repay(
    e: &Env,
    client: &token::Client,
    receiver_id: &Address,
    amount: i128,
    fee: i128,
) -> Result<(), Error> {
    // xfer back the lent capital + fees from the receiver contract to the flash loan
    transfer_from_to_pool(e, client, receiver_id, &(amount + fee))?;

    // loan is now repaid with interest.
    // we need to update the fee_per_share_universal
    // parameter since we inputted more money in the pool.
    update_fee_per_share_universal(&e, fee);

    Ok(())
}
