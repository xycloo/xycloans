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

#[test]
fn deposit() {
    let e: Env = Default::default();
    let admin1 = Address::random(&e);

    let user1 = Address::random(&e);
    let user2 = Address::random(&e);
    let user3 = Address::random(&e);

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

    token.mint(&user1, &1000000000);
    token.mint(&user2, &500000000);
    token.mint(&user3, &500000000);

    vault_client.deposit(&user1, &user1, &1000000000);
    assert_eq!(token.balance(&user1), 0);
    assert_eq!(token.balance(&flash_loan_id), 1000000000);
    assert_eq!(token.balance(&vault_id), 0);

    let u1_batch = vault_client.get_shares(&user1, &0);
    assert_eq!(u1_batch.deposit, 1000000000);
    assert_eq!(u1_batch.curr_s, 1000000000);
    assert_eq!(u1_batch.init_s, 1000000000);

    vault_client.deposit(&user1, &user2, &500000000);
    assert_eq!(token.balance(&user2), 0);
    assert_eq!(token.balance(&flash_loan_id), 1500000000);
    assert_eq!(token.balance(&vault_id), 0);

    let u2_batch = vault_client.get_shares(&user2, &0);
    assert_eq!(u2_batch.deposit, 500000000);
    assert_eq!(u2_batch.curr_s, 500000000);
    assert_eq!(u2_batch.init_s, 500000000);

    // flash loans produce yield [this yield is not realistic given the amount of liquidity deposited but serves as a good example to understand the vault's share minting formula]
    token.mint(&vault_id, &1000000000); // half of the deposited liquidity

    vault_client.deposit(&user1, &user3, &500000000);
    assert_eq!(token.balance(&user3), 0);
    assert_eq!(token.balance(&flash_loan_id), 2000000000);
    assert_eq!(token.balance(&vault_id), 1000000000);

    let u2_batch = vault_client.get_shares(&user3, &0);
    assert_eq!(u2_batch.deposit, 500000000);
    assert_eq!(u2_batch.curr_s, 300000000);
    assert_eq!(u2_batch.init_s, 300000000);
}
