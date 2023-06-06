use crate::types::Error;
use soroban_sdk::{contractimpl, Address, Env};

pub trait VaultContractTrait {
    /// initialize

    /// Constructor function, only to be callable once

    /// `initialize()` must be provided with:
    /// `admin: Address` The vault's admin, effictively the pool's admin as the vault is the flash loan's admin. The admin in a vault is always the proxy contract.
    /// `token_id: Address` The pool's token.
    /// `flash_loan` The address of the associated flash loan contract. `flash_loan` should have `current_contract_address()` as `lp`.
    fn initialize(
        e: Env,
        admin: Address,
        token_id: Address,
        flash_loan: Address,
    ) -> Result<(), Error>;

    /// deposit

    /// Allows to deposit into the pool and mints liquidity provider shares to the lender.
    /// This action currently must be authorized by the `admin`, so the proxy contract.
    /// This allows a pool to be only funded when the pool is part of the wider protocol, and is not an old pool.
    /// This design decision may be removed in the next release, follow https://github.com/xycloo/xycloans/issues/16

    /// `deposit()` must be provided with:
    /// `from: Address` Address of the liquidity provider.
    /// `amount: i128` Amount of `token_id` that `from` wants to deposit in the pool.
    fn deposit(e: Env, from: Address, amount: i128) -> Result<(), Error>;

    /// deposit_fees

    /// Triggers an update to the `universal_fee_per_share` value, used to calculate rewards.
    /// This action can only be called by the falsh loan contract associated with this vault.
    /// The flash loan will call `deposit_fees` when it has already transferred `amount` fees into the vault.
    /// Fees are in fact stored in the vault contract, not the flash loan contract.

    /// `deposit_fees()` must be provided with:
    /// `amount: i128` Amount of fees that are being deposited in the vault.
    fn deposit_fees(e: Env, amount: i128) -> Result<(), Error>;

    /// update_fee_rewards

    /// Updates the matured rewards for a certain user `addr`
    /// This function may be called by anyone.

    /// `update_fee_rewards()` must be provided with:
    /// `addr: Address` The address that is udpating its fee rewards.
    fn update_fee_rewards(e: Env, addr: Address) -> Result<(), Error>;

    /// withdraw_matured

    /// Allows a certain user `addr` to withdraw the matured fees.
    /// Before calling `withdraw_matured()` the user should call `update_fee_rewards`.
    /// If not, the matured fees that were not updated will not be lost, just not included in the payment.

    /// `withdraw_matured()` must be provided with:
    /// `addr: Address` The address that is withdrawing its fee rewards.
    fn withdraw_matured(e: Env, addr: Address) -> Result<(), Error>;

    /// withdraw

    /// Allows to withdraw liquidity from the pool by burning liquidity provider shares.
    /// Will result in a cross contract call to the flash loan, which holds the funds that are being withdrawn.
    /// The liquidity provider can also withdraw only a portion of its shares.

    /// withdraw() must be provided with:
    /// `addr: Address` Address of the liquidity provider
    /// `amount: i28` Amount of shares that are being withdrawn
    fn withdraw(e: Env, addr: Address, amount: i128) -> Result<(), Error>;

    /// shares

    /// Getter function, returns the amount of shares that `id: Address` holds.
    fn shares(e: Env, id: Address) -> i128;
}

pub struct VaultContract;

#[allow(unused_variables)]
#[contractimpl]
impl VaultContractTrait for VaultContract {
    fn initialize(
        e: Env,
        admin: Address,
        token_id: Address,
        flash_loan: Address,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn deposit_fees(e: Env, amount: i128) -> Result<(), Error> {
        Ok(())
    }

    fn deposit(e: Env, from: Address, amount: i128) -> Result<(), Error> {
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

    fn shares(e: Env, id: Address) -> i128 {
        0
    }
}
