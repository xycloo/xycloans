#![cfg(test)]

use super::*;

use soroban_sdk::{
    bigint,
    testutils::{Accounts, Ledger, LedgerInfo},
    BytesN, Env, IntoVal,
};

#[test]
fn test_valid_sequence() {
    let env = Env::default();

    env.ledger().set(LedgerInfo {
        timestamp: 1669726145,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    let u1 = env.accounts().generate();
    let u2 = env.accounts().generate();

    let contract_id = env.register_contract(None, AllowancePotContract);
    let client = AllowancePotContractClient::new(&env, &contract_id);

    let id = env.register_contract_token(&BytesN::from_array(
        &env,
        &[
            78, 52, 121, 202, 209, 66, 106, 25, 193, 181, 10, 91, 46, 213, 58, 244, 217, 115, 23,
            232, 144, 71, 210, 113, 57, 46, 203, 166, 210, 20, 155, 105,
        ],
    ));

    let token = token::Client::new(&env, &id);
    // decimals, name, symbol don't matter in tests
    token.init(
        &Identifier::Account(u1.clone()),
        &token::TokenMetadata {
            name: "USD coin".into_val(&env),
            symbol: "USDC".into_val(&env),
            decimals: 7,
        },
    );

    token.with_source_account(&u1).mint(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &Identifier::Account(u1.clone()),
        &BigInt::from_i32(&env, 1000000000),
    );

    token.with_source_account(&u1).approve(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &Identifier::Contract(contract_id.clone()),
        &BigInt::from_i32(&env, 500000000),
    );

    assert_eq!(
        token.allowance(
            &Identifier::Account(u1.clone()),
            &Identifier::Contract(contract_id),
        ),
        500000000
    );
    client
        .with_source_account(&u1)
        .init(&u2, &id, &bigint!(&env, 500000000), &(7 * 24 * 60 * 60));

    env.ledger().set(LedgerInfo {
        timestamp: 1669726146,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    client.with_source_account(&u2).withdraw();
    assert_eq!(token.balance(&Identifier::Account(u2.clone())), 9615384);

    env.ledger().set(LedgerInfo {
        timestamp: 1669726146 + (7 * 24 * 60 * 60) + 1,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    client.with_source_account(&u2).withdraw();
    assert_eq!(token.balance(&Identifier::Account(u2.clone())), 9615384 * 2);

    env.ledger().set(LedgerInfo {
        timestamp: 1669726146 + (7 * 24 * 60 * 60) + (7 * 24 * 60 * 60),
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    client.with_source_account(&u2).withdraw();
}

#[test]
#[should_panic(expected = "Status(ContractError(3))")]
fn test_invalid_sequence() {
    let env = Env::default();

    env.ledger().set(LedgerInfo {
        timestamp: 1669726145,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    let u1 = env.accounts().generate();
    let u2 = env.accounts().generate();

    let contract_id = env.register_contract(None, AllowancePotContract);
    let client = AllowancePotContractClient::new(&env, &contract_id);

    let id = env.register_contract_token(&BytesN::from_array(
        &env,
        &[
            78, 52, 121, 202, 209, 66, 106, 25, 193, 181, 10, 91, 46, 213, 58, 244, 217, 115, 23,
            232, 144, 71, 210, 113, 57, 46, 203, 166, 210, 20, 155, 105,
        ],
    ));

    let token = token::Client::new(&env, &id);
    // decimals, name, symbol don't matter in tests
    token.init(
        &Identifier::Account(u1.clone()),
        &token::TokenMetadata {
            name: "USD coin".into_val(&env),
            symbol: "USDC".into_val(&env),
            decimals: 7,
        },
    );

    token.with_source_account(&u1).mint(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &Identifier::Account(u1.clone()),
        &BigInt::from_i32(&env, 1000000000),
    );

    token.with_source_account(&u1).approve(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &Identifier::Contract(contract_id.clone()),
        &BigInt::from_i32(&env, 500000000),
    );

    assert_eq!(
        token.allowance(
            &Identifier::Account(u1.clone()),
            &Identifier::Contract(contract_id),
        ),
        500000000
    );
    client
        .with_source_account(&u1)
        .init(&u2, &id, &bigint!(&env, 500000000), &(7 * 24 * 60 * 60));

    env.ledger().set(LedgerInfo {
        timestamp: 1669726146,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    client.with_source_account(&u2).withdraw();
    assert_eq!(token.balance(&Identifier::Account(u2.clone())), 9615384);

    env.ledger().set(LedgerInfo {
        timestamp: 1669726146 + (7 * 24 * 60 * 60) + 1,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    client.with_source_account(&u2).withdraw();
    assert_eq!(token.balance(&Identifier::Account(u2.clone())), 9615384 * 2);

    env.ledger().set(LedgerInfo {
        timestamp: 1669726146 + (7 * 24 * 60 * 60) + 1 + 20,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    client.with_source_account(&u2).withdraw();
}
