#![no_std]

use soroban_auth::{Identifier, Signature};
use soroban_sdk::{contractimpl, contracttype, log, vec, BytesN, Env, Vec};

mod token {
    soroban_sdk::contractimport!(file = "../soroban_token_spec.wasm");
}

mod flash_loan {
    soroban_sdk::contractimport!(
        file = "../target/wasm32-unknown-unknown/release/soroban_flash_loans_prototype.wasm"
    );
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    TokenId,
    Admin,
    TotSupply,
    FlashLoan,
    InitialDep(Identifier),
    Nonce(Identifier),
    Batch(BatchKey),
    Batches(Identifier),
}

#[derive(Clone)]
#[contracttype]
pub struct BatchKey(pub Identifier, pub u64);

#[derive(Clone)]
#[contracttype]
pub struct BatchObj {
    init_s: i128,
    deposit: i128,
    curr_s: i128,
}

fn get_contract_id(e: &Env) -> Identifier {
    Identifier::Contract(e.get_current_contract())
}

fn put_tot_supply(e: &Env, supply: i128) {
    let key = DataKey::TotSupply;
    e.storage().set(key, supply);
}

fn get_tot_supply(e: &Env) -> i128 {
    let key = DataKey::TotSupply;
    e.storage().get(key).unwrap_or(Ok(0)).unwrap()
}

fn put_token_id(e: &Env, token_id: BytesN<32>) {
    let key = DataKey::TokenId;
    e.storage().set(key, token_id);
}

fn get_token_id(e: &Env) -> BytesN<32> {
    let key = DataKey::TokenId;
    e.storage().get(key).unwrap().unwrap()
}

fn put_flash_loan(e: &Env, id: BytesN<32>) {
    let key = DataKey::FlashLoan;
    e.storage().set(key, id);
}

fn get_flash_loan(e: &Env) -> BytesN<32> {
    let key = DataKey::FlashLoan;
    e.storage().get(key).unwrap().unwrap()
}

fn get_token_balance(e: &Env) -> i128 {
    let contract_id = get_token_id(e);
    let client = token::Client::new(e, contract_id);

    client.balance(&get_contract_id(e)) + client.balance(&Identifier::Contract(get_flash_loan(e)))
}

fn transfer(e: &Env, to: &Identifier, amount: i128) {
    let client = token::Client::new(e, get_token_id(e));
    client.xfer(
        &Signature::Invoker,
        &client.nonce(&Signature::Invoker.identifier(e)),
        to,
        &amount,
    );
}

fn transfer_in_vault(e: &Env, from: &Identifier, amount: &i128) {
    let client = token::Client::new(e, get_token_id(e));
    let vault_id = get_contract_id(e);

    client.xfer_from(&Signature::Invoker, &0, from, &vault_id, amount);
}

fn has_administrator(e: &Env) -> bool {
    let key = DataKey::Admin;
    e.storage().has(key)
}

fn read_administrator(e: &Env) -> Identifier {
    let key = DataKey::Admin;
    e.storage().get_unchecked(key).unwrap()
}

fn write_administrator(e: &Env, id: Identifier) {
    let key = DataKey::Admin;
    e.storage().set(key, id);
}

fn read_nonce(e: &Env, id: &Identifier) -> i128 {
    let key = DataKey::Nonce(id.clone());
    e.storage().get(key).unwrap_or(Ok(0)).unwrap()
}

fn mint_shares(e: &Env, to: Identifier, shares: i128, deposit: i128) -> u64 {
    let tot_supply = get_tot_supply(e);
    put_tot_supply(e, tot_supply + shares);

    let ts = e.ledger().timestamp();
    let key = DataKey::Batch(BatchKey(to.clone(), ts));

    let val = BatchObj {
        init_s: shares,
        deposit,
        curr_s: shares,
    };

    add_user_batch(e, to, ts);
    e.storage().set(key, val);

    ts
}

fn get_user_batches(e: &Env, id: Identifier) -> Vec<u64> {
    let key = DataKey::Batches(id);
    e.storage()
        .get(key)
        .unwrap_or_else(|| Ok(Vec::new(e)))
        .unwrap()
}

fn add_user_batch(e: &Env, id: Identifier, batch_ts: u64) {
    let mut batches = get_user_batches(e, id.clone());
    batches.push_front(batch_ts);

    let key = DataKey::Batches(id);
    e.storage().set(key, batches);
}

fn remove_user_batch(e: &Env, id: Identifier, batch_ts: u64) {
    let mut batches = get_user_batches(e, id.clone());
    let batch_idx = batches.iter().position(|x| x.unwrap() == batch_ts).unwrap();

    batches.remove(batch_idx as u32);

    let key = DataKey::Batches(id);
    e.storage().set(key, batches);
}

fn burn_shares(e: &Env, to: Identifier, shares: i128, batch_ts: u64) {
    let tot_supply = get_tot_supply(e);
    let key = DataKey::Batch(BatchKey(to.clone(), batch_ts));

    let mut batch: BatchObj = e.storage().get(key.clone()).unwrap().unwrap();
    batch.curr_s -= shares;
    put_tot_supply(e, tot_supply - shares);

    if batch.curr_s == 0 {
        e.storage().remove(key); // if there are 0 shares remove the batch
        remove_user_batch(e, to, batch_ts);
    } else {
        e.storage().set(key, batch);
    }
}

pub trait VaultContractTrait {
    // Sets the admin and the vault's token id
    fn initialize(e: Env, admin: Identifier, token_id: BytesN<32>, flash_loan: BytesN<32>);

    // Returns the nonce for the admin
    fn nonce(e: Env) -> i128;

    // deposit shares into the vault: mints the vault shares to "from"
    fn deposit(e: Env, from: Identifier, amount: i128) -> u64;

    /// withdraw fees
    fn fee_withd(e: Env, to: Identifier, batch_ts: u64, shares: i128);

    // get vault shares for a user
    fn get_shares(e: Env, id: Identifier, batch_ts: u64) -> BatchObj;

    fn batches(e: Env, id: Identifier) -> Vec<u64>;

    fn withdraw(e: Env, to: Identifier) -> i128;
}

pub struct VaultContract;

#[contractimpl]
impl VaultContractTrait for VaultContract {
    fn initialize(e: Env, admin: Identifier, token_id: BytesN<32>, flash_loan: BytesN<32>) {
        log!(&e, "initializing");

        if has_administrator(&e) {
            panic!("admin is already set");
        }

        write_administrator(&e, admin);
        put_flash_loan(&e, flash_loan);
        put_token_id(&e, token_id);
    }

    fn nonce(e: Env) -> i128 {
        read_nonce(&e, &read_administrator(&e))
    }

    fn deposit(e: Env, from: Identifier, amount: i128) -> u64 {
        log!(&e, "depositing");
        transfer_in_vault(&e, &from, &amount);

        let fl_client = flash_loan::Client::new(&e, get_flash_loan(&e));
        let contract_id = get_token_id(&e);
        let token_client = token::Client::new(&e, contract_id);

        token_client.xfer(
            &Signature::Invoker,
            &0,
            &Identifier::Contract(get_flash_loan(&e)),
            &amount,
        );

        //        fl_client.prov_liq(&Signature::Invoker, &amount);

        let tot_supply = get_tot_supply(&e);

        let shares = if 0 == tot_supply {
            amount
        } else {
            (amount * tot_supply) / (get_token_balance(&e) - amount)
        };

        e.storage().set(DataKey::InitialDep(from.clone()), amount);
        mint_shares(&e, from, shares, amount)
    }

    fn get_shares(e: Env, id: Identifier, batch_ts: u64) -> BatchObj {
        let key = DataKey::Batch(BatchKey(id, batch_ts));

        let batch: BatchObj = e.storage().get(key).unwrap().unwrap();

        batch
    }

    fn batches(e: Env, id: Identifier) -> Vec<u64> {
        get_user_batches(&e, id)
    }

    fn fee_withd(e: Env, to: Identifier, batch_ts: u64, shares: i128) {
        let tot_supply = get_tot_supply(&e);
        let tot_bal = get_token_balance(&e);
        let batch: BatchObj = e
            .storage()
            .get(DataKey::Batch(BatchKey(to.clone(), batch_ts)))
            .unwrap()
            .unwrap();
        let deposit = batch.deposit;
        let init_s = batch.init_s;
        let curr_s = batch.curr_s;

        if curr_s < shares {
            panic!("not enough shares");
        }

        let new_deposit = deposit * (shares * 10000000 / init_s) / 10000000;

        let fee_amount = ((tot_bal * shares) / tot_supply) - new_deposit;
        if fee_amount >= 0 {
            transfer(&e, &to, fee_amount);
            burn_shares(&e, to.clone(), shares, batch_ts);
            let new_tot_supply = get_tot_supply(&e);
            let new_tot_bal = get_token_balance(&e);

            //        if curr_s != shares {

            if tot_bal != new_deposit {
                let new_shares = (new_deposit * new_tot_supply) / (new_tot_bal - new_deposit);
                mint_shares(&e, to, new_shares, new_deposit);
            } else {
                let new_shares = (new_deposit * tot_supply) / new_deposit;
                mint_shares(&e, to, new_shares, new_deposit);
            }
        }

        //log!(&e, "new dep: {}, new shares:", new_deposit.clone(),);
    }

    fn withdraw(e: Env, to: Identifier) -> i128 {
        let batches = get_user_batches(&e, to.clone());
        log!(&e, "batches {}", batches.clone());

        let mut amount: i128 = 0;
        let mut temp_supply: i128 = get_tot_supply(&e);
        let mut temp_balance: i128 = get_token_balance(&e);

        for batch_el in batches.iter() {
            let batch_ts = batch_el.unwrap_or_else(|_| panic!("no ts in batch"));

            let key = DataKey::Batch(BatchKey(to.clone(), batch_ts));
            let batch: BatchObj = e
                .storage()
                .get(key.clone())
                .unwrap_or_else(|| panic!("no batch with this id"))
                .unwrap();

            let deposit = batch.deposit;
            let init_s = batch.init_s;
            let curr_s = batch.curr_s;

            let new_deposit = deposit * (curr_s * 10000000 / init_s) / 10000000;
            let fee_amount = ((temp_balance * curr_s) / temp_supply) - new_deposit;

            amount += fee_amount;

            temp_balance -= fee_amount;
            temp_supply -= curr_s;

            //            transfer(&e, to.clone(), fee_amount);
            burn_shares(&e, to.clone(), curr_s, batch_ts);

            if temp_balance != new_deposit {
                temp_supply += (new_deposit * temp_supply) / (temp_balance - new_deposit);
                log!(&e, "deposit != balance", amount);
            } else {
                temp_supply += (new_deposit * temp_supply) / (new_deposit);
                log!(&e, "deposit == balance", amount);
            }
        }

        let initial_deposit = e
            .storage()
            .get::<DataKey, i128>(DataKey::InitialDep(to.clone()))
            .unwrap()
            .unwrap();

        let fl_client = flash_loan::Client::new(&e, get_flash_loan(&e));
        fl_client.withdraw(&Signature::Invoker, &initial_deposit, &to);
        transfer(&e, &to, amount);
        amount
    }
}

// TODO
// minted shares are to be saved in the format:
// SharesBatch(ID, ts) => BatchObj { current_n_shares, deposit, initial_n_shares }
// also add an entry UserBatches(ID) => [ts_0, ts_1, ts_n]

// when minting, make sure to use [1]

// add collect_fees method:
// 1. uses [3] to get fee amount => xfer to lp + burn shares
// 2. then mints new shares batch to lp with [5] + adds to total supply

// withdraw method:
// 1. get sum of all fees -> for ts in userbatches { calc with [5] and sum }
