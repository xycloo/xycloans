use soroban_auth::{Identifier, Signature};
use soroban_sdk::{contractimpl, BytesN, Env};

use crate::storage::*;
use crate::types::Error;

pub struct ProxyCommon;
pub struct ProxyLP;
pub struct ProxyBorrow;

pub trait AdminTrait {
    fn initialize(env: Env, admin: Identifier) -> Result<(), Error>;

    fn set_vault(
        env: Env,
        sig: Signature,
        token_contract_id: BytesN<32>,
        vault_contract_id: BytesN<32>,
    ) -> Result<(), Error>;

    fn set_fl(
        env: Env,
        sig: Signature,
        token_contract_id: BytesN<32>,
        flash_loan_contract_id: BytesN<32>,
    ) -> Result<(), Error>;
}

pub trait LPTrait {
    fn deposit(
        env: Env,
        sig: Signature,
        token_contract_id: BytesN<32>,
        amount: i128,
    ) -> Result<(), Error>;

    fn fee_width(
        env: Env,
        sig: Signature,
        token_contract_id: BytesN<32>,
        batch_ts: u64,
        amount: i128,
    ) -> Result<(), Error>;
}

pub trait BorrowTraait {
    fn borrow(
        env: Env,
        token_contract_id: BytesN<32>,
        amount: i128,
        receiver_contract_id: BytesN<32>,
    ) -> Result<(), Error>;
}

#[contractimpl]
impl AdminTrait for ProxyCommon {
    fn initialize(env: Env, admin: Identifier) -> Result<(), Error> {
        if has_admin(&env) {
            return Err(Error::AlreadyInitialized);
        }

        set_admin(&env, admin);
        Ok(())
    }

    fn set_vault(
        env: Env,
        sig: Signature,
        token_contract_id: BytesN<32>,
        vault_contract_id: BytesN<32>,
    ) -> Result<(), Error> {
        check_admin(&env, &sig)?;
        set_vault(&env, token_contract_id, vault_contract_id);
        Ok(())
    }

    fn set_fl(
        env: Env,
        sig: Signature,
        token_contract_id: BytesN<32>,
        flash_loan_contract_id: BytesN<32>,
    ) -> Result<(), Error> {
        check_admin(&env, &sig)?;
        set_flash_loan(&env, token_contract_id, flash_loan_contract_id);
        Ok(())
    }
}

#[contractimpl]
impl LPTrait for ProxyLP {
    fn deposit(
        env: Env,
        sig: Signature,
        token_contract_id: BytesN<32>,
        amount: i128,
    ) -> Result<(), Error> {
        let provider = sig.identifier(&env);
        vault_deposit(&env, provider, token_contract_id, amount)?;
        Ok(())
    }

    fn fee_width(
        env: Env,
        sig: Signature,
        token_contract_id: BytesN<32>,
        batch_ts: u64,
        shares: i128,
    ) -> Result<(), Error> {
        let provider = sig.identifier(&env);
        vault_withdraw_fees(&env, provider, token_contract_id, batch_ts, shares)?;
        Ok(())
    }
}

#[contractimpl]
impl BorrowTraait for ProxyBorrow {
    fn borrow(
        env: Env,
        token_contract_id: BytesN<32>,
        amount: i128,
        receiver_contract_id: BytesN<32>,
    ) -> Result<(), Error> {
        flash_loan_borrow(&env, token_contract_id, amount, receiver_contract_id)?;
        Ok(())
    }
}
