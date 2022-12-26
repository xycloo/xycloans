#![cfg(test)]

use soroban_auth::{Identifier, Signature};
use soroban_sdk::{
    contractimpl,
    testutils::{Accounts, Ledger, LedgerInfo},
    BytesN, Env, IntoVal,
};

mod token {
    use soroban_sdk::contractimport;

    contractimport!(file = "../soroban_token_spec.wasm");
}

mod loan_ctr {
    use soroban_sdk::contractimport;

    contractimport!(
        file = "../target/wasm32-unknown-unknown/release/soroban_flash_loans_prototype.wasm"
    );
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

    let flash_loan_contract_id =
        env.register_contract_wasm(&BytesN::from_array(&env, &[5; 32]), loan_ctr::WASM);
    let flash_loan_client = loan_ctr::Client::new(&env, &flash_loan_contract_id);

    let increment_contract =
        env.register_contract(&BytesN::from_array(&env, &[2; 32]), BalIncrement);

    let receiver_contract =
        env.register_contract(None, crate::flash_loan_receiver_standard::FlashLoanReceiver);

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
        &0,
        &Identifier::Account(lp1.clone()),
        &1000000000,
    );

    token.with_source_account(&u1).mint(
        &Signature::Invoker,
        &0,
        &Identifier::Contract(increment_contract),
        &1000000000,
    );

    flash_loan_client.init(&id);

    token.with_source_account(&lp1).approve(
        &Signature::Invoker,
        &0,
        &Identifier::Contract(flash_loan_contract_id.clone()),
        &1000000000,
    );

    flash_loan_client
        .with_source_account(&lp1)
        .prov_liq(&Signature::Invoker, &1000000000);

    flash_loan_client.borrow(&Identifier::Contract(receiver_contract.clone()), &100000);

    assert_eq!(
        token.balance(&Identifier::Contract(receiver_contract.clone())),
        50
    );
    assert_eq!(
        token.balance(&Identifier::Contract(flash_loan_contract_id.clone())),
        1000000000
    );
    assert_eq!(token.balance(&Identifier::Account(lp1.clone())), 50);
    assert_eq!(
        token.balance(&Identifier::Contract(flash_loan_contract_id.clone())),
        1000000000
    );
    assert_eq!(token.balance(&Identifier::Account(u1.clone())), 0);

    flash_loan_client
        .with_source_account(&lp1)
        .withdraw(&Signature::Invoker);

    assert_eq!(token.balance(&Identifier::Account(lp1)), 1000000000 + 50);
    assert_eq!(
        token.balance(&Identifier::Contract(flash_loan_contract_id)),
        0
    );
    assert_eq!(token.balance(&Identifier::Contract(receiver_contract)), 50);
    assert_eq!(token.balance(&Identifier::Account(u1)), 0);
}

mod flash_loan_receiver_standard {
    use crate::{receiver_interface, receiver_interface::Contract, token};

    use super::BalIncrementClient;

    use soroban_auth::{Identifier, Signature};
    use soroban_sdk::{contractimpl, BytesN, Env};

    pub struct FlashLoanReceiver;

    fn compute_fee(amount: &i128) -> i128 {
        5 * amount / 10000 // 0.05%, still TBD
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
                &0,
                &Identifier::Contract(BytesN::from_array(&e, &[2; 32])),
                &100000,
            );
            client.increment(&Identifier::Contract(e.current_contract()), &100000);

            let total_amount = 100000 + compute_fee(&100000);

            token_client.approve(
                &Signature::Invoker,
                &0,
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
    pub fn increment(e: Env, id: Identifier, amount: i128) {
        let token_id = BytesN::from_array(
            &e,
            &[
                78, 52, 121, 202, 209, 66, 106, 25, 193, 181, 10, 91, 46, 213, 58, 244, 217, 115,
                23, 232, 144, 71, 210, 113, 57, 46, 203, 166, 210, 20, 155, 105,
            ],
        );
        let client = token::Client::new(&e, token_id);

        client.xfer(&Signature::Invoker, &0, &id, &(amount + 100))
    }

    pub fn decrement(e: Env, id: Identifier, amount: i128) {
        let token_id = BytesN::from_array(
            &e,
            &[
                78, 52, 121, 202, 209, 66, 106, 25, 193, 181, 10, 91, 46, 213, 58, 244, 217, 115,
                23, 232, 144, 71, 210, 113, 57, 46, 203, 166, 210, 20, 155, 105,
            ],
        );
        let client = token::Client::new(&e, token_id);

        client.xfer(&Signature::Invoker, &0, &id, &(amount - 100))
    }
}
