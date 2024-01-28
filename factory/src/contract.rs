use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

use crate::types::{Error, DataKey};
use crate::{pool, storage::*, events};

#[contract]
pub struct XycloansFactory;

pub trait PluggableInterface {
    /// > This function is disabled by default, compile with --features pluggable to enable it.
    ///
    /// Plugs in the protocol a vault contract for a certain token.
    /// Once both the vault and the associated flash loan are plugged in the proxy, there effictively is a new pool in the protocol.
    ///
    /// [`set_pool()`] must be provided with:
    /// [`token_address: Address`] Address of the token used by the vault.
    /// [`pool_address: Address`] Address of the vault contract.
    fn set_pool(env: Env, token_address: Address, pool_address: Address) -> Result<(), Error>;
}

pub trait AdminInterface {
    /// Constructor function, only to be callable once

    /// [`initialize()`] must be provided with:
    /// [`admin: Address`] Address of the proxy's admin
    /// [`pool_hash: BytesN<32>`] Hash of the pool

    /// The proxy's admin will only be able to plug in and out pools from the protocol
    /// without having any control over the deposited funds.
    fn initialize(env: Env, admin: Address, pool_hash: BytesN<32>) -> Result<(), Error>;

    /// Deploys a pool.
    fn deploy_pool(env: Env, token_address: Address, salt: BytesN<32>) -> Result<Address, Error>;
}

pub trait Common {
    /// Reads from the storage the pool contract for a given token
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

    fn deploy_pool(env: Env, token_address: Address, salt: BytesN<32>) -> Result<Address, Error> {
        read_admin(&env)?.require_auth();

        let key = &DataKey::Pool(token_address.clone());
        if env.storage().persistent().has(key) {
            return Err(Error::PoolExists)
        }

        let pool_address = env.deployer().with_current_contract(salt).deploy(read_pool_hash(&env));

        let pool = pool::Client::new(&env, &pool_address);
        pool.initialize(&token_address);

        set_pool(&env, token_address, &pool_address);
        events::deployed_pool(&env, &pool_address);

        Ok(pool_address)
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

        set_pool(&env, token_address, &pool_address);
        Ok(())
    }
}
