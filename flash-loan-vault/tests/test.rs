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

//use crate::{VaultContract, VaultContractClient};
use soroban_auth::{Identifier, Signature};
use soroban_sdk::testutils::Logger;
use soroban_sdk::testutils::{Ledger, LedgerInfo};
use soroban_sdk::{testutils::Accounts, BytesN, Env, IntoVal};

#[test]
fn test() {
    let e: Env = Default::default();
    let admin1 = e.accounts().generate();

    e.ledger().set(LedgerInfo {
        timestamp: 1666359075,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    let user1 = e.accounts().generate();
    let user2 = e.accounts().generate();
    let user1_id = Identifier::Account(user1.clone());
    let user2_id = Identifier::Account(user2.clone());

    let token_id = e.register_contract_token(&BytesN::from_array(
        &e,
        &[
            78, 52, 121, 202, 209, 66, 106, 25, 193, 181, 10, 91, 46, 213, 58, 244, 217, 115, 23,
            232, 144, 71, 210, 113, 57, 46, 203, 166, 210, 20, 155, 105,
        ],
    ));
    let usdc_token = token::Client::new(&e, &token_id);
    usdc_token.init(
        &Identifier::Account(admin1.clone()),
        &token::TokenMetadata {
            name: "USD coin".into_val(&e),
            symbol: "USDC".into_val(&e),
            decimals: 7,
        },
    );

    let vault_contract_id =
        e.register_contract_wasm(&BytesN::from_array(&e, &[5; 32]), vault::WASM);
    let vault_client = vault::Client::new(&e, &vault_contract_id);
    let vault_id = Identifier::Contract(vault_contract_id);

    let flash_loan_contract_id =
        e.register_contract_wasm(&BytesN::from_array(&e, &[8; 32]), loan_ctr::WASM);
    let flash_loan_client = loan_ctr::Client::new(&e, &flash_loan_contract_id);

    flash_loan_client.init(&token_id, &vault_id);
    vault_client.initialize(&user1_id, &token_id, &flash_loan_contract_id);

    usdc_token
        .with_source_account(&admin1)
        .mint(&Signature::Invoker, &0, &user1_id, &(1000));

    usdc_token
        .with_source_account(&admin1)
        .mint(&Signature::Invoker, &0, &user2_id, &(1000));

    usdc_token
        .with_source_account(&user1)
        .approve(&Signature::Invoker, &0, &vault_id, &1000);

    usdc_token
        .with_source_account(&user2)
        .approve(&Signature::Invoker, &0, &vault_id, &1000);

    vault_client.deposit(&user1_id, &500);

    assert_eq!(usdc_token.balance(&user1_id), 500);

    let batch = vault_client.get_shares(&user1_id, &1666359075);

    e.ledger().set(LedgerInfo {
        timestamp: 1667369075,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    vault_client.fee_withd(&user1_id, &1666359075, &500);

    assert_eq!(usdc_token.balance(&user1_id), 500);

    let batch = vault_client.get_shares(&user1_id, &1667369075);

    e.ledger().set(LedgerInfo {
        timestamp: 1767369075,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    vault_client.deposit(&user2_id, &1000);

    assert_eq!(usdc_token.balance(&user2_id), 0);

    let batch = vault_client.get_shares(&user2_id, &1767369075);

    e.ledger().set(LedgerInfo {
        timestamp: 1867369075,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    vault_client.fee_withd(&user2_id, &1767369075, &1000);

    // fees arrive
    usdc_token
        .with_source_account(&admin1)
        .mint(&Signature::Invoker, &0, &vault_id, &(100));

    e.ledger().set(LedgerInfo {
        timestamp: 1967369075,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    vault_client.fee_withd(&user2_id, &1867369075, &500);

    let _batch = vault_client.get_shares(&user2_id, &1867369075);

    extern crate std;
    for batch_el in vault_client.batches(&user1_id).iter() {
        let el_u = batch_el.unwrap();
        std::println!(
            "\n\n user 1 batch {:?} is {:?} {:?} {:?} \n",
            el_u,
            vault_client.get_shares(&user1_id, &el_u).curr_s,
            vault_client.get_shares(&user1_id, &el_u).init_s,
            vault_client.get_shares(&user1_id, &el_u).deposit,
        );
    }

    for batch_el in vault_client.batches(&user2_id).iter() {
        let el_u = batch_el.unwrap();
        std::println!(
            "\n\n user 2 batch {:?} is {:?} {:?} {:?} \n",
            el_u,
            vault_client.get_shares(&user2_id, &el_u).curr_s,
            vault_client.get_shares(&user2_id, &el_u).init_s,
            vault_client.get_shares(&user2_id, &el_u).deposit,
        );
    }

    std::println!(
        "vault balance : {}, u1 bal : {}, u2 bal : {}",
        usdc_token.balance(&vault_id),
        usdc_token.balance(&user1_id),
        usdc_token.balance(&user2_id)
    );

    usdc_token
        .with_source_account(&admin1)
        .mint(&Signature::Invoker, &0, &vault_id, &(100));

    e.ledger().set(LedgerInfo {
        timestamp: 2067369075,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    std::println!(
        "vault balance : {}, u1 bal : {}, u2 bal : {}",
        usdc_token.balance(&vault_id),
        usdc_token.balance(&user1_id),
        usdc_token.balance(&user2_id)
    );

    for batch_el in vault_client.batches(&user1_id).iter() {
        let el_u = batch_el.unwrap();
        std::println!(
            "\n\n user 1 batch {:?} is {:?} {:?} {:?} \n",
            el_u,
            vault_client.get_shares(&user1_id, &el_u).curr_s,
            vault_client.get_shares(&user1_id, &el_u).init_s,
            vault_client.get_shares(&user1_id, &el_u).deposit,
        );
    }

    for batch_el in vault_client.batches(&user2_id).iter() {
        let el_u = batch_el.unwrap();
        std::println!(
            "\n\n user 2 batch {:?} is {:?} {:?} {:?} \n",
            el_u,
            vault_client.get_shares(&user2_id, &el_u).curr_s,
            vault_client.get_shares(&user2_id, &el_u).init_s,
            vault_client.get_shares(&user2_id, &el_u).deposit,
        );
    }

    let _logs = e.logger().all();
}
