use soroban_sdk::{contractimpl, Address, BytesN, Env};

use crate::{
    types::{DataKey, Error},
    utils::{
        get_lp, has_lp, invoke_receiver, is_initialized, set_lp, set_token, transfer, try_repay,
    },
};

pub struct FlashLoanCommon;
pub struct FlashLoanBorrow;
pub struct FlashLoanLender;

pub trait Common {
    #[doc = "Initializes the contract. @dev specify: [the token to use, ]"]
    fn init(e: Env, token_id: BytesN<32>, lp: Address) -> Result<(), Error>;
}

pub trait Borrow {
    #[doc = "Borrow money specifyng a receiver, which should abide to the &FlashLoanReceiver standard interface"]
    fn borrow(
        e: Env,
        receiver_id: Address,
        receiver_id_bytes: BytesN<32>,
        amount: i128,
    ) -> Result<(), Error>;
}

pub trait Lender {
    fn withdraw(e: Env, lender: Address, amount: i128, to: Address) -> Result<(), Error>;
}

#[contractimpl]
impl Common for FlashLoanCommon {
    fn init(e: Env, token_id: BytesN<32>, lp: Address) -> Result<(), Error> {
        let token_key = DataKey::TokenId;
        if e.storage().has(&token_key) {
            return Err(Error::ContractAlreadyInitialized);
        }

        set_token(&e, token_id);
        set_lp(&e, lp);
        Ok(())
    }
}

#[contractimpl]
impl Borrow for FlashLoanBorrow {
    fn borrow(
        e: Env,
        receiver_id: Address,
        receiver_id_bytes: BytesN<32>, // we should check for a way to convert the address to BytesN<32> or the other way around without using the test utils
        amount: i128,
    ) -> Result<(), Error> {
        if !is_initialized(&e) {
            return Err(Error::Generic);
        }

        transfer(&e, &receiver_id, &amount)?;
        invoke_receiver(&e, &receiver_id_bytes);
        try_repay(&e, &receiver_id, &amount)?;
        Ok(())
    }
}

#[contractimpl]
impl Lender for FlashLoanLender {
    fn withdraw(e: Env, lender: Address, amount: i128, to: Address) -> Result<(), Error> {
        if !is_initialized(&e) || !has_lp(&e) {
            return Err(Error::Generic);
        }

        lender.require_auth();

        if lender != get_lp(&e) {
            return Err(Error::Generic);
        }

        transfer(&e, &to, &amount)?;

        Ok(())
    }
}
