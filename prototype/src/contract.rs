use soroban_auth::{verify, Identifier, Signature};
use soroban_sdk::{contractimpl, symbol, BytesN, Env};

use crate::{
    types::{DataKey, Error},
    utils::{
        get_contract_id, get_lp, get_nonce, get_token_balance, has_lp, invoke_receiver,
        is_initialized, remove_lp, set_lp, set_token, set_vault, try_repay, vault_xfer,
        xfer_in_pool,
    },
    //    vault,
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
    fn borrow(e: Env, receiver_id: Identifier, amount: i128) -> Result<(), Error>;
}

pub trait Lender {
    fn prov_liq(e: Env, sig: Signature, amount: i128) -> Result<(), Error>;
    fn withdraw(e: Env, sig: Signature) -> Result<(), Error>;
    //    fn width_fee(e: Env, sig: Signature, shares: i128) -> Result<(), Error>;
}

#[contractimpl]
impl Common for FlashLoanCommon {
    fn init(e: Env, token_id: BytesN<32>, fee_vault: BytesN<32>) -> Result<(), Error> {
        let token_key = DataKey::TokenId;
        if e.storage().has(token_key) {
            return Err(Error::ContractAlreadyInitialized);
        }

        set_token(&e, token_id);
        set_vault(&e, fee_vault);
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
            vault_xfer(&e, &receiver_id, &amount)?;
            invoke_receiver(&e, receiver_id_bytes);
            try_repay(&e, &receiver_id, &amount)?;
            Ok(())
        } else {
            Err(Error::Generic)
        }
    }
}

// Lender has to be redefined

#[contractimpl]
impl Lender for FlashLoanLender {
    fn prov_liq(e: Env, sig: Signature, amount: i128) -> Result<(), Error> {
        if !is_initialized(&e) || has_lp(&e) {
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
        set_lp(&e, lp_id);

        //        let vault_client = vault::Client::new(&e, get_vault(&e));
        //        vault_client.deposit(&lp_id, &amount);
        //        set_deposit(&e, lp_id, amount);

        Ok(())
    }

    fn withdraw(e: Env, sig: Signature) -> Result<(), Error> {
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

        //        let deposit_amount = get_deposit(&e, lp_id.clone());
        //        vault_xfer(&e, &lp_id, &deposit_amount)?;

        //        let vault_client = vault::Client::new(&e, get_vault(&e));
        //        let fee_shares = vault_client.get_shares(&lp_id);
        //        vault_client.withd_fee(&lp_id, &fee_shares);
        //        remove_deposit(&e, lp_id);

        let amount = get_token_balance(&e);
        vault_xfer(&e, &lp_id, &amount)?;
        remove_lp(&e);

        Ok(())
    }

    // fee goes directly to the LP
}
