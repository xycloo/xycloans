#![cfg(test)]

mod token {
    use soroban_sdk::contractimport;

    contractimport!(file = "../soroban_token_spec.wasm");
}

mod vault {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/flash_loan_vault.wasm");
}

mod loan_ctr {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/flash_loan.wasm");
}

use soroban_sdk::testutils::Logger;
use soroban_sdk::testutils::{Ledger, LedgerInfo};
use soroban_sdk::{symbol, vec, IntoVal, RawVal, Symbol};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

#[test]
fn workflow() {
    let e: Env = Default::default();
    let admin1 = Address::random(&e);

    let user1 = Address::random(&e);
    let user2 = Address::random(&e);

    let token_id = e.register_stellar_asset_contract(admin1.clone());
    let usdc_token = token::Client::new(&e, &token_id);

    let vault_contract_id =
        e.register_contract_wasm(&BytesN::from_array(&e, &[5; 32]), vault::WASM);
    let vault_client = vault::Client::new(&e, &vault_contract_id);
    let vault_id = Address::from_contract_id(&e, &vault_contract_id);

    let flash_loan_contract_id =
        e.register_contract_wasm(&BytesN::from_array(&e, &[8; 32]), loan_ctr::WASM);
    let flash_loan_id = Address::from_contract_id(&e, &flash_loan_contract_id);
    let flash_loan_client = loan_ctr::Client::new(&e, &flash_loan_contract_id);

    flash_loan_client.init(&token_id, &vault_id);
    vault_client.initialize(&user1, &token_id, &flash_loan_id, &flash_loan_contract_id);

    usdc_token.mint(&admin1, &user1, &1000);
    usdc_token.mint(&admin1, &user2, &1000);

    vault_client.deposit(&user1, &user1, &500);

    assert_eq!(usdc_token.balance(&user1), 500);

    vault_client.fee_withd(&user1, &user1, &0, &500);

    assert_eq!(usdc_token.balance(&user1), 500);

    //    let _batch = vault_client.get_shares(&user1, &0);

    vault_client.deposit(&user1, &user2, &1000);

    assert_eq!(usdc_token.balance(&user2), 0);

    let _batch = vault_client.get_shares(&user2, &0);

    vault_client.fee_withd(&user1, &user2, &0, &1000);

    // fees arrive
    usdc_token.mint(&admin1, &vault_id, &(100));

    vault_client.fee_withd(&user1, &user2, &1, &500);

    let _batch = vault_client.get_shares(&user2, &1);

    extern crate std;
    for batch_el in vault_client.batches(&user1).iter() {
        let el_u = batch_el.unwrap();
        if let Ok(wrapped_batch) = vault_client.try_get_shares(&user1, &el_u) {
            let batch = wrapped_batch.unwrap();
            std::println!(
                "\n\n user 1 batch {:?} is {:?} {:?} {:?} \n",
                el_u,
                batch.curr_s,
                batch.init_s,
                batch.deposit,
            );
        }
    }

    for batch_el in vault_client.batches(&user2).iter() {
        let el_u = batch_el.unwrap();
        if let Ok(wrapped_batch) = vault_client.try_get_shares(&user2, &el_u) {
            let batch = wrapped_batch.unwrap();
            std::println!(
                "\n\n user 2 batch {:?} is {:?} {:?} {:?} \n",
                el_u,
                batch.curr_s,
                batch.init_s,
                batch.deposit,
            );
        }
    }

    std::println!(
        "vault balance : {}, u1 bal : {}, u2 bal : {}",
        usdc_token.balance(&vault_id),
        usdc_token.balance(&user1),
        usdc_token.balance(&user2)
    );

    usdc_token.mint(&admin1, &vault_id, &(100));

    e.ledger().set(LedgerInfo {
        timestamp: 2067369075,
        protocol_version: 1,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
    });

    std::println!(
        "vault balance : {}, u1 bal : {}, u2 bal : {}",
        usdc_token.balance(&vault_id),
        usdc_token.balance(&user1),
        usdc_token.balance(&user2)
    );

    for batch_el in vault_client.batches(&user1).iter() {
        let el_u = batch_el.unwrap();
        if let Ok(wrapped_batch) = vault_client.try_get_shares(&user1, &el_u) {
            let batch = wrapped_batch.unwrap();
            std::println!(
                "\n\n user 1 batch {:?} is {:?} {:?} {:?} \n",
                el_u,
                batch.curr_s,
                batch.init_s,
                batch.deposit,
            );
        }
    }

    for batch_el in vault_client.batches(&user2).iter() {
        let el_u = batch_el.unwrap();
        if let Ok(wrapped_batch) = vault_client.try_get_shares(&user2, &el_u) {
            let batch = wrapped_batch.unwrap();
            std::println!(
                "\n\n user 2 batch {:?} is {:?} {:?} {:?} \n",
                el_u,
                batch.curr_s,
                batch.init_s,
                batch.deposit,
            );
        }
    }

    let _logs = e.logger().all();
}

#[test]
fn workflow_withdraw_position() {
    let e: Env = Default::default();
    let admin1 = Address::random(&e);

    let user1 = Address::random(&e);
    let user2 = Address::random(&e);

    let token_id = e.register_stellar_asset_contract(admin1.clone());
    let usdc_token = token::Client::new(&e, &token_id);

    let vault_contract_id =
        e.register_contract_wasm(&BytesN::from_array(&e, &[5; 32]), vault::WASM);
    let vault_client = vault::Client::new(&e, &vault_contract_id);
    let vault_id = Address::from_contract_id(&e, &vault_contract_id);

    let flash_loan_contract_id =
        e.register_contract_wasm(&BytesN::from_array(&e, &[8; 32]), loan_ctr::WASM);
    let flash_loan_id = Address::from_contract_id(&e, &flash_loan_contract_id);
    let flash_loan_client = loan_ctr::Client::new(&e, &flash_loan_contract_id);

    flash_loan_client.init(&token_id, &vault_id);
    vault_client.initialize(&user1, &token_id, &flash_loan_id, &flash_loan_contract_id);

    usdc_token.mint(&admin1, &user1, &100000000000);
    usdc_token.mint(&admin1, &user2, &100000000000);

    vault_client.deposit(&user1, &user1, &50000000000);

    assert_eq!(usdc_token.balance(&user1), 50000000000);

    vault_client.deposit(&user1, &user2, &100000000000);
    assert_eq!(usdc_token.balance(&user2), 0);

    vault_client.fee_withd(&user1, &user2, &0, &100000000000);
    assert_eq!(usdc_token.balance(&user2), 0);

    // fees arrive
    usdc_token.mint(&admin1, &vault_id, &10000);

    vault_client.fee_withd(&user1, &user2, &1, &100000000000);
    assert_eq!(usdc_token.balance(&user2), 6666);

    vault_client.withdraw(&user1, &user1);
    assert_eq!(usdc_token.balance(&user1), 100000003334);
}

#[test]
fn vault_admin_auth() {
    let e: Env = Default::default();
    let admin1 = Address::random(&e);

    let user1 = Address::random(&e);
    let user2 = Address::random(&e);

    let token_id = e.register_stellar_asset_contract(admin1.clone());
    let token = token::Client::new(&e, &token_id);

    let vault_contract_id =
        e.register_contract_wasm(&BytesN::from_array(&e, &[5; 32]), vault::WASM);
    let vault_client = vault::Client::new(&e, &vault_contract_id);
    let vault_id = Address::from_contract_id(&e, &vault_contract_id);

    let flash_loan_contract_id =
        e.register_contract_wasm(&BytesN::from_array(&e, &[8; 32]), loan_ctr::WASM);
    let flash_loan_id = Address::from_contract_id(&e, &flash_loan_contract_id);
    let flash_loan_client = loan_ctr::Client::new(&e, &flash_loan_contract_id);

    flash_loan_client.init(&token_id, &vault_id);
    vault_client.initialize(&user1, &token_id, &flash_loan_id, &flash_loan_contract_id);

    token.mint(&admin1, &user1, &1000);
    token.mint(&admin1, &user2, &1000);

    vault_client.deposit(&user1, &user1, &500);
    let expected_auth: Vec<(Address, BytesN<32>, Symbol, soroban_sdk::Vec<RawVal>)> = std::vec![(
        user1.clone(),
        vault_contract_id.clone(),
        symbol!("deposit"),
        vec![
            &e,
            user1.into_val(&e),
            user1.into_val(&e),
            500_i128.into_val(&e),
        ],
    )];
    assert_eq!(e.recorded_top_authorizations(), expected_auth);

    vault_client.fee_withd(&user1, &user1, &0, &500);
    let expected_auth: Vec<(Address, BytesN<32>, Symbol, soroban_sdk::Vec<RawVal>)> = std::vec![(
        user1.clone(),
        vault_contract_id.clone(),
        symbol!("fee_withd"),
        vec![
            &e,
            user1.into_val(&e),
            user1.into_val(&e),
            0_i128.into_val(&e),
            500_i128.into_val(&e),
        ],
    )];
    assert_eq!(e.recorded_top_authorizations(), expected_auth);

    vault_client.withdraw(&user1, &user1);
    let expected_auth: Vec<(Address, BytesN<32>, Symbol, soroban_sdk::Vec<RawVal>)> = std::vec![(
        user1.clone(),
        vault_contract_id,
        symbol!("withdraw"),
        vec![&e, user1.into_val(&e), user1.into_val(&e),],
    )];
    assert_eq!(e.recorded_top_authorizations(), expected_auth);
}

#[test]
fn vault_admin_invalid_auth() {
    let e: Env = Default::default();
    let admin1 = Address::random(&e);

    let user1 = Address::random(&e);
    let user2 = Address::random(&e);

    let not_user1 = Address::random(&e);

    let token_id = e.register_stellar_asset_contract(admin1.clone());
    let token = token::Client::new(&e, &token_id);

    let vault_contract_id =
        e.register_contract_wasm(&BytesN::from_array(&e, &[5; 32]), vault::WASM);
    let vault_client = vault::Client::new(&e, &vault_contract_id);
    let vault_id = Address::from_contract_id(&e, &vault_contract_id);

    let flash_loan_contract_id =
        e.register_contract_wasm(&BytesN::from_array(&e, &[8; 32]), loan_ctr::WASM);
    let flash_loan_id = Address::from_contract_id(&e, &flash_loan_contract_id);
    let flash_loan_client = loan_ctr::Client::new(&e, &flash_loan_contract_id);

    flash_loan_client.init(&token_id, &vault_id);
    vault_client.initialize(&user1, &token_id, &flash_loan_id, &flash_loan_contract_id);

    token.mint(&admin1, &user1, &1000);
    token.mint(&admin1, &user2, &1000);

    let _res = vault_client.try_deposit(&not_user1, &user1, &500);
    assert_eq!(e.recorded_top_authorizations(), []);

    let _res = vault_client.try_fee_withd(&not_user1, &user1, &0, &500);
    assert_eq!(e.recorded_top_authorizations(), []);

    let _res = vault_client.try_withdraw(&not_user1, &user1);
    assert_eq!(e.recorded_top_authorizations(), []);
}
