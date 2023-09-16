use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

use crate::types::Error;
use crate::{pool, storage::*};

#[contract]
pub struct XycloansFactory;

pub trait PluggableInterface {
    /// > This function is disabled by default, compile with --features pluggable to enable it.
    ///
    /// Plugs in the protocol a vault contract for a certain token.
    /// Once both the vault and the associated flash loan are plugged in the proxy, there effictively is a new pool in the protocol.
    ///
    /// [`set_vault()`] must be provided with:    /// [`set_vault()`] must be provided with:
    /// [`token_address: Address`] Address of the token used by the vault.
    /// [`pool_address: Address`] Address of the vault contract.
    fn set_pool(env: Env, token_address: Address, pool_address: Address) -> Result<(), Error>;
}

pub trait AdminInterface {
    /// Constructor function, only to be callable once

    /// [`initialize()`] must be provided with:
    /// [`admin: Address`] Address of the proxy's admin

    /// The proxy's admin will only be able to plug in and out pools from the protocol
    /// without having any control over the deposited funds.
    fn initialize(env: Env, admin: Address, pool_hash: BytesN<32>) -> Result<(), Error>;

    /// Deploys a flash loan-vault pair and initializes them accordingly.
    fn deploy_pair(env: Env, token_address: Address, salt: BytesN<32>) -> Result<(), Error>;
}

pub trait Common {
    /// Reads from the storage the flash loan contract for a given token
    fn get_pool_address(env: Env, token_address: Address) -> Result<Address, Error>;
}

#[contractimpl]
impl AdminInterface for XycloansFactory {
    fn initialize(env: Env, admin: Address, pool_hash: BytesN<32>) -> Result<(), Error> {
        if has_admin(&env) {
            return Err(Error::AlreadyInitialized);
        }

        set_admin(&env, admin);
        write_pool_hash(&env, &pool_hash);

        Ok(())
    }

    fn deploy_pair(env: Env, token_address: Address, salt: BytesN<32>) -> Result<(), Error> {
        read_admin(&env)?.require_auth();

        let pool_address = env
            .deployer()
            .with_address(env.current_contract_address(), salt)
            .deploy(read_pool_hash(&env));

        let pool = pool::Client::new(&env, &pool_address);

        pool.initialize(&token_address);

        set_pool(&env, token_address, pool_address);
        Ok(())
    }
}

#[contractimpl]
impl Common for XycloansFactory {
    fn get_pool_address(env: Env, token_address: Address) -> Result<Address, Error> {
        read_pool(&env, token_address)
    }
}

#[cfg(feature = "pluggable")]
impl PluggableInterface for XycloansFactory {
    fn set_pool(env: Env, token_address: Address, pool_address: Address) -> Result<(), Error> {
        read_admin(&env)?.require_auth();

        set_vault(&env, token_address, vault_address);
        Ok(())
    }
}

/*
These traits and implementations have been deprecated in version 0.2.0.

The main reason is that the VM instantiation cost (mostly parsing the WASM) where too significant that we
didn't see the proxy forwarding anything as users will simply invoke the interested flash loan or vault.
The below code would thus be exported anyways taking up bytecode size and cost for every invocation.

For now, we're keeping the code commented out, but in the future we see it being deleted.


use crate::vault::Client;

pub struct ProxyLP;
pub struct ProxyBorrow;

/// All the methods below route the invocations to the requested vault/flash loan.
/// The user must provide the same arguments as in the vault's invocations, plus the `token_address: Address`
/// which tells the proxy which token the user is interacting with, which is then mapped to that token's vault.
pub trait LPTrait {
    fn deposit(
        env: Env,
        lender: Address,
        token_address: Address,
        amount: i128,
    ) -> Result<(), Error>;

    fn update_rewards(env: Env, lender: Address, token_address: Address) -> Result<(), Error>;

    fn withdraw_matured(env: Env, lender: Address, token_address: Address)
        -> Result<(), Error>;

    fn withdraw_liquidity(
        env: Env,
        lender: Address,
        token_address: Address,
        shares: i128,
    ) -> Result<(), Error>;
}

pub trait BorrowTrait {
    fn borrow(
        env: Env,
        token_address: Address,
        amount: i128,
        receiver_address: Address,
    ) -> Result<(), Error>;
}

#[contractimpl]
impl LPTrait for ProxyLP {
    fn deposit(
        env: Env,
        lender: Address,
        token_address: Address,
        amount: i128,
    ) -> Result<(), Error> {
        lender.require_auth();

        let vault = read_vault(&env, token_address)?;
        let vault_client = Client::new(&env, &vault);

        vault_client.deposit(&lender, &amount);
        Ok(())
    }

    fn update_rewards(env: Env, lender: Address, token_address: Address) -> Result<(), Error> {
        let vault = read_vault(&env, token_address)?;
        let vault_client = Client::new(&env, &vault);

        vault_client.update_fee_rewards(&lender);
        Ok(())
    }

    fn withdraw_matured(
        env: Env,
        lender: Address,
        token_address: Address,
    ) -> Result<(), Error> {
        vault_withdraw_matured_fees(&env, lender, token_address)?;

        Ok(())
    }

    fn withdraw_liquidity(
        env: Env,
        lender: Address,
        token_address: Address,
        shares: i128,
    ) -> Result<(), Error> {
        let vault = read_vault(&env, token_address)?;
        let vault_client = Client::new(&env, &vault);

        vault_client.withdraw(&lender, &shares);
        Ok(())
    }
}

#[contractimpl]
impl BorrowTrait for ProxyBorrow {
    fn borrow(
        env: Env,
        token_address: Address,
        amount: i128,
        receiver_address: Address,
    ) -> Result<(), Error> {
        flash_loan_borrow(&env, token_address, amount, receiver_address)?;
        Ok(())
    }
}
 */
