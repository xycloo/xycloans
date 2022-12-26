use soroban_auth::{Identifier, Signature};
use soroban_sdk::{contractimpl, panic_with_error, BigInt, BytesN, Env};

use crate::{
    types::{DataKey, VaultError},
    utils::*,
};

pub trait VaultContractTrait {
    /// Sets the admin and the vault's token id
    fn initialize(
        e: Env,
        admin: Identifier,
        token_id: BytesN<32>,
        max_supply: Option<BigInt>,
    ) -> Result<(), VaultError>;

    /// Returns the nonce for the admin
    fn nonce(e: Env) -> BigInt;

    /// deposit shares into the vault: mints the vault shares to "from"
    fn deposit(e: Env, from: Identifier, amount: BigInt) -> Result<(), VaultError>;

    /// withdraw an ammount of the vault's token id to "to" by burning shares
    fn withd_fee(e: Env, to: Identifier, shares: BigInt) -> Result<(), VaultError>;

    /// get vault shares for a user
    fn get_shares(e: Env, id: Identifier) -> Result<BigInt, VaultError>;
}

pub struct VaultContract;

#[contractimpl]
impl VaultContractTrait for VaultContract {
    fn initialize(
        e: Env,
        admin: Identifier,
        token_id: BytesN<32>,
        max_supply: Option<BigInt>,
    ) -> Result<(), VaultError> {
        if has_administrator(&e) {
            panic!("admin is already set");
        }

        if let Some(supply) = max_supply {
            put_max_supply(&e, supply)
        }

        write_administrator(&e, admin);
        put_token_id(&e, token_id);
        Ok(())
    }

    fn nonce(e: Env) -> BigInt {
        read_nonce(&e, &read_administrator(&e))
    }

    fn deposit(e: Env, from: Identifier, amount: BigInt) -> Result<(), VaultError> {
        if !check_administrator(&e) {
            return Err(VaultError::NotAdmin);
        }

        if e.data().has(DataKey::MaxSupply) && !assert_supply(&e, &amount) {
            return Err(VaultError::SharesExceeded);
        }

        let tot_supply = get_tot_supply(&e);
        let balance = get_token_balance(&e);
        let shares = if BigInt::zero(&e) == balance {
            amount
        } else {
            (amount * tot_supply) / balance
        };

        mint_shares(&e, from, shares);
        Ok(())
    }

    fn get_shares(e: Env, id: Identifier) -> Result<BigInt, VaultError> {
        Ok(e.data()
            .get(DataKey::Balance(id))
            .unwrap_or_else(|| Ok(BigInt::zero(&e)))
            .unwrap())
    }

    fn withd_fee(e: Env, to: Identifier, shares: BigInt) -> Result<(), VaultError> {
        if !check_administrator(&e) {
            return Err(VaultError::NotAdmin);
        }

        let tot_supply = get_tot_supply(&e);
        let amount = (shares.clone() * get_token_balance(&e)) / tot_supply;

        burn_shares(&e, to.clone(), shares);
        transfer(&e, to, amount);
        Ok(())
    }
}
