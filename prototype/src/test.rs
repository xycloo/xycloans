#![cfg(test)]

use super::*;

use soroban_sdk::{
    bigint,
    testutils::{Accounts, Ledger, LedgerInfo},
    BytesN, Env, IntoVal,
};

#[test]
fn test_successful_borrow() {
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

    let contract_id = env.register_contract(None, FlashLoansContract);
    let client = FlashLoansContractClient::new(&env, &contract_id);

    let increment_contract =
        env.register_contract(&BytesN::from_array(&env, &[2; 32]), BalIncrement);
    let increment_client = BalIncrementClient::new(&env, &increment_contract);

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
        &Identifier::Contract(contract_id.clone()),
        &BigInt::from_i32(&env, 1000000000),
    );

    token.with_source_account(&u1).mint(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &Identifier::Contract(increment_contract.clone()),
        &BigInt::from_i32(&env, 1000000000),
    );
    client.init(&id);

    let result = client.with_source_account(&u1).try_borrow(
        &Signature::Invoker,
        &bigint!(&env, 100),
        &symbol!("increment"),
        &increment_contract,
        &(Identifier::Contract(contract_id), bigint!(&env, 100)).into_val(&env),
    );

    assert_eq!(token.balance(&Identifier::Account(u1.clone())), 10);
}

#[test]
fn test_unsuccessful_borrow() {
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

    let contract_id = env.register_contract(None, FlashLoansContract);
    let client = FlashLoansContractClient::new(&env, &contract_id);

    let increment_contract =
        env.register_contract(&BytesN::from_array(&env, &[2; 32]), BalIncrement);
    let increment_client = BalIncrementClient::new(&env, &increment_contract);

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
        &Identifier::Contract(contract_id.clone()),
        &BigInt::from_i32(&env, 1000000000),
    );

    token.with_source_account(&u1).mint(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &Identifier::Contract(increment_contract.clone()),
        &BigInt::from_i32(&env, 1000000000),
    );
    client.init(&id);

    let result = client.with_source_account(&u1).try_borrow(
        &Signature::Invoker,
        &bigint!(&env, 100),
        &symbol!("decrement"),
        &increment_contract,
        &(
            Identifier::Contract(contract_id.clone()),
            bigint!(&env, 100),
        )
            .into_val(&env),
    );

    assert_eq!(token.balance(&Identifier::Account(u1.clone())), 0);
    assert_eq!(
        token.balance(&Identifier::Contract(contract_id)),
        1000000000
    );
    assert_eq!(
        token.balance(&Identifier::Contract(increment_contract)),
        1000000000
    );
}

pub struct BalIncrement;

#[contractimpl]
impl BalIncrement {
    pub fn increment(e: Env, id: Identifier, amount: BigInt) {
        let token_id = BytesN::from_array(
            &e,
            &[
                78, 52, 121, 202, 209, 66, 106, 25, 193, 181, 10, 91, 46, 213, 58, 244, 217, 115,
                23, 232, 144, 71, 210, 113, 57, 46, 203, 166, 210, 20, 155, 105,
            ],
        );
        let client = token::Client::new(&e, token_id);

        client.xfer(
            &Signature::Invoker,
            &BigInt::zero(&e),
            &id,
            &(amount + bigint!(&e, 10)),
        )
    }

    pub fn decrement(e: Env, id: Identifier, amount: BigInt) {
        let token_id = BytesN::from_array(
            &e,
            &[
                78, 52, 121, 202, 209, 66, 106, 25, 193, 181, 10, 91, 46, 213, 58, 244, 217, 115,
                23, 232, 144, 71, 210, 113, 57, 46, 203, 166, 210, 20, 155, 105,
            ],
        );
        let client = token::Client::new(&e, token_id);

        client.xfer(
            &Signature::Invoker,
            &BigInt::zero(&e),
            &id,
            &(amount - bigint!(&e, 10)),
        )
    }
}
