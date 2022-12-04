use soroban_auth::{verify, Identifier, Signature};
use soroban_sdk::{contractimpl, symbol, BigInt, BytesN, Env};

use crate::{
    types::{DataKey, Error},
    utils::{
        get_contract_id, get_deposit, get_nonce, get_vault, invoke_receiver, is_initialized,
        remove_deposit, set_deposit, set_token, set_vault, try_repay, vault_xfer, xfer_in_pool,
    },
    vault,
};

pub struct FlashLoanCommon;
pub struct FlashLoanBorrow;
pub struct FlashLoanLender;

pub trait Common {
    #[doc = "Initializes the contract. @dev specify: [the token to use, ]"]
    fn init(e: Env, token_id: BytesN<32>, fee_vault: BytesN<32>) -> Result<(), Error>;
}

pub trait Borrow {
    #[doc = "Borrow money specifyng a receiver, which should abide to the &FlashLoanReceiver standard interface"]
    fn borrow(e: Env, receiver_id: Identifier, amount: BigInt) -> Result<(), Error>;
}

pub trait Lender {
    fn prov_liq(e: Env, sig: Signature, amount: BigInt) -> Result<(), Error>;
    fn withdraw(e: Env, sig: Signature) -> Result<(), Error>;
    fn width_fee(e: Env, sig: Signature, shares: BigInt) -> Result<(), Error>;
}

#[contractimpl]
impl Common for FlashLoanCommon {
    fn init(e: Env, token_id: BytesN<32>, fee_vault: BytesN<32>) -> Result<(), Error> {
        let token_key = DataKey::TokenId;
        if e.data().has(token_key) {
            return Err(Error::ContractAlreadyInitialized);
        }

        set_token(&e, token_id);
        set_vault(&e, fee_vault);
        Ok(())
    }
}

#[contractimpl]
impl Borrow for FlashLoanBorrow {
    fn borrow(e: Env, receiver_id: Identifier, amount: BigInt) -> Result<(), Error> {
        if !is_initialized(&e) {
            return Err(Error::Generic);
        }

        if let Identifier::Contract(receiver_id_bytes) = &receiver_id {
            vault_xfer(&e, &receiver_id, &amount)?;
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
    fn prov_liq(e: Env, sig: Signature, amount: BigInt) -> Result<(), Error> {
        if !is_initialized(&e) {
            return Err(Error::Generic);
        }

        let lp_id = sig.identifier(&e);
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

        xfer_in_pool(&e, &lp_id, &amount)?;

        let vault_client = vault::Client::new(&e, get_vault(&e));
        vault_client.deposit(&lp_id, &amount);
        set_deposit(&e, lp_id, amount);

        Ok(())
    }

    fn withdraw(e: Env, sig: Signature) -> Result<(), Error> {
        if !is_initialized(&e) {
            return Err(Error::Generic);
        }

        let lp_id = sig.identifier(&e);
        verify(
            &e,
            &sig,
            symbol!("withdraw"),
            (get_contract_id(&e), get_nonce(&e, sig.identifier(&e))),
        );

        let deposit_amount = get_deposit(&e, lp_id.clone());
        vault_xfer(&e, &lp_id, &deposit_amount)?;

        let vault_client = vault::Client::new(&e, get_vault(&e));
        let fee_shares = vault_client.get_shares(&lp_id);
        vault_client.withd_fee(&lp_id, &fee_shares);
        remove_deposit(&e, lp_id);

        Ok(())
    }

    fn width_fee(e: Env, sig: Signature, shares: BigInt) -> Result<(), Error> {
        if !is_initialized(&e) {
            return Err(Error::Generic);
        }

        let lp_id = sig.identifier(&e);
        verify(
            &e,
            &sig,
            symbol!("withd_fee"),
            (get_contract_id(&e), get_nonce(&e, sig.identifier(&e))),
        );

        let vault_client = vault::Client::new(&e, get_vault(&e));
        vault_client.withd_fee(&lp_id, &shares);

        Ok(())
    }
}
