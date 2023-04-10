use core::ops::AddAssign;

use crate::{
    flash_loan,
    math::{
        compute_deposit_ratio, compute_fee_amount, compute_fee_earned, compute_fee_per_share,
        compute_shares_amount,
    },
    rewards::{update_fee_per_share_universal, update_rewards},
    storage::*,
    types::{BatchObj, Error},
};
use soroban_sdk::{contractimpl, log, Address, BytesN, Env};

pub trait VaultContractTrait {
    fn initialize(
        e: Env,
        admin: Address,
        token_id: BytesN<32>,
        flash_loan: Address,
        flash_loan_bytes: BytesN<32>,
    ) -> Result<(), Error>;

    fn deposit(e: Env, admin: Address, from: Address, amount: i128) -> Result<i128, Error>;

    fn deposit_fees(e: Env, flash_loan: Address, amount: i128) -> Result<(), Error>;

    fn update_fee_rewards(e: Env, admin: Address, to: Address) -> Result<(), Error>;

    fn withdraw_matured(e: Env, admin: Address, addr: Address) -> Result<(), Error>;

    fn get_shares(e: Env, id: Address) -> i128;

    fn get_increment(e: Env, id: Address) -> Result<i128, Error>;

    fn withdraw(e: Env, admin: Address, to: Address) -> Result<(), Error>;
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

        update_fee_per_share_universal(&e, amount);

        // add the new fees to the collected slot
        //        let mut collected_last_recorded = get_collected_last_recorded(&e);
        //        collected_last_recorded.add_assign(amount);
        //        put_collected_last_recorded(&e, collected_last_recorded);

        Ok(())
    }

    fn deposit(e: Env, admin: Address, from: Address, amount: i128) -> Result<i128, Error> {
        auth_admin(&e, admin)?;
        update_rewards(&e, from.clone())?;

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
                read_flash_loan_balance(&e, &token_client),
            )
        };

        // transfer the funds into the flash loan
        transfer_into_flash_loan(&e, &token_client, &from, &amount);

        log!(&e, "minting {}", shares);
        mint_shares(&e, from, shares);

        Ok(0)
    }

    fn withdraw_matured(e: Env, admin: Address, addr: Address) -> Result<(), Error> {
        auth_admin(&e, admin)?;

        let token_client = get_token_client(&e);
        let matured = read_matured_fees_particular(&e, addr.clone());

        transfer(&e, &token_client, &addr, matured);
        write_matured_fees_particular(&e, addr, 0);
        Ok(())
    }

    fn update_fee_rewards(e: Env, admin: Address, id: Address) -> Result<(), Error> {
        auth_admin(&e, admin)?;
        update_rewards(&e, id)?;

        //        transfer(&e, &token_client, &id, lender_fees);

        /*        if let Some(Ok(batch)) = get_batch(&e, to.clone(), batch_n) {
                    let total_supply = get_tot_supply(&e);
                    let total_balance = get_token_balance(&e, &token_client);

                    if batch.curr_s < shares {
                        return Err(Error::InvalidShareBalance);
                    }

                    let new_deposit = compute_deposit_ratio(batch.deposit, shares, batch.init_s);
                    let fee_amount = compute_fee_amount(new_deposit, shares, total_supply, total_balance);

                    transfer(&e, &to, fee_amount);
                    burn_shares(&e, to.clone(), shares, batch_n);

                    let new_tot_supply = total_supply - shares;
                    let new_tot_bal = total_balance - fee_amount;

                    let new_shares = if total_balance != new_deposit {
                        compute_shares_amount(new_deposit, new_tot_supply, new_tot_bal - new_deposit)
                    } else {
                        compute_shares_amount(new_deposit, total_supply, new_deposit)
                    };

                    mint_shares(&e, to, new_shares, new_deposit);
                } else {
                    return Err(Error::BatchDoesntExist);
                }
        */
        Ok(())
    }

    fn withdraw(e: Env, admin: Address, to: Address) -> Result<(), Error> {
        auth_admin(&e, admin)?;

        // construct the token client we'll use later on
        let token_client = get_token_client(&e);

        let increment = get_increment(&e, to.clone());

        let mut amount: i128 = 0;
        let mut temp_supply: i128 = get_tot_supply(&e);
        let mut temp_balance: i128 = get_token_balance(&e, &token_client);

        for batch_n in 0..increment {
            if let Some(Ok(batch)) = get_batch(&e, to.clone(), batch_n) {
                let withdrawable_shares = batch.curr_s;
                let new_deposit =
                    compute_deposit_ratio(batch.deposit, withdrawable_shares, batch.init_s);
                let fee_amount =
                    compute_fee_amount(new_deposit, withdrawable_shares, temp_supply, temp_balance);

                amount += fee_amount;
                temp_balance -= fee_amount;
                temp_supply -= withdrawable_shares;

                burn_shares(&e, to.clone(), withdrawable_shares, batch_n);

                if temp_balance != new_deposit {
                    temp_supply +=
                        compute_shares_amount(new_deposit, temp_supply, temp_balance - new_deposit)
                } else {
                    temp_supply += compute_shares_amount(new_deposit, temp_supply, new_deposit)
                }
            }
        }

        let initial_deposit = get_initial_deposit(&e, to.clone());
        let flash_loan_id_bytes = get_flash_loan_bytes(&e);
        let flash_loan_client = flash_loan::Client::new(&e, &flash_loan_id_bytes);
        flash_loan_client.withdraw(&get_contract_addr(&e), &initial_deposit, &to);
        transfer(&e, &token_client, &to, amount);

        Ok(())
    }

    fn get_shares(e: Env, id: Address) -> i128 {
        //        get_collected_last_recorded(&e)
        //read_matured_fees_particular(&e, id)
        read_fee_per_share_particular(&e, id)
    }

    fn get_increment(e: Env, id: Address) -> Result<i128, Error> {
        Ok(get_increment(&e, id))
    }
}
