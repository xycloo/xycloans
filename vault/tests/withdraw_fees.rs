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

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};


#[test]
fn fee_withdraw_multiple_users() {
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
    vault_client.initialize(&user1, &token_id, &flash_loan_id, &flash_loan_contract_id); // user1 is the vault's admin

    token.mint(&user1, &50000000);
    token.mint(&user2, &100000000);

    vault_client.deposit(&user1, &user1, &50000000);
    vault_client.withdraw_fee(&user1, &user1, &0, &50000000);
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

    vault_client.withdraw_fee(&user1, &user1, &1, &50000000);
    assert_eq!(token.balance(&user1), 5000);
    assert_eq!(token.balance(&vault_id), 10000);
    assert_eq!(token.balance(&flash_loan_id), 150000000);

    vault_client.withdraw_fee(&user1, &user2, &1, &100000000);
    assert_eq!(token.balance(&user2), 10000);
    assert_eq!(token.balance(&vault_id), 0);
    assert_eq!(token.balance(&flash_loan_id), 150000000);
}
