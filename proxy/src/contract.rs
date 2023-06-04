use soroban_sdk::{contractimpl, Address, Env};

use crate::storage::*;
use crate::types::Error;
use crate::vault::Client;

pub struct ProxyCommon;
pub struct ProxyLP;
pub struct ProxyBorrow;

pub trait AdminTrait {
    /// initialize

    /// Constructor function, only to be callable once

    /// `initialize()` must be provided with:
    /// `admin: Address` Address of the proxy's admin

    /// The proxy's admin will only be able to plug in and out pools from the protocol
    /// without having any control over the deposited funds.
    fn initialize(env: Env, admin: Address) -> Result<(), Error>;

    /// set_vault

    /// Plugs in the protocol a vault contract for a certain token.
    /// Once both the vault and the associated flash loan are plugged in the proxy, there effictively is a new pool in the protocol.

    /// `set_vault()` must be provided with:
    /// `token_contract_id: Address` Address of the token used by the vault.
    /// `vault_contract_id: Address` Address of the vault contract.
    fn set_vault(
        env: Env,
        token_contract_id: Address,
        vault_contract_id: Address,
    ) -> Result<(), Error>;

    /// set_flash_laon

    /// Plugs in the protocol a flash loan contract for a certain token.
    /// Once both the vault and the associated flash loan are plugged in the proxy, there effictively is a new pool in the protocol.

    /// `set_flash_loan()` must be provided with:
    /// `token_contract_id: Address` Address of the token used by the flash loan.
    /// `flash_loan_contract_id: Address` Address of the flash loan contract.
    fn set_flash_loan(
        env: Env,
        token_contract_id: Address,
        flash_loan_contract_id: Address,
    ) -> Result<(), Error>;
}

/// All the methods below route the invocations to the requested vault/flash loan.
/// The user must provide the same arguments as in the vault's invocations, plus the `token_contract_id: Address`
/// which tells the proxy which token the user is interacting with, which is then mapped to that token's vault.  
pub trait LPTrait {
    fn deposit(
        env: Env,
        lender: Address,
        token_contract_id: Address,
        amount: i128,
    ) -> Result<(), Error>;

    fn update_rewards(env: Env, lender: Address, token_contract_id: Address) -> Result<(), Error>;

    fn withdraw_matured(env: Env, lender: Address, token_contract_id: Address)
        -> Result<(), Error>;

    fn withdraw_liquidity(
        env: Env,
        lender: Address,
        token_contract_id: Address,
        shares: i128,
    ) -> Result<(), Error>;
}

pub trait BorrowTrait {
    fn borrow(
        env: Env,
        token_contract_id: Address,
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
        token_contract_id: Address,
        vault_contract_id: Address,
    ) -> Result<(), Error> {
        get_admin(&env)?.require_auth();
        set_vault(&env, token_contract_id, vault_contract_id);
        Ok(())
    }

    fn set_flash_loan(
        env: Env,
        token_contract_id: Address,
        flash_loan_contract_id: Address,
    ) -> Result<(), Error> {
        get_admin(&env)?.require_auth();
        set_flash_loan(&env, token_contract_id, flash_loan_contract_id);
        Ok(())
    }
}

#[contractimpl]
impl LPTrait for ProxyLP {
    fn deposit(
        env: Env,
        lender: Address,
        token_contract_id: Address,
        amount: i128,
    ) -> Result<(), Error> {
        lender.require_auth();

        let vault = get_vault(&env, token_contract_id)?;
        let vault_client = Client::new(&env, &vault);

        vault_client.deposit(&lender, &amount);
        Ok(())
    }

    fn update_rewards(env: Env, lender: Address, token_contract_id: Address) -> Result<(), Error> {
        let vault = get_vault(&env, token_contract_id)?;
        let vault_client = Client::new(&env, &vault);

        vault_client.update_fee_rewards(&lender);
        Ok(())
    }

    fn withdraw_matured(
        env: Env,
        lender: Address,
        token_contract_id: Address,
    ) -> Result<(), Error> {
        vault_withdraw_matured_fees(&env, lender, token_contract_id)?;

        Ok(())
    }

    fn withdraw_liquidity(
        env: Env,
        lender: Address,
        token_contract_id: Address,
        shares: i128,
    ) -> Result<(), Error> {
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
        token_contract_id: Address,
        amount: i128,
        receiver_address: Address,
    ) -> Result<(), Error> {
        flash_loan_borrow(&env, token_contract_id, amount, receiver_address)?;
        Ok(())
    }
}
