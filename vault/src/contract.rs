use crate::{
    balance::{burn_shares, mint_shares},
    flash_loan,
    math::{compute_deposit, compute_shares_amount},
    rewards::{pay_matured, update_fee_per_share_universal, update_rewards},
    storage::*,
    token_utility::{get_token_client, read_flash_loan_balance, transfer_into_flash_loan},
    types::Error,
};
use soroban_sdk::{contractimpl, Address, Env};

pub trait VaultContractTrait {
    /// initialize

    /// Constructor function, only to be callable once
    /// `initialize()` must be provided with:
    /// `admin: Address` The vault's admin, effictively the pool's admin as the vault is the flash loan's admin.
    /// `token_id: Address` The pool's token.
    /// `flash_loan` The address of the associated flash loan contract. `flash_loan` should have `current_contract_address()` as `lp`.
    fn initialize(
        e: Env,
        admin: Address,
        token_id: Address,
        flash_loan: Address,
    ) -> Result<(), Error>;

    /// Deposits liquidity into the flash loan and mints shares
    fn deposit(e: Env, admin: Address, from: Address, amount: i128) -> Result<(), Error>;

    fn deposit_fees(e: Env, flash_loan: Address, amount: i128) -> Result<(), Error>;

    fn update_fee_rewards(e: Env, addr: Address) -> Result<(), Error>;

    fn withdraw_matured(e: Env, addr: Address) -> Result<(), Error>;

    fn withdraw(e: Env, addr: Address, amount: i128) -> Result<(), Error>;

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

    fn deposit_fees(e: Env, flash_loan: Address, amount: i128) -> Result<(), Error> {
        // we assert that it is the paired flash loan that is depositing the fees
        flash_loan.require_auth();
        let flash_loan_stored = get_flash_loan(&e);
        if flash_loan != flash_loan_stored {
            return Err(Error::InvalidAdminAuth);
        }

        // update the universal fee per share amount here to avoid the need for a collected_last_recorded storage slot.
        update_fee_per_share_universal(&e, amount);

        Ok(())
    }

    fn deposit(e: Env, admin: Address, from: Address, amount: i128) -> Result<(), Error> {
        // authenticate the admin and check authorization
        auth_admin(&e, admin)?;

        // we update the rewards before the deposit to avoid the abuse of the collected fees by withdrawing them with liquidity that didn't contribute to their generation.
        update_rewards(&e, from.clone());

        // construct the token client we'll use later on
        let token_client = get_token_client(&e);

        // calculate the number of shares to mint
        let total_supply = get_tot_supply(&e);
        let total_deposited = read_total_deposited(&e);
        let shares = if 0 == total_supply {
            amount
        } else {
            compute_shares_amount(amount, total_supply, total_deposited)
        };

        // transfer the funds into the flash loan
        transfer_into_flash_loan(&e, &token_client, &from, &amount);
        write_total_deposited(&e, amount);

        // mint the new shares to the lender
        mint_shares(&e, from, shares);

        Ok(())
    }

    fn withdraw_matured(e: Env, addr: Address) -> Result<(), Error> {
        // authenticate the admin and check authorization
        //        auth_admin(&e, admin)?; // auth here shouldn't be required since it locks user capital under the proxy

        // require lender auth for withdrawal
        addr.require_auth();

        // pay the matured yield
        pay_matured(&e, addr)?;

        Ok(())
    }

    fn update_fee_rewards(e: Env, addr: Address) -> Result<(), Error> {
        update_rewards(&e, addr);

        Ok(())
    }

    fn withdraw(e: Env, addr: Address, amount: i128) -> Result<(), Error> {
        // require lender auth for withdrawal
        addr.require_auth();

        // construct the token client we'll use later on
        let token_client = get_token_client(&e);

        let addr_balance = read_balance(&e, addr.clone());

        // if the desired burned shares are more than the lender's balance return an error
        // if the amount is 0 return an error to save gas
        if addr_balance < amount || amount == 0 {
            return Err(Error::InvalidShareBalance);
        }

        let total_supply = get_tot_supply(&e);

        // compute addr's deposit corresponding to the burned shares
        let addr_deposit = compute_deposit(
            amount,
            total_supply,
            read_flash_loan_balance(&e, &token_client),
        );

        // update addr's rewards
        update_rewards(&e, addr.clone());
        //        pay_matured(&e, addr.clone());

        // pay out the corresponding deposit
        //        let flash_loan_id_bytes = get_flash_loan_bytes(&e);
        let flash_loan = get_flash_loan(&e);
        let flash_loan_client = flash_loan::Client::new(&e, &flash_loan);
        flash_loan_client.withdraw(&addr_deposit, &addr);

        // burn the shares
        burn_shares(&e, addr, amount);

        Ok(())
    }

    fn shares(e: Env, id: Address) -> i128 {
        read_balance(&e, id)
    }
}
