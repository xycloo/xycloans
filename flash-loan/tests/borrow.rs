#![cfg(test)]

use crate::flash_loan_receiver_standard::FlashLoanReceiverClient;
use crate::flash_loan_receiver_standard_unsuccessful::FlashLoanReceiverClient as FlashLoanReceiverUnsuccessfulClient;
use soroban_sdk::{contractimpl, testutils::Address as _, Address, BytesN, Env, Symbol};

mod token {
    use soroban_sdk::contractimport;

    contractimport!(file = "../soroban_token_spec.wasm");
}

mod loan_ctr {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_flash_loan.wasm");
}

mod receiver_interface {
    use soroban_sdk::contractimport;

    contractimport!(
        file = "../target/wasm32-unknown-unknown/release/soroban_flash_loan_receiver_standard.wasm"
    );
}

#[test]
fn successful_borrow() {
    let env = Env::default();

    let u1 = Address::random(&env);
    let lp1 = Address::random(&env);

    let flash_loan_contract =
        env.register_contract_wasm(&BytesN::from_array(&env, &[5; 32]), loan_ctr::WASM);
    let flash_loan_client = loan_ctr::Client::new(&env, &flash_loan_contract);
    let flash_loan_contract_id = Address::from_contract_id(&env, &flash_loan_contract);

    let increment_contract =
        env.register_contract(&BytesN::from_array(&env, &[2; 32]), BalIncrement);
    let increment_contract_id = Address::from_contract_id(&env, &increment_contract);
    let increment_client = BalIncrementClient::new(&env, &increment_contract);

    let receiver_contract =
        env.register_contract(None, crate::flash_loan_receiver_standard::FlashLoanReceiver);
    let receiver_contract_id = Address::from_contract_id(&env, &receiver_contract);
    let receiver_client = FlashLoanReceiverClient::new(&env, &receiver_contract);

    let id = env.register_stellar_asset_contract(u1.clone());
    let token = token::Client::new(&env, &id);

    receiver_client.init(&u1, &id);
    increment_client.init(&u1, &id);

    token.mint(&lp1, &1000000000);

    token.mint(&increment_contract_id, &1000000000);

    flash_loan_client.init(&id, &lp1);

    token.transfer(&lp1, &flash_loan_contract_id, &1000000000);

    flash_loan_client.borrow(&receiver_contract_id, &100000);

    assert_eq!(token.balance(&receiver_contract_id), 50);
    assert_eq!(token.balance(&flash_loan_contract_id), 1000000000);
    assert_eq!(token.balance(&lp1), 50);
    assert_eq!(token.balance(&flash_loan_contract_id), 1000000000);
    assert_eq!(token.balance(&u1), 0);

    flash_loan_client.withdraw(&lp1, &500000000, &lp1);

    assert_eq!(token.balance(&lp1), 500000000 + 50);
    assert_eq!(token.balance(&flash_loan_contract_id), 500000000);
    assert_eq!(token.balance(&receiver_contract_id), 50);
    assert_eq!(token.balance(&u1), 0);
}

#[test]
#[should_panic(expected = "Status(ContractError(4))")]
fn unsuccessful_borrow() {
    let env = Env::default();

    let u1 = Address::random(&env);
    let lp1 = Address::random(&env);

    let flash_loan_contract =
        env.register_contract_wasm(&BytesN::from_array(&env, &[5; 32]), loan_ctr::WASM);
    let flash_loan_client = loan_ctr::Client::new(&env, &flash_loan_contract);
    let flash_loan_contract_id = Address::from_contract_id(&env, &flash_loan_contract);

    let increment_contract =
        env.register_contract(&BytesN::from_array(&env, &[2; 32]), BalIncrement);
    let increment_contract_id = Address::from_contract_id(&env, &increment_contract);
    let increment_client = BalIncrementClient::new(&env, &increment_contract);

    let receiver_contract = env.register_contract(
        None,
        crate::flash_loan_receiver_standard_unsuccessful::FlashLoanReceiver,
    );
    let receiver_contract_id = Address::from_contract_id(&env, &receiver_contract);
    let receiver_client = FlashLoanReceiverUnsuccessfulClient::new(&env, &receiver_contract);

    let id = env.register_stellar_asset_contract(u1.clone());
    let token = token::Client::new(&env, &id);

    receiver_client.init(&u1, &id);
    increment_client.init(&u1, &id);

    token.mint(&lp1, &1000000000);

    token.mint(&increment_contract_id, &1000000000);

    flash_loan_client.init(&id, &lp1);

    token.transfer(&lp1, &flash_loan_contract_id, &1000000000);

    flash_loan_client.borrow(&receiver_contract_id, &100000);
}

mod flash_loan_receiver_standard {
    use super::BalIncrementClient;
    use crate::{receiver_interface, token};
    use soroban_sdk::{contractimpl, Address, BytesN, Env, Symbol};

    pub struct FlashLoanReceiver;

    fn compute_fee(amount: &i128) -> i128 {
        amount / 2000 // 0.05%, still TBD
    }

    #[contractimpl]
    impl FlashLoanReceiver {
        pub fn init(e: Env, admin: Address, token: BytesN<32>) {
            admin.require_auth();
            e.storage().set(&Symbol::short("T"), &token);
        }

        pub fn exec_op(e: Env) -> Result<(), receiver_interface::ReceiverError> {
            let token_client = token::Client::new(
                &e,
                &e.storage()
                    .get::<Symbol, BytesN<32>>(&Symbol::short("T"))
                    .unwrap()
                    .unwrap(),
            );
            let client = BalIncrementClient::new(&e, &BytesN::from_array(&e, &[2; 32]));

            token_client.transfer(
                &e.current_contract_address(),
                &Address::from_contract_id(&e, &BytesN::from_array(&e, &[2; 32])),
                &100000,
            );
            client.increment(&e.current_contract_address(), &100000);

            let total_amount = 100000 + compute_fee(&100000);

            token_client.increase_allowance(
                &e.current_contract_address(),
                &Address::from_contract_id(&e, &BytesN::from_array(&e, &[5; 32])),
                &total_amount,
            );

            Ok(())
        }
    }
}

mod flash_loan_receiver_standard_unsuccessful {
    use crate::{receiver_interface, token};
    use soroban_sdk::{contractimpl, Address, BytesN, Env, Symbol};

    pub struct FlashLoanReceiver;

    fn compute_fee(amount: &i128) -> i128 {
        amount / 2000 // 0.05%, still TBD
    }

    #[contractimpl]
    impl FlashLoanReceiver {
        pub fn init(e: Env, admin: Address, token: BytesN<32>) {
            admin.require_auth();
            e.storage().set(&Symbol::short("T"), &token);
        }

        pub fn exec_op(e: Env) -> Result<(), receiver_interface::ReceiverError> {
            let token_client = token::Client::new(
                &e,
                &e.storage()
                    .get::<Symbol, BytesN<32>>(&Symbol::short("T"))
                    .unwrap()
                    .unwrap(),
            );

            let total_amount = 100000 + compute_fee(&100000);
            token_client.increase_allowance(
                &e.current_contract_address(),
                &Address::from_contract_id(&e, &BytesN::from_array(&e, &[5; 32])),
                &total_amount,
            );

            Ok(())
        }
    }
}

pub struct BalIncrement;

#[contractimpl]
impl BalIncrement {
    pub fn init(e: Env, admin: Address, token: BytesN<32>) {
        admin.require_auth();
        e.storage().set(&Symbol::short("T"), &token);
    }

    pub fn increment(e: Env, id: Address, amount: i128) {
        let token_client = token::Client::new(
            &e,
            &e.storage()
                .get::<Symbol, BytesN<32>>(&Symbol::short("T"))
                .unwrap()
                .unwrap(),
        );

        token_client.transfer(&e.current_contract_address(), &id, &(amount + 100))
    }

    pub fn decrement(e: Env, id: Address, amount: i128) {
        let token_client = token::Client::new(
            &e,
            &e.storage()
                .get::<Symbol, BytesN<32>>(&Symbol::short("T"))
                .unwrap()
                .unwrap(),
        );

        token_client.transfer(&e.current_contract_address(), &id, &(amount - 100))
    }
}
