#![cfg(test)]

use soroban_auth::{Identifier, Signature};
use soroban_sdk::{
    bigint, contractimpl,
    testutils::{Accounts, Ledger, LedgerInfo},
    BigInt, BytesN, Env, IntoVal,
};

mod token {
    use soroban_sdk::contractimport;

    contractimport!(file = "./soroban_token_spec.wasm");
}

mod loan_ctr {
    use soroban_sdk::contractimport;

    contractimport!(
        file = "../target/wasm32-unknown-unknown/release/soroban_flash_loans_prototype.wasm"
    );
}

mod vault_ctr {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/soroban_vault.wasm");
}

mod receiver_interface {
    use soroban_sdk::contractimport;

    contractimport!(
        file = "../target/wasm32-unknown-unknown/release/soroban_flash_loan_receiver_standard.wasm"
    );
}

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
    let lp1 = env.accounts().generate();

    let contract_id =
        env.register_contract_wasm(&BytesN::from_array(&env, &[5; 32]), loan_ctr::WASM);
    let flash_loan_client = loan_ctr::Client::new(&env, &contract_id);

    let increment_contract =
        env.register_contract(&BytesN::from_array(&env, &[2; 32]), BalIncrement);

    let receiver_contract =
        env.register_contract(None, crate::flash_loan_receiver_standard::FlashLoanReceiver);

    let vault_contract =
        env.register_contract_wasm(&BytesN::from_array(&env, &[13; 32]), vault_ctr::WASM);
    let vault_client = vault_ctr::Client::new(&env, &vault_contract);

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
        &Identifier::Account(lp1.clone()),
        &BigInt::from_i32(&env, 1000000000),
    );

    token.with_source_account(&u1).mint(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &Identifier::Contract(increment_contract),
        &BigInt::from_i32(&env, 1000000000),
    );

    token.with_source_account(&lp1).approve(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &Identifier::Contract(contract_id.clone()),
        &BigInt::from_i32(&env, 1000000000),
    );

    vault_client.initialize(&Identifier::Contract(contract_id.clone()), &id, &None);
    flash_loan_client.init(&id, &vault_contract);

    flash_loan_client
        .with_source_account(&lp1)
        .prov_liq(&Signature::Invoker, &bigint!(&env, 1000000000));

    flash_loan_client.borrow(
        &Identifier::Contract(receiver_contract.clone()),
        &bigint!(&env, 100000),
    );

    assert_eq!(
        token.balance(&Identifier::Contract(receiver_contract.clone())),
        50
    );
    assert_eq!(
        token.balance(&Identifier::Contract(contract_id.clone())),
        1000000000
    );

    flash_loan_client
        .with_source_account(&lp1)
        .width_fee(&Signature::Invoker, &bigint!(&env, 500000000));

    assert_eq!(
        token.balance(&Identifier::Contract(vault_contract.clone())),
        25
    );
    assert_eq!(token.balance(&Identifier::Account(lp1.clone())), 25);

    flash_loan_client
        .with_source_account(&lp1)
        .withdraw(&Signature::Invoker);

    assert_eq!(token.balance(&Identifier::Contract(vault_contract)), 0);
    assert_eq!(token.balance(&Identifier::Account(lp1)), 1000000050);
    assert_eq!(token.balance(&Identifier::Contract(contract_id)), 0);
    assert_eq!(token.balance(&Identifier::Contract(receiver_contract)), 50);
    assert_eq!(token.balance(&Identifier::Account(u1)), 0);
}

mod flash_loan_receiver_standard {
    use crate::{receiver_interface, receiver_interface::Contract, token};

    use super::BalIncrementClient;

    use soroban_auth::{Identifier, Signature};
    use soroban_sdk::{bigint, contractimpl, BigInt, BytesN, Env};

    pub struct FlashLoanReceiver;

    fn compute_fee(e: &Env, amount: &BigInt) -> BigInt {
        bigint!(e, 5) * amount / 10000 // 0.05%, still TBD
    }

    #[contractimpl]
    impl receiver_interface::Contract for FlashLoanReceiver {
        fn exec_op(e: Env) -> Result<(), receiver_interface::ReceiverError> {
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

            Ok(())
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
