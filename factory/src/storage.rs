use soroban_sdk::{unwrap::UnwrapOptimized, Address, BytesN, Env};

use crate::types::{DataKey, Error};

pub(crate) fn set_admin(env: &Env, admin: Address) {
    env.storage().instance().set(&DataKey::Admin, &admin);
}

pub(crate) fn read_admin(env: &Env) -> Result<Address, Error> {
    if let Some(admin_id) = env.storage().instance().get(&DataKey::Admin) {
        Ok(admin_id)
    } else {
        Err(Error::NotInitialized)
    }
}

pub(crate) fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

pub(crate) fn set_pool(env: &Env, token_address: Address, pool_address: &Address) {
    let key = &DataKey::Pool(token_address);
    env.storage().persistent().set(key, &pool_address);
}

pub(crate) fn read_pool(env: &Env, token_address: Address) -> Result<Address, Error> {
    let key = &DataKey::Pool(token_address);
    if let Some(vault_address) = env.storage().persistent().get(key) {
        Ok(vault_address)
    } else {
        Err(Error::NoPool)
    }
}

pub(crate) fn read_pool_hash(env: &Env) -> BytesN<32> {
    env.storage()
        .instance()
        .get(&DataKey::PoolHash)
        .unwrap_optimized()
}

pub(crate) fn write_pool_hash(env: &Env, hash: &BytesN<32>) {
    env.storage().instance().set(&DataKey::PoolHash, hash)
}
