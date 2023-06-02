use fixed_point_math::STROOP;

mod vault {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_fl_vault.wasm");
}

mod loan_ctr {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_flash_loan.wasm");
}

use soroban_sdk::{testutils::Address as _, token, Address, Env};

#[test]
fn fee_withdraw_multiple_users() {
    let e: Env = Default::default();
    e.mock_all_auths();

    let admin1 = Address::random(&e);

    let user1 = Address::random(&e);
    let user2 = Address::random(&e);

    let token_id = e.register_stellar_asset_contract(admin1);
    let token = token::Client::new(&e, &token_id);

    let vault_id = e.register_contract_wasm(&None, vault::WASM);
    let vault_client = vault::Client::new(&e, &vault_id);

    let flash_loan_id = e.register_contract_wasm(&None, loan_ctr::WASM);
    //    let flash_loan_client = loan_ctr::Client::new(&e, &flash_loan_contract_id);

    //    flash_loan_client.init(&token_id, &vault_id);
    vault_client.initialize(&user1, &token_id, &flash_loan_id); // user1 is the vault's admin

    token.mint(&user1, &(50 * STROOP as i128));
    token.mint(&user2, &(100 * STROOP as i128));

    vault_client.deposit(&user1, &user1, &(50 * STROOP as i128));

    vault_client.deposit(&user1, &user2, &(100 * STROOP as i128));

    // flash loan generates yield and deposits it into the vault
    token.mint(&vault_id, &(10 * STROOP as i128));
    vault_client.deposit_fees(&flash_loan_id, &(10 * STROOP as i128));

    vault_client.update_fee_rewards(&user1);
    vault_client.update_fee_rewards(&user2);
    vault_client.withdraw_matured(&user2);
    vault_client.withdraw_matured(&user1);

    assert_eq!(token.balance(&user1), 33333300);
    assert_eq!(token.balance(&user2), 66666600);

    // flash loan generates yield and deposits it into the vault
    token.mint(&vault_id, &(10 * STROOP as i128));
    vault_client.deposit_fees(&flash_loan_id, &(10 * STROOP as i128));

    assert_eq!(token.balance(&user1), 33333300);
    assert_eq!(token.balance(&user2), 66666600);

    token.mint(&user2, &(150 * STROOP as i128));
    vault_client.deposit(&user1, &user2, &(150 * STROOP as i128));

    vault_client.update_fee_rewards(&user2);
    vault_client.withdraw_matured(&user2);
    vault_client.update_fee_rewards(&user1);
    vault_client.withdraw_matured(&user1);

    assert_eq!(token.balance(&user1), 66666600); // should receive 1/3 of the deposited fees ~= 3.3 * 1e7 since at the time of the fees deposit user1 held 1/3 of the total supply
    assert_eq!(token.balance(&user2), 66666600 * 2); // should receive 2/3 of the deposited fees =~ 6.6 * 1e7 since at the time of the fees deposit user1 held 2/3 of the total supply. The new deposit at line 76 shouldn't have infuence since the deposited liquidity didn't contribute to the generation of the fees.
    let error: i128 = STROOP.into();
    assert!(token.balance(&vault_id) < error); // at the end of all the fees withdrawals the vault's balance should be ~= 0. We can tolerate an error given by periodic numbers.

    /*vault_client.withdraw_fee(&user1, &user1, &0, &50000000);
    assert_eq!(token.balance(&user1), 0);
    assert_eq!(token.balance(&vault_id), 0);
    assert_eq!(token.balance(&flash_loan_id), 50000000);

    vault_client.deposit(&user1, &user2, &100000000);
    vault_client.withdraw_fee(&user1, &user2, &0, &100000000);
    assert_eq!(token.balance(&user2), 0);
    assert_eq!(token.balance(&vault_id), 0);
    assert_eq!(token.balance(&flash_loan_id), 150000000);

    // flash loans generate yield
    token.mint(&vault_id, &15000); // 1/3 of the deposited liquidity

    vault_client.withdraw_fee(&user1, &user1, &1, &33000000);
    assert_eq!(token.balance(&user1), 3300);
    assert_eq!(token.balance(&vault_id), 11700);
    assert_eq!(token.balance(&flash_loan_id), 150000000);

    vault_client.withdraw_fee(&user1, &user2, &1, &100000000);
    assert_eq!(token.balance(&user2), 10000);
    assert_eq!(token.balance(&vault_id), 1700);
    assert_eq!(token.balance(&flash_loan_id), 150000000);*/
}
