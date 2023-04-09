mod token {
    use soroban_sdk::contractimport;

    contractimport!(file = "../soroban_token_spec.wasm");
}

mod vault {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_fl_vault.wasm");
}

mod loan_ctr {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_flash_loan.wasm");
}

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};
use soroban_sdk::{vec, IntoVal, RawVal, Symbol};

#[test]
fn vault_admin_auth() {
    let e: Env = Default::default();
    let admin1 = Address::random(&e);

    let user1 = Address::random(&e);
    let user2 = Address::random(&e);

    let token_id = e.register_stellar_asset_contract(admin1);
    let token = token::Client::new(&e, &token_id);

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

    token.mint(&user1, &(10 * STROOP as i128));
    token.mint(&user2, &(10 * STROOP as i128));

    vault_client.deposit(&user1, &user1, &(10 * STROOP as i128));
    let expected_auth: Vec<(Address, BytesN<32>, Symbol, soroban_sdk::Vec<RawVal>)> = std::vec![(
        user1.clone(),
        vault_contract_id.clone(),
        Symbol::short("deposit"),
        vec![
            &e,
            user1.into_val(&e),
            user1.into_val(&e),
            (10 * STROOP as i128).into_val(&e),
        ],
    )];
    assert_eq!(e.recorded_top_authorizations(), expected_auth);

    vault_client.withdraw_fee(&user1, &user1, &0, &(5 * STROOP as i128));
    let expected_auth: Vec<(Address, BytesN<32>, Symbol, soroban_sdk::Vec<RawVal>)> = std::vec![(
        user1.clone(),
        vault_contract_id.clone(),
        Symbol::new(&e, "withdraw_fee"),
        vec![
            &e,
            user1.into_val(&e),
            user1.into_val(&e),
            0_i128.into_val(&e),
            (5 * STROOP as i128).into_val(&e),
        ],
    )];
    assert_eq!(e.recorded_top_authorizations(), expected_auth);

    vault_client.withdraw(&user1, &user1);
    let expected_auth: Vec<(Address, BytesN<32>, Symbol, soroban_sdk::Vec<RawVal>)> = std::vec![(
        user1.clone(),
        vault_contract_id,
        Symbol::short("withdraw"),
        vec![&e, user1.into_val(&e), user1.into_val(&e),],
    )];
    assert_eq!(e.recorded_top_authorizations(), expected_auth);
}

#[test]
fn vault_admin_invalid_auth() {
    let e: Env = Default::default();
    let admin1 = Address::random(&e);

    let user1 = Address::random(&e);
    let user2 = Address::random(&e);

    let not_user1 = Address::random(&e);

    let token_id = e.register_stellar_asset_contract(admin1);
    let token = token::Client::new(&e, &token_id);

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

    token.mint(&user1, &(10 * STROOP as i128));
    token.mint(&user2, &(10 * STROOP as i128));

    let _res = vault_client.try_deposit(&not_user1, &user1, &(5 * STROOP as i128));
    assert_eq!(e.recorded_top_authorizations(), []);

    let _res = vault_client.try_withdraw_fee(&not_user1, &user1, &0, &(5 * STROOP as i128));
    assert_eq!(e.recorded_top_authorizations(), []);

    let _res = vault_client.try_withdraw(&not_user1, &user1);
    assert_eq!(e.recorded_top_authorizations(), []);
}

use fixed_point_math::STROOP;
