use soroban_auth::{verify, Identifier, Signature};
use soroban_sdk::{contractimpl, symbol, BytesN, Env};

use crate::{
    types::{DataKey, Error},
    utils::{
        get_contract_id, get_lp, get_nonce, has_lp, invoke_receiver, is_initialized, set_lp,
        set_token, transfer, try_repay,
    },
};

pub struct FlashLoanCommon;
pub struct FlashLoanBorrow;
pub struct FlashLoanLender;

pub trait Common {
    #[doc = "Initializes the contract. @dev specify: [the token to use, ]"]
    fn init(e: Env, token_id: BytesN<32>, lp: Identifier) -> Result<(), Error>;
}

pub trait Borrow {
    #[doc = "Borrow money specifyng a receiver, which should abide to the &FlashLoanReceiver standard interface"]
    fn borrow(e: Env, receiver_id: Identifier, amount: i128) -> Result<(), Error>;
}

pub trait Lender {
    fn prov_liq(e: Env, sig: Signature, amount: i128) -> Result<(), Error>;
    fn withdraw(e: Env, sig: Signature, amount: i128, to: Identifier) -> Result<(), Error>;
}

#[contractimpl]
impl Common for FlashLoanCommon {
    fn init(e: Env, token_id: BytesN<32>, lp: Identifier) -> Result<(), Error> {
        let token_key = DataKey::TokenId;
        if e.storage().has(token_key) {
            return Err(Error::ContractAlreadyInitialized);
        }

        set_token(&e, token_id);
        set_lp(&e, lp);
        Ok(())
    }
}

#[contractimpl]
impl Borrow for FlashLoanBorrow {
    fn borrow(e: Env, receiver_id: Identifier, amount: i128) -> Result<(), Error> {
        if !is_initialized(&e) {
            return Err(Error::Generic);
        }

        if let Identifier::Contract(receiver_id_bytes) = &receiver_id {
            transfer(&e, &receiver_id, &amount)?;
            invoke_receiver(&e, receiver_id_bytes);
            try_repay(&e, &receiver_id, &amount)?;
            Ok(())
        } else {
            Err(Error::Generic)
        }
    }
}

#[contractimpl]
impl Lender for FlashLoanLender {
    fn prov_liq(e: Env, sig: Signature, amount: i128) -> Result<(), Error> {
        if !is_initialized(&e) || !has_lp(&e) {
            return Err(Error::Generic);
        }

        let lp_id = sig.identifier(&e);

        if lp_id != get_lp(&e) {
            return Err(Error::Generic);
        }

        verify(
            &e,
            &sig,
            symbol!("deposit"),
            (
                &amount,
                get_contract_id(&e),
                get_nonce(&e, sig.identifier(&e)),
            ),
        );

        Ok(())
    }

    fn withdraw(e: Env, sig: Signature, amount: i128, to: Identifier) -> Result<(), Error> {
        if !is_initialized(&e) || !has_lp(&e) {
            return Err(Error::Generic);
        }

        let lp_id = sig.identifier(&e);

        if lp_id != get_lp(&e) {
            return Err(Error::Generic);
        }

        verify(
            &e,
            &sig,
            symbol!("withdraw"),
            (get_contract_id(&e), get_nonce(&e, lp_id.clone())),
        );

        transfer(&e, &to, &amount)?;

        Ok(())
    }
}
