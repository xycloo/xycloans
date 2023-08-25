use crate::{
    balance::{burn_shares, mint_shares},
    events,
    execution::invoke_receiver,
    rewards::{pay_matured, update_rewards},
    storage::*,
    token_utility::{get_token_client, transfer, transfer_in_pool, try_repay},
    types::Error,
};
use soroban_sdk::{contract, contractimpl, token, Address, Env};

#[contract]
pub struct Pool;

pub trait FlashLoan {
    /// The entry point for executing a flash loan, the initiator (or borrower) provides:
    /// `receiver_id: Address` The address of the receiver contract which contains the borrowing logic.
    /// `amount` Amount of `token_id` to borrow (`token_id` is set when the contract is initialized).
    fn borrow(e: Env, receiver_id: Address, amount: i128) -> Result<(), Error>;
}

pub trait Vault {
    /// deposit

    /// Allows to deposit into the pool and mints liquidity provider shares to the lender.
    /// This action currently must be authorized by the `admin`, so the proxy contract.
    /// This allows a pool to be only funded when the pool is part of the wider protocol, and is not an old pool.
    /// This design decision may be removed in the next release, follow https://github.com/xycloo/xycloans/issues/16

    /// `deposit()` must be provided with:
    /// `from: Address` Address of the liquidity provider.
    /// `amount: i128` Amount of `token_id` that `from` wants to deposit in the pool.
    fn deposit(env: Env, from: Address, amount: i128) -> Result<(), Error>;

    /// update_fee_rewards

    /// Updates the matured rewards for a certain user `addr`
    /// This function may be called by anyone.

    /// `update_fee_rewards()` must be provided with:
    /// `addr: Address` The address that is udpating its fee rewards.
    fn update_fee_rewards(e: Env, addr: Address) -> Result<(), Error>;

    /// withdraw_matured

    /// Allows a certain user `addr` to withdraw the matured fees.
    /// Before calling `withdraw_matured()` the user should call `update_fee_rewards`.
    /// If not, the matured fees that were not updated will not be lost, just not included in the payment.

    /// `withdraw_matured()` must be provided with:
    /// `addr: Address` The address that is withdrawing its fee rewards.
    fn withdraw_matured(e: Env, addr: Address) -> Result<(), Error>;

    /// withdraw

    /// Allows to withdraw liquidity from the pool by burning liquidity provider shares.
    /// Will result in a cross contract call to the flash loan, which holds the funds that are being withdrawn.
    /// The liquidity provider can also withdraw only a portion of its shares.

    /// withdraw() must be provided with:
    /// `addr: Address` Address of the liquidity provider
    /// `amount: i28` Amount of shares that are being withdrawn
    fn withdraw(env: Env, addr: Address, amount: i128) -> Result<(), Error>;

    /// Returns the amount of shares that an address holds.
    fn shares(e: Env, addr: Address) -> i128;

    /// Returns the amount of matured fees for an address.
    fn matured(env: Env, addr: Address) -> i128;
}

pub trait Initializable {
    /// initialize

    /// Constructor function, only to be callable once

    /// `initialize()` must be provided with:
    /// `admin: Address` The vault's admin, effictively the pool's admin as the vault is the flash loan's admin. The admin is currently never used in release 0.2.0, but we are keeping it awaiting to see how the overall ecosystem revolves around governance.
    /// `token_id: Address` The pool's token.
    /// `flash_loan` The address of the associated flash loan contract. `flash_loan` should have `current_contract_address()` as `lp`.
    fn initialize(env: Env, admin: Address, token: Address) -> Result<(), Error>;
}

#[contractimpl]
impl Initializable for Pool {
    fn initialize(
        env: Env,
        admin: Address, // TODO: decide if this needs to be removed
        token: Address,
    ) -> Result<(), Error> {
        if has_token_id(&env) {
            return Err(Error::AlreadyInitialized);
        }

        write_administrator(&env, admin.clone()); // TODO: remove if admin deleted from v2

        put_token_id(&env, token);
        Ok(())
    }
}

#[contractimpl]
impl Vault for Pool {
    fn deposit(env: Env, from: Address, amount: i128) -> Result<(), Error> {
        from.require_auth();

        // we update the rewards before the deposit to avoid the abuse of the collected fees by withdrawing them with liquidity that didn't contribute to their generation.
        update_rewards(&env, from.clone());

        // transfer the funds into the flash loan
        let token_client = get_token_client(&env);
        transfer_in_pool(&env, &token_client, &from, &amount);
        //transfer_into_flash_loan(&e, &token_client, &from, &amount);

        // mint the new shares to the lender.
        // shares to mint will always be the amount deposited, see https://github.com/xycloo/xycloans/issues/17
        mint_shares(&env, from.clone(), amount);

        events::deposited(&env, from, amount);
        Ok(())
    }

    fn withdraw_matured(e: Env, addr: Address) -> Result<(), Error> {
        // require lender auth for withdrawal
        addr.require_auth();

        // pay the matured yield
        pay_matured(&e, addr.clone())?;

        events::matured_withdrawn(&e, addr);
        Ok(())
    }

    fn update_fee_rewards(e: Env, addr: Address) -> Result<(), Error> {
        update_rewards(&e, addr.clone());

        events::matured_updated(&e, addr);
        Ok(())
    }

    fn withdraw(env: Env, addr: Address, amount: i128) -> Result<(), Error> {
        // require lender auth for withdrawal
        addr.require_auth();

        let addr_balance = read_balance(&env, addr.clone());

        // if the desired burned shares are more than the lender's balance return an error
        // if the amount is 0 return an error
        if addr_balance < amount || amount == 0 {
            return Err(Error::InvalidShareBalance);
        }

        // update addr's rewards
        update_rewards(&env, addr.clone());

        // pay out the corresponding deposit
        let token_client = get_token_client(&env);
        transfer(&env, &token_client, &addr, &amount);

        // burn the shares
        burn_shares(&env, addr.clone(), amount);

        events::withdrawn(&env, addr, amount);
        Ok(())
    }

    fn shares(e: Env, addr: Address) -> i128 {
        read_balance(&e, addr)
    }

    fn matured(env: Env, addr: Address) -> i128 {
        read_matured_fees_particular(&env, addr)
    }
}

#[contractimpl]
impl FlashLoan for Pool {
    fn borrow(e: Env, receiver_id: Address, amount: i128) -> Result<(), Error> {
        // load the flash loan's token and build the client.
        // get_token_id() checks that the pool is initialized.
        let token_id: Address = get_token_id(&e)?;
        let client = token::Client::new(&e, &token_id);

        // transfer `amount` to `receiver_id`
        transfer(&e, &client, &receiver_id, &amount);

        // invoke the `exec_op()` function of the receiver contract
        invoke_receiver(&e, &receiver_id);

        // try `transfer_from()` of (`amount` + fees) from the receiver to the flash loan
        try_repay(&e, &client, &receiver_id, amount)?;

        events::loan_successful(&e, receiver_id, amount);
        Ok(())
    }
}
