/*
DEPRECATED WORKFLOW TEST

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

use soroban_sdk::testutils::{Ledger, LedgerInfo};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

#[test]
fn workflow() {
    let e: Env = Default::default();
    let admin1 = Address::random(&e);

    let user1 = Address::random(&e);
    let user2 = Address::random(&e);

    let token_id = e.register_stellar_asset_contract(admin1);
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

    usdc_token.mint(&user1, &100000000);
    usdc_token.mint(&user2, &100000000);

    vault_client.deposit(&user1, &user1, &50000000);

    assert_eq!(usdc_token.balance(&user1), 50000000);
    vault_client.withdraw_fee(&user1, &user1, &0, &50000000);

    //    assert_eq!(usdc_token.balance(&user1), 50000000);

    //    let _batch = vault_client.get_shares(&user1, &0);

    vault_client.deposit(&user1, &user2, &100000000);

    assert_eq!(usdc_token.balance(&user2), 0);

    let _batch = vault_client.get_shares(&user2, &0);

    vault_client.withdraw_fee(&user1, &user2, &0, &100000000);

    // fees arrive
    usdc_token.mint(&vault_id, &1000);

    vault_client.withdraw_fee(&user1, &user2, &1, &50000000);

    let _batch = vault_client.get_shares(&user2, &1);

    extern crate std;
    for el_u in 0..vault_client.get_increment(&user1) {
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

    for el_u in 0..vault_client.get_increment(&user2) {
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

    usdc_token.mint(&vault_id, &(100));

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

    for el_u in 0..vault_client.get_increment(&user1) {
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

    for el_u in 0..vault_client.get_increment(&user2) {
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

    //    let logs = e.logger().all();
    //    std::println!("{:?}", logs);
}
*/
