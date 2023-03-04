use soroban_sdk::{contractimpl, Address, BytesN, Env};

use crate::storage::*;
use crate::types::Error;
use crate::vault::Client;

pub struct ProxyTest;

pub struct ProxyCommon;
pub struct ProxyLP;
pub struct ProxyBorrow;

pub trait AdminTrait {
    fn initialize(env: Env, admin: Address) -> Result<(), Error>;

    fn set_vault(
        env: Env,
        admin: Address,
        token_contract_id: BytesN<32>,
        vault_contract_id: BytesN<32>,
    ) -> Result<(), Error>;

    fn set_fl(
        env: Env,
        admin: Address,
        token_contract_id: BytesN<32>,
        flash_loan_contract_id: BytesN<32>,
    ) -> Result<(), Error>;
}

pub trait LPTrait {
    /// Deposit liquidity into an existing vault
    fn deposit(env: Env, lender: Address, token_contract_id: BytesN<32>, amount: i128) -> i128;

    fn call_dep(e: Env, provider: Address, vault: BytesN<32>) -> i128;

    /// Withdraw fees for a certain amount of shares of a batch
    fn fee_width(
        env: Env,
        lender: Address,
        token_contract_id: BytesN<32>,
        batch_ts: i128,
        amount: i128,
    ) -> Result<(), Error>;
}

pub trait BorrowTrait {
    /// Borrow an `amount` of a token through a flash loan
    fn borrow(
        env: Env,
        token_contract_id: BytesN<32>,
        amount: i128,
        receiver_contract_id: BytesN<32>,
        receiver_address: Address,
    ) -> Result<(), Error>;
}

#[contractimpl]
impl AdminTrait for ProxyCommon {
    fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if has_admin(&env) {
            return Err(Error::AlreadyInitialized);
        }

        set_admin(&env, admin);
        Ok(())
    }

    fn set_vault(
        env: Env,
        admin: Address,
        token_contract_id: BytesN<32>,
        vault_contract_id: BytesN<32>,
    ) -> Result<(), Error> {
        check_admin(&env, &admin)?;
        set_vault(&env, token_contract_id, vault_contract_id);
        Ok(())
    }

    fn set_fl(
        env: Env,
        admin: Address,
        token_contract_id: BytesN<32>,
        flash_loan_contract_id: BytesN<32>,
    ) -> Result<(), Error> {
        check_admin(&env, &admin)?;
        set_flash_loan(&env, token_contract_id, flash_loan_contract_id);
        Ok(())
    }
}

#[contractimpl]
impl LPTrait for ProxyLP {
    fn deposit(env: Env, lender: Address, token_contract_id: BytesN<32>, amount: i128) -> i128 {
        let vault = get_vault(&env, token_contract_id).unwrap();
        let vault_client = Client::new(&env, &vault);

        vault_client.deposit(&lender, &amount)
    }

    fn call_dep(e: Env, provider: Address, vault: BytesN<32>) -> i128 {
        let vault_client = Client::new(&e, &vault);
        vault_client.deposit(&provider, &2)
    }

    fn fee_width(
        env: Env,
        lender: Address,
        token_contract_id: BytesN<32>,
        batch_n: i128,
        shares: i128,
    ) -> Result<(), Error> {
        vault_withdraw_fees(&env, lender, token_contract_id, batch_n, shares)?;
        Ok(())
    }
}

#[contractimpl]
impl BorrowTrait for ProxyBorrow {
    fn borrow(
        env: Env,
        token_contract_id: BytesN<32>,
        amount: i128,
        receiver_contract_id: BytesN<32>,
        receiver_address: Address,
    ) -> Result<(), Error> {
        flash_loan_borrow(
            &env,
            token_contract_id,
            amount,
            receiver_contract_id,
            receiver_address,
        )?;
        Ok(())
    }
}
