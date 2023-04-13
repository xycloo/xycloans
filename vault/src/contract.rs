use crate::{
    balance::{burn_shares, mint_shares},
    flash_loan,
    math::{compute_deposit, compute_shares_amount},
    rewards::{pay_matured, update_fee_per_share_universal, update_rewards},
    storage::*,
    token_utility::{get_token_client, read_flash_loan_balance, transfer_into_flash_loan},
    types::Error,
};
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

    fn update_fee_rewards(e: Env, admin: Address, to: Address) -> Result<(), Error>;

    fn withdraw_matured(e: Env, admin: Address, addr: Address) -> Result<(), Error>;

    // needs to be re-implemented
    fn get_shares(e: Env, id: Address) -> i128;

    // should be removed by the end of the update
    fn get_increment(e: Env, id: Address) -> Result<i128, Error>;

    fn withdraw(e: Env, admin: Address, addr: Address, amount: i128) -> Result<(), Error>;
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
        if has_administrator(&e) {
            return Err(Error::VaultAlreadyInitialized);
        }

        write_administrator(&e, admin);
        put_flash_loan(&e, flash_loan);
        put_flash_loan_bytes(&e, flash_loan_bytes);
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

        // transfer the fees in the vault from the flash loan contract
        let client = get_token_client(&e);
        client.transfer(&flash_loan, &e.current_contract_address(), &amount);

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
        let shares = if 0 == total_supply {
            amount
        } else {
            compute_shares_amount(
                amount,
                total_supply,
                read_flash_loan_balance(&e, &token_client), // shares are calculated for the liquidity, since fees aren't re-invested as liquidity we only count the flash loan contract's balance.
            )
        };

        // transfer the funds into the flash loan
        transfer_into_flash_loan(&e, &token_client, &from, &amount);

        // mint the new shares to the lender
        mint_shares(&e, from, shares);

        Ok(())
    }

    fn withdraw_matured(e: Env, admin: Address, addr: Address) -> Result<(), Error> {
        // authenticate the admin and check authorization
        auth_admin(&e, admin)?;

        // pay the matured yield
        pay_matured(&e, addr);

        Ok(())
    }

    fn update_fee_rewards(e: Env, admin: Address, id: Address) -> Result<(), Error> {
        auth_admin(&e, admin)?;
        update_rewards(&e, id);

        Ok(())
    }

    fn withdraw(e: Env, admin: Address, addr: Address, amount: i128) -> Result<(), Error> {
        auth_admin(&e, admin)?;

        // construct the token client we'll use later on
        let token_client = get_token_client(&e);

        let addr_balance = read_balance(&e, addr.clone());

        // if the desired burned shares are more than the lender's balance return an error
        if addr_balance < amount {
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
        let flash_loan_id_bytes = get_flash_loan_bytes(&e);
        let flash_loan_client = flash_loan::Client::new(&e, &flash_loan_id_bytes);
        flash_loan_client.withdraw(&e.current_contract_address(), &addr_deposit, &addr);

        // burn the shares
        burn_shares(&e, addr, amount);

        Ok(())
    }

    fn get_shares(e: Env, id: Address) -> i128 {
        //        get_collected_last_recorded(&e)
        //read_matured_fees_particular(&e, id)
        //        read_fee_per_share_particular(&e, id)
        0
    }

    fn get_increment(e: Env, id: Address) -> Result<i128, Error> {
        Ok(0)
    }
}
