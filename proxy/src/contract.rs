use soroban_sdk::{contractimpl, Address, BytesN, Env};

use crate::storage::*;
use crate::types::Error;
use crate::vault::{Client, Error as VaultErr};

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

    fn set_flash_loan(
        env: Env,
        admin: Address,
        token_contract_id: BytesN<32>,
        flash_loan_contract_id: BytesN<32>,
    ) -> Result<(), Error>;
}

pub trait LPTrait {
    /// Deposit liquidity into an existing vault
    fn deposit(
        env: Env,
        lender: Address,
        token_contract_id: BytesN<32>,
        amount: i128,
    ) -> Result<(), Error>;

    fn update_rewards(
        env: Env,
        lender: Address,
        token_contract_id: BytesN<32>,
    ) -> Result<(), Error>;

    fn withdraw_matured(
        env: Env,
        lender: Address,
        token_contract_id: BytesN<32>,
    ) -> Result<(), Error>;

    fn withdraw_liquidity(
        env: Env,
        lender: Address,
        token_contract_id: BytesN<32>,
        shares: i128,
    ) -> Result<(), Error>;
}

pub trait BorrowTrait {
    /// Borrow an `amount` of a token through a flash loan
    fn borrow(
        env: Env,
        token_contract_id: BytesN<32>,
        amount: i128,
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
        admin.require_auth();
        set_vault(&env, token_contract_id, vault_contract_id);
        Ok(())
    }

    fn set_flash_loan(
        env: Env,
        admin: Address,
        token_contract_id: BytesN<32>,
        flash_loan_contract_id: BytesN<32>,
    ) -> Result<(), Error> {
        check_admin(&env, &admin)?;
        admin.require_auth();
        set_flash_loan(&env, token_contract_id, flash_loan_contract_id);
        Ok(())
    }
}

#[contractimpl]
impl LPTrait for ProxyLP {
    fn deposit(
        env: Env,
        lender: Address,
        token_contract_id: BytesN<32>,
        amount: i128,
    ) -> Result<(), Error> {
        lender.require_auth();

        let vault = get_vault(&env, token_contract_id)?;
        let vault_client = Client::new(&env, &vault);

        vault_client.deposit(&env.current_contract_address(), &lender, &amount);
        Ok(())
    }

    fn update_rewards(
        env: Env,
        lender: Address,
        token_contract_id: BytesN<32>,
    ) -> Result<(), Error> {
        let vault = get_vault(&env, token_contract_id)?;
        let vault_client = Client::new(&env, &vault);

        vault_client.update_fee_rewards(&lender);
        Ok(())
    }

    fn withdraw_matured(
        env: Env,
        lender: Address,
        token_contract_id: BytesN<32>,
    ) -> Result<(), Error> {
        //        lender.require_auth(); // auth isn't required here as we require it in the vault directly
        vault_withdraw_matured_fees(&env, lender, token_contract_id)?;

        Ok(())
    }

    fn withdraw_liquidity(
        env: Env,
        lender: Address,
        token_contract_id: BytesN<32>,
        shares: i128,
    ) -> Result<(), Error> {
        //        lender.require_auth(); // auth isn't required here as we require it in the vault directly

        let vault = get_vault(&env, token_contract_id)?;
        let vault_client = Client::new(&env, &vault);

        vault_client.withdraw(&lender, &shares);
        Ok(())
    }
}

#[contractimpl]
impl BorrowTrait for ProxyBorrow {
    fn borrow(
        env: Env,
        token_contract_id: BytesN<32>,
        amount: i128,
        receiver_address: Address,
    ) -> Result<(), Error> {
        flash_loan_borrow(&env, token_contract_id, amount, receiver_address)?;
        Ok(())
    }
}
