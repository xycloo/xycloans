#![cfg(test)]
//use soroban_auth::{Address, Signature};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, IntoVal};

mod token {
    use soroban_sdk::contractimport;

    contractimport!(file = "../../soroban_token_spec.wasm");
}

mod loan_ctr {
    use soroban_sdk::contractimport;

    contractimport!(file = "../../target/wasm32-unknown-unknown/release/xycloans_flash_loan.wasm");
}

mod receiver_interface {
    use soroban_sdk::contractimport;

    contractimport!(
        file =
            "../../target/wasm32-unknown-unknown/release/soroban_flash_loan_receiver_standard.wasm"
    );
}

mod receiver_ctr {
    use crate::receiver_interface::ReceiverError;
    use soroban_sdk::contractimport;
    contractimport!(file = "../../target/wasm32-unknown-unknown/release/simple.wasm");
}

#[test]
fn test_successful_borrow() {
    let env = Env::default();

    // Beginning of liquidity provider setup and invocations, this part won't be of much interest to developers who only seek to borrow from our flash loans without becoming a liquidity provider/lender
    let u1 = Address::random(&env);
    let lp1 = Address::random(&env);

    let flash_loan_contract_id =
        env.register_contract_wasm(&BytesN::from_array(&env, &[5; 32]), loan_ctr::WASM);
    let flash_loan_client = loan_ctr::Client::new(&env, &flash_loan_contract_id);
    let flash_loan_addr = Address::from_contract_id(&env, &flash_loan_contract_id);

    let token_id = env.register_stellar_asset_contract(u1.clone());
    let token = token::Client::new(&env, &token_id);

    token.mint(&lp1, &1000000000);
    token.transfer(
        &lp1,
        &Address::from_contract_id(&env, &flash_loan_contract_id),
        &1000000000,
    );

    flash_loan_client.init(&token_id, &lp1);

    // Beginning of "developer invocations"

    let receiver_contract = env.register_contract_wasm(None, receiver_ctr::WASM);
    let receiver_client = receiver_ctr::Client::new(&env, &receiver_contract);

    receiver_client.init(&token_id, &flash_loan_addr, &1000000);

    // These `100 $USDC` below are the profits the receiver contract would make. We simply mint the contract some tokens without performing any cdp or arbitrage trading action since it's beyond the scope of the quickstart.
    token.mint(&Address::from_contract_id(&env, &receiver_contract), &1000);

    // Borrowing from the lender, this invocation will result in an invocation to your receiver contract (the one you wrote in `lib.rs`)
    flash_loan_client.borrow(
        &Address::from_contract_id(&env, &receiver_contract),
        &1000000,
    );

    assert_eq!(
        token.balance(&Address::from_contract_id(&env, &receiver_contract)),
        500
    );

    assert_eq!(
        token.balance(&Address::from_contract_id(&env, &flash_loan_contract_id)),
        1000000000
    );
}

/*
#[test]
#[should_panic]
fn test_unsuccessful_borrow() {
    let env = Env::default();

    // Beginning of liquidity provider setup and invocations, this part won't be of much interest to developers who only seek to borrow from our flash loans without becoming a liquidity provider/lender
    let u1 = env.accounts().generate();
    let lp1 = env.accounts().generate();

    let flash_loan_contract_id =
        env.register_contract_wasm(&BytesN::from_array(&env, &[5; 32]), loan_ctr::WASM);
    let flash_loan_client = loan_ctr::Client::new(&env, &flash_loan_contract_id);

    // Test standard token contract
    // Test standard token contract
    let id = env.register_contract_wasm(
        &BytesN::from_array(
            &env,
            &[
                78, 52, 121, 202, 209, 66, 106, 25, 193, 181, 10, 91, 46, 213, 58, 244, 217, 115,
                23, 232, 144, 71, 210, 113, 57, 46, 203, 166, 210, 20, 155, 105,
            ],
        ),
        token::WASM,
    );
    let token = token::Client::new(&env, &id);
    token.initialize(
        &Address::Account(u1.clone()),
        &7u32,
        &"name".into_val(&env),
        &"symbol".into_val(&env),
    );

    token.with_source_account(&u1).mint(
        &Signature::Invoker,
        &0,
        &Address::Account(lp1.clone()),
        &1000000000,
    );
    token.with_source_account(&lp1).transfer(
        &Signature::Invoker,
        &0,
        &Address::Contract(flash_loan_contract_id.clone()),
        &1000000000,
    );

    flash_loan_client.init(&id, &Address::Account(lp1.clone()));

    // Beginning of "developer invocations"

    let receiver_contract = env.register_contract_wasm(None, receiver_ctr::WASM);

    // Borrowing from the lender, this invocation will result in an invocation to your receiver contract (the one you wrote in `lib.rs`)
    flash_loan_client.borrow(&Address::Contract(receiver_contract.clone()), &100000);

    // Assertions to verify that the flash loan went through successfully.
    assert_eq!(token.balance(&Address::Contract(receiver_contract)), 50);
    assert_eq!(
        token.balance(&Address::Contract(flash_loan_contract_id.clone())),
        1000000000
    );
    assert_eq!(token.balance(&Address::Account(lp1)), 50);
    assert_eq!(
        token.balance(&Address::Contract(flash_loan_contract_id)),
        1000000000
    );
}
*/
