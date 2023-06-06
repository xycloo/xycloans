use soroban_sdk::{contractimpl, token, Address, Env};

use crate::{
    events,
    execution::invoke_receiver,
    storage::{get_lp, get_token_id, is_initialized, set_lp, set_token},
    token_utility::{transfer, try_repay},
    types::Error,
};

pub struct FlashLoanCommon;
pub struct FlashLoanBorrow;
pub struct FlashLoanLender;

pub trait Common {
    /// init

    /// Constructor function, only to be callable once. // this behaviour is currently achieved by reading if the `token_id` already lives in the contract's state.
    /// init() initializes the flash loan contract, setting the token and the liquidity provider.
    ///     - Flash loan contracts only hold and lend one token.
    ///     - The `lp` is the flash loan's admin. `lp` is the only entity that can withdraw funds from the flash loan contract.
    fn init(e: Env, token_id: Address, lp: Address) -> Result<(), Error>;
}

pub trait Borrow {
    /// borrow

    /// The entry point for executing a flash loan, the initiator (or borrower) provides:
    /// `receiver_id: Address` The address of the receiver contract which contains the borrowing logic.
    /// `amount` Amount of `token_id` to borrow (`token_id` is set when the contract is initialized).
    fn borrow(e: Env, receiver_id: Address, amount: i128) -> Result<(), Error>;
}

pub trait Lender {
    /// withdraw

    /// Allows the liquidity provider to withdraw their funds.
    /// Only the `lp` can call this function:
    /// `amount` Amount of `token_id` to withdraw.
    /// `to` Receipient of the withdrawal.
    fn withdraw(e: Env, amount: i128, to: Address) -> Result<(), Error>;
}

#[contractimpl]
impl Common for FlashLoanCommon {
    fn init(e: Env, token_id: Address, lp: Address) -> Result<(), Error> {
        // the flash loan can't be re-initialized
        if is_initialized(&e) {
            return Err(Error::AlreadyInitialized);
        }

        // write to storage
        set_token(&e, token_id);
        set_lp(&e, lp);
        Ok(())
    }
}

#[contractimpl]
impl Borrow for FlashLoanBorrow {
    fn borrow(e: Env, receiver_id: Address, amount: i128) -> Result<(), Error> {
        // the contract needs to be initialized before lending
        if !is_initialized(&e) {
            return Err(Error::NotInitialized);
        }

        // load the flash loan's token and build the client
        let token_id: Address = get_token_id(&e);
        let client = token::Client::new(&e, &token_id);

        // transfer `amount` to `receiver_id`
        transfer(&e, &client, &receiver_id, &amount);

        // invoke the `exec_op()` function of the receiver contract
        invoke_receiver(&e, &receiver_id);

        // try `transfer_from()` of (`amount` + fees) from the receiver to the flash loan
        try_repay(&e, &client, &receiver_id, &amount)?;

        events::loan_successful(&e, receiver_id, amount);
        Ok(())
    }
}

#[contractimpl]
impl Lender for FlashLoanLender {
    fn withdraw(e: Env, amount: i128, to: Address) -> Result<(), Error> {
        // the contract needs to be initialized
        if !is_initialized(&e) {
            return Err(Error::NotInitialized);
        }

        // auth the admin
        get_lp(&e).require_auth();

        // load the flash loan's token and build the client
        let token_id: Address = get_token_id(&e);
        let client = token::Client::new(&e, &token_id);

        // transfer the requested amount to `to`
        transfer(&e, &client, &to, &amount);

        events::withdraw(&e, amount, to);
        Ok(())
    }
}
