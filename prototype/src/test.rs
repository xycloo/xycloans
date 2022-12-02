#![cfg(test)]

use crate::{
    contract::{FlashLoansContract, FlashLoansContractClient},
    test::flash_loan_receiver_standard::FlashLoanReceiver,
    token::Identifier,
    types::Error,
};

use super::*;

use soroban_auth::Signature;
use soroban_sdk::{
    bigint, contractimpl,
    testutils::{Accounts, Ledger, LedgerInfo},
    BigInt, BytesN, Env, IntoVal,
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

    let contract_id =
        env.register_contract(&BytesN::from_array(&env, &[5; 32]), FlashLoansContract);
    let flash_loan_client = FlashLoansContractClient::new(&env, &contract_id);

    let increment_contract =
        env.register_contract(&BytesN::from_array(&env, &[2; 32]), BalIncrement);

    let receiver_contract = env.register_contract(None, FlashLoanReceiver);

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
        &Identifier::Contract(increment_contract),
        &BigInt::from_i32(&env, 1000000000),
    );

    flash_loan_client.init(&id);
    flash_loan_client.borrow(
        &Identifier::Contract(receiver_contract.clone()),
        &bigint!(&env, 100000),
    );

    assert_eq!(token.balance(&Identifier::Contract(receiver_contract)), 50);
    assert_eq!(
        token.balance(&Identifier::Contract(contract_id)),
        1000000050
    );
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

    let contract_id =
        env.register_contract(&BytesN::from_array(&env, &[5; 32]), FlashLoansContract);
    let flash_loan_client = FlashLoansContractClient::new(&env, &contract_id);

    let increment_contract =
        env.register_contract(&BytesN::from_array(&env, &[2; 32]), BalIncrement);

    let receiver_contract = env.register_contract(None, fail::FlashLoanReceiver);

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
        &Identifier::Contract(increment_contract),
        &BigInt::from_i32(&env, 1000000000),
    );

    flash_loan_client.init(&id);
    let res = flash_loan_client.try_borrow(
        &Identifier::Contract(receiver_contract.clone()),
        &bigint!(&env, 100000),
    );

    assert_eq!(res, Err(Ok(Error::GenericRepay)));

    assert_eq!(token.balance(&Identifier::Contract(receiver_contract)), 0);

    assert_eq!(
        token.balance(&Identifier::Contract(contract_id)),
        1000000000
    );
}

mod flash_loan_receiver_standard {
    use super::BalIncrementClient;
    use crate::token::{self, Signature};
    use soroban_auth::Identifier;
    use soroban_sdk::{bigint, contractimpl, BigInt, BytesN, Env};
    pub struct FlashLoanReceiver;

    fn compute_fee(e: &Env, amount: &BigInt) -> BigInt {
        bigint!(e, 5) * amount / 10000 // 0.05%, still TBD
    }

    #[contractimpl]
    impl FlashLoanReceiver {
        pub fn exec_op(e: Env) {
            let token_client = token::Client::new(
                &e,
                &BytesN::from_array(
                    &e,
                    &[
                        78, 52, 121, 202, 209, 66, 106, 25, 193, 181, 10, 91, 46, 213, 58, 244,
                        217, 115, 23, 232, 144, 71, 210, 113, 57, 46, 203, 166, 210, 20, 155, 105,
                    ],
                ),
            );
            let client = BalIncrementClient::new(&e, &BytesN::from_array(&e, &[2; 32]));

            token_client.xfer(
                &Signature::Invoker,
                &BigInt::zero(&e),
                &Identifier::Contract(BytesN::from_array(&e, &[2; 32])),
                &bigint!(&e, 100000),
            );
            client.increment(
                &Identifier::Contract(e.current_contract()),
                &bigint!(&e, 100000),
            );

            let total_amount = bigint!(&e, 100000) + compute_fee(&e, &bigint!(&e, 100000));

            token_client.approve(
                &Signature::Invoker,
                &BigInt::zero(&e),
                &Identifier::Contract(BytesN::from_array(&e, &[5; 32])),
                &total_amount,
            );
        }
    }
}

mod fail {

    use soroban_auth::Identifier;
    use soroban_sdk::{bigint, contractimpl, BigInt, BytesN, Env};

    use crate::token::{self, Signature};

    use super::BalIncrementClient;

    pub struct FlashLoanReceiver;

    fn compute_fee(e: &Env, amount: &BigInt) -> BigInt {
        bigint!(e, 5) * amount / 10000 // 0.05%, still TBD
    }

    #[contractimpl]
    impl FlashLoanReceiver {
        pub fn exec_op(e: Env) {
            let token_client = token::Client::new(
                &e,
                &BytesN::from_array(
                    &e,
                    &[
                        78, 52, 121, 202, 209, 66, 106, 25, 193, 181, 10, 91, 46, 213, 58, 244,
                        217, 115, 23, 232, 144, 71, 210, 113, 57, 46, 203, 166, 210, 20, 155, 105,
                    ],
                ),
            );
            let client = BalIncrementClient::new(&e, &BytesN::from_array(&e, &[2; 32]));

            token_client.xfer(
                &Signature::Invoker,
                &BigInt::zero(&e),
                &Identifier::Contract(BytesN::from_array(&e, &[2; 32])),
                &bigint!(&e, 100000),
            );
            client.decrement(
                &Identifier::Contract(e.current_contract()),
                &bigint!(&e, 100000),
            );

            let total_amount = bigint!(&e, 100000) + compute_fee(&e, &bigint!(&e, 100000));

            token_client.approve(
                &Signature::Invoker,
                &BigInt::zero(&e),
                &Identifier::Contract(BytesN::from_array(&e, &[5; 32])),
                &total_amount,
            );
        }
    }
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
            &(amount + bigint!(&e, 100)),
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
            &(amount - bigint!(&e, 100)),
        )
    }
}
