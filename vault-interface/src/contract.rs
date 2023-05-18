use crate::types::Error;
use soroban_sdk::{contractimpl, Address, BytesN, Env};

pub trait VaultContractTrait {
    /// Initializes the vault
    /// @param admin Vault admin, the only address that can interact with the vault.
    /// @param token_id Token that the vault manages and pays rewards with.
    /// @param flash_loan Address of the paired flash loan, same token as the vault.
    /// @param flash_loan_bytes Bytes of the flash loan contract [should be deprecated give the new Address::contract_id() method]
    /// @returns an error if the contract has already been initialized.
    fn initialize(
        e: Env,
        admin: Address,
        token_id: BytesN<32>,
        flash_loan: Address,
        flash_loan_bytes: BytesN<32>,
    ) -> Result<(), Error>;

    /// Deposits liquidity into the flash loan and mints shares
    fn deposit(e: Env, admin: Address, from: Address, amount: i128) -> Result<(), Error>;

    fn deposit_fees(e: Env, flash_loan: Address, amount: i128) -> Result<(), Error>;

    fn update_fee_rewards(e: Env, addr: Address) -> Result<(), Error>;

    fn withdraw_matured(e: Env, addr: Address) -> Result<(), Error>;

    // needs to be re-implemented
    fn get_shares(e: Env, id: Address) -> i128;

    // should be removed by the end of the update
    fn get_increment(e: Env, id: Address) -> Result<i128, Error>;

    fn withdraw(e: Env, addr: Address, amount: i128) -> Result<(), Error>;
}

pub struct VaultContract;

#[contractimpl]
impl VaultContractTrait for VaultContract {
    fn initialize(
        e: Env,
        admin: Address,
        token_id: BytesN<32>,
        flash_loan: Address,
        flash_loan_bytes: BytesN<32>,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn deposit_fees(e: Env, flash_loan: Address, amount: i128) -> Result<(), Error> {
        Ok(())
    }

    fn deposit(e: Env, admin: Address, from: Address, amount: i128) -> Result<(), Error> {
        Ok(())
    }

    fn withdraw_matured(e: Env, addr: Address) -> Result<(), Error> {
        Ok(())
    }

    fn update_fee_rewards(e: Env, addr: Address) -> Result<(), Error> {
        Ok(())
    }

    fn withdraw(e: Env, addr: Address, amount: i128) -> Result<(), Error> {
        Ok(())
    }

    fn get_shares(e: Env, id: Address) -> i128 {
        0
    }

    fn get_increment(e: Env, id: Address) -> Result<i128, Error> {
        Ok(0)
    }
}
