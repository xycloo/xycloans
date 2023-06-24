use crate::{
    balance::{burn_shares, mint_shares},
    events, flash_loan,
    rewards::{pay_matured, update_fee_per_share_universal, update_rewards},
    storage::*,
    token_utility::{get_token_client, transfer_into_flash_loan},
    types::Error,
};
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

#[contractimpl]
impl VaultContractTrait for VaultContract {
    fn initialize(
        e: Env,
        admin: Address,
        token_id: Address,
        flash_loan: Address,
    ) -> Result<(), Error> {
        if has_administrator(&e) {
            return Err(Error::VaultAlreadyInitialized);
        }

        write_administrator(&e, admin);
        put_flash_loan(&e, flash_loan);
        put_token_id(&e, token_id);

        Ok(())
    }

    fn deposit_fees(e: Env, amount: i128) -> Result<(), Error> {
        // we assert that it is the paired flash loan that is depositing the fees
        get_flash_loan(&e).require_auth();

        // update the universal fee per share amount here to avoid the need for a collected_last_recorded storage slot.
        update_fee_per_share_universal(&e, amount);

        events::fees_deposited(&e, amount);
        Ok(())
    }

    fn deposit(e: Env, from: Address, amount: i128) -> Result<(), Error> {
        from.require_auth();

        // we update the rewards before the deposit to avoid the abuse of the collected fees by withdrawing them with liquidity that didn't contribute to their generation.
        update_rewards(&e, from.clone());

        // construct the token client we'll use later on
        let token_client = get_token_client(&e);

        // transfer the funds into the flash loan
        transfer_into_flash_loan(&e, &token_client, &from, &amount);

        // mint the new shares to the lender.
        // shares to mint will always be the amount deposited, see https://github.com/xycloo/xycloans/issues/17
        mint_shares(&e, from.clone(), amount);

        events::deposited(&e, from, amount);
        Ok(())
    }

    fn withdraw_matured(e: Env, addr: Address) -> Result<(), Error> {
        // require lender auth for withdrawal
        addr.require_auth();

        // pay the matured yield
        pay_matured(&e, addr.clone())?;

        events::matured_withdrawn(&e, addr);
        Ok(())
    }

    fn update_fee_rewards(e: Env, addr: Address) -> Result<(), Error> {
        update_rewards(&e, addr.clone());

        events::matured_updated(&e, addr);
        Ok(())
    }

    fn withdraw(e: Env, addr: Address, amount: i128) -> Result<(), Error> {
        // require lender auth for withdrawal
        addr.require_auth();

        let addr_balance = read_balance(&e, addr.clone());

        // if the desired burned shares are more than the lender's balance return an error
        // if the amount is 0 return an error to save gas
        if addr_balance < amount || amount == 0 {
            return Err(Error::InvalidShareBalance);
        }

        /*
        This has been depreacted in release 0.2.0, but may be re-switched on in future upgrades.
        The cost of using this approach compared to deposit = amount brings in more costs due to storage reads.

            let token_client = get_token_client(&e);
            let total_supply = get_tot_supply(&e);
            // compute addr's deposit corresponding to the burned shares.
            let addr_deposit = compute_deposit(
                amount,
                total_supply,
                read_flash_loan_balance(&e, &token_client),
            );
         */

        // update addr's rewards
        update_rewards(&e, addr.clone());

        // pay out the corresponding deposit
        let flash_loan = get_flash_loan(&e);
        let flash_loan_client = flash_loan::Client::new(&e, &flash_loan);
        flash_loan_client.withdraw(&amount, &addr);

        // burn the shares
        burn_shares(&e, addr.clone(), amount);

        events::withdrawn(&e, addr, amount);
        Ok(())
    }

    fn shares(e: Env, id: Address) -> i128 {
        read_balance(&e, id)
    }
}
