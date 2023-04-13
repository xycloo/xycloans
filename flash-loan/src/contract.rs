use soroban_sdk::{contractimpl, Address, BytesN, Env};

use crate::{
    execution::invoke_receiver,
    storage::{get_lp, get_token_id, is_initialized, set_lp, set_token},
    token,
    token_utility::{transfer, try_repay},
    types::Error,
};

pub struct FlashLoanCommon;
pub struct FlashLoanBorrow;
pub struct FlashLoanLender;

pub trait Common {
    /// Initializes the flash loan
    /// @param token_id token of the flash loan
    /// @param lp liquidity provider for the loan. In the Xycloans protocol the lp will always be the associated vault
    fn init(e: Env, token_id: BytesN<32>, lp: Address) -> Result<(), Error>;
}

pub trait Borrow {
    /// Initialize borrow to the `receiver_id` contract
    /// @param receiver_id Address of the receiver contract
    /// @param amount Amount of the flash loans's token to borrow
    fn borrow(e: Env, receiver_id: Address, amount: i128) -> Result<(), Error>;
}

pub trait Lender {
    /// Withdraws an amount of liquidity to an address
    /// @param lender Address of the lender
    /// @param amount Amount to withdraw
    /// @param to Recipient of the liquidity
    fn withdraw(e: Env, lender: Address, amount: i128, to: Address) -> Result<(), Error>;
}

#[contractimpl]
impl Common for FlashLoanCommon {
    fn init(e: Env, token_id: BytesN<32>, lp: Address) -> Result<(), Error> {
        // the flash loan can't be re-initialized
        if is_initialized(&e) {
            return Err(Error::AlreadyInitialized);
        }

        // we require the liquidity provider to be a contract as the flash loan with invoke it to deposit the fees
        if lp.contract_id().is_none() {
            return Err(Error::LPNotAContract);
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

        // assert that the receiver_id is a contract not an account
        if let Some(receiver_id_bytes) = receiver_id.contract_id() {
            // load the flash loan's token and build the client
            let token_id: BytesN<32> = get_token_id(&e);
            let client = token::Client::new(&e, &token_id);

            // transfer `amount` to `receiver_id`
            transfer(&e, &client, &receiver_id, &amount);

            // invoke the `exec_op()` function of the receiver contract
            invoke_receiver(&e, &receiver_id_bytes);

            // try `transfer_from()` of (`amount` + fees) from the receiver to the flash loan
            try_repay(&e, &client, &receiver_id, &amount)?;

            Ok(())
        } else {
            Err(Error::GenericLend)
        }
    }
}

#[contractimpl]
impl Lender for FlashLoanLender {
    fn withdraw(e: Env, lender: Address, amount: i128, to: Address) -> Result<(), Error> {
        // the contract needs to be initialized
        if !is_initialized(&e) {
            return Err(Error::NotInitialized);
        }

        // we assert that it's the flash loan admin calling and authorizing `withdraw`
        if lender != get_lp(&e) {
            return Err(Error::NotLP);
        }
        lender.require_auth();

        // load the flash loan's token and build the client
        let token_id: BytesN<32> = get_token_id(&e);
        let client = token::Client::new(&e, &token_id);

        // transfer the requested amount to `to`
        transfer(&e, &client, &to, &amount);

        Ok(())
    }
}
