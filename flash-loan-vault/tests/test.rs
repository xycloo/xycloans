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
    let admin1 = e.accounts().generate(); // generating the usdc admin

    // loan setup

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
    // decimals, name, symbol don't matter in tests
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

    // minting 1000 usdc to user1
    usdc_token
        .with_source_account(&admin1)
        .mint(&Signature::Invoker, &0, &user1_id, &(1000));

    // minting 1000 usdc to user2
    usdc_token
        .with_source_account(&admin1)
        .mint(&Signature::Invoker, &0, &user2_id, &(1000));

    // user 1 deposits 5 usdc into vault
    usdc_token
        .with_source_account(&user1)
        .approve(&Signature::Invoker, &0, &vault_id, &1000);

    usdc_token
        .with_source_account(&user2)
        .approve(&Signature::Invoker, &0, &vault_id, &1000);

    //    log!(&e, "depositing");

    // user1 buys shares from the vault
    vault_client.deposit(&user1_id, &500);

    extern crate std;

    assert_eq!(usdc_token.balance(&user1_id), 500);

    let batch = vault_client.get_shares(&user1_id, &1666359075);
    std::println!("{:?} ", batch.curr_s,);

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
    std::println!("{:?}", batch.curr_s);

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
    std::println!("{:?}", batch.curr_s);

    e.ledger().set(LedgerInfo {
        timestamp: 1867369075,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    std::println!("balance: {:?}", usdc_token.balance(&vault_id));

    vault_client.fee_withd(&user2_id, &1767369075, &1000);

    let batch = vault_client.get_shares(&user2_id, &1867369075);
    std::println!(
        "new u2 batch {:?}, {:?}, {:?}",
        batch.curr_s,
        batch.init_s,
        batch.deposit
    );

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
    /*
        std::println!(
            "u1 balance : {} || should receive fee of {}",
            usdc_token.balance(&user1_id),
            ((25 * 3) / 15) - (5 * (3 * 10000000 / 5) / 10000000)
        );
    */
    //vault_client.fee_withd(&user1_id, &1667369075, &3);
    //    vault_client.fee_withd(&user2_id, &1867369075, &5);

    //    vault_client.fee_withd(&user1_id, &1667369075, &5);

    vault_client.fee_withd(&user2_id, &1867369075, &500);

    let _batch = vault_client.get_shares(&user2_id, &1867369075);

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

    //    vault_client.fee_withd(&user2_id, &1967369075, &468);
    /*

       std::println!(
           "new u2 batch {:?}, {:?}, {:?} || u1 balance: {}",
           batch.curr_s,
           batch.init_s,
           batch.deposit,
           usdc_token.balance(&user1_id)
       );

       std::println!(
           "user 1 batch: {:?} || user 2 batch {:?} || {:?}",
           vault_client.batches(&user1_id),
           vault_client.batches(&user2_id),
           vault_client.get_shares(&user2_id, &1867369075).deposit
       );
    */

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
        "vault u2 withdraw all fees result: {:?}",
        vault_client.withdraw(&user2_id)
    );

    //    vault_client.withdraw(&user1_id);

    std::println!(
        "vault u1 withdraw all fees result: {:?}",
        vault_client.withdraw(&user1_id)
    );

    /*
        vault_client.fee_withd(&user2_id, &1867369075, &500);
        vault_client.fee_withd(&user2_id, &1967369075, &468);

        vault_client.fee_withd(&user1_id, &1667369075, &500);
    */
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
    //    std::println!("{}", logs.join("\n"));

    //    assert_eq!(vault_client.get_shares(&user1_id, &batch_ts), 5 as i128);
    /*

    // user 2 deposits 8 usdc into vault
    usdc_token
        .with_source_account(&user2)
        .approve(&Signature::Invoker, &0, &vault_id, &8);

    // user2 buys shares from the vault
    vault_client.deposit(&user2_id, &8);

    // the vault generates yield
    usdc_token
        .with_source_account(&admin1)
        .mint(&Signature::Invoker, &0, &vault_id, &13);

    // user1 withdraws from the vault
    vault_client.withdraw(&user1_id, &3);
    assert_eq!(
        usdc_token.with_source_account(&admin1).balance(&user1_id),
        1001
    ); // user 1 now has 1001 USDC and still has 2 shares in the vault.
    assert_eq!(vault_client.get_shares(&user1_id), 2 as i128);*/
}
