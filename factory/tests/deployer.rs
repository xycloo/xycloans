use soroban_sdk::{testutils::Address as _, token, Address, BytesN, Env};

mod factory {
    use soroban_sdk::contractimport;
    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_factory.wasm");
}

mod flash_loan {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_flash_loan.wasm");
}

mod vault {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_fl_vault.wasm");
}

#[test]
fn test_deployer() {
    let e: Env = Default::default();
    e.mock_all_auths();
    let vault_wasm_hash = e.install_contract_wasm(vault::WASM);
    let flash_loan_wasm_hash = e.install_contract_wasm(flash_loan::WASM);

    let token_admin = Address::random(&e);
    let protocol = Address::random(&e);

    let token_address = e.register_stellar_asset_contract(token_admin);

    let factory_id = e.register_contract_wasm(&None, factory::WASM);
    let factory_client = factory::Client::new(&e, &factory_id);

    factory_client.initialize(&protocol, &flash_loan_wasm_hash, &vault_wasm_hash);
    factory_client.deploy_pair(
        &token_address,
        &(
            BytesN::from_array(&e, &[0; 32]),
            BytesN::from_array(&e, &[1; 32]),
        ),
    );

    assert!(factory_client
        .try_get_flash_loan_address(&token_address)
        .is_ok());

    assert!(factory_client.try_get_vault_address(&token_address).is_ok())
}

// Below we test that the factory contract initializes the pair with the correct parameters.

#[test]
fn test_deposit() {
    let env = Env::default();
    env.mock_all_auths();

    let vault_wasm_hash = env.install_contract_wasm(vault::WASM);
    let flash_loan_wasm_hash = env.install_contract_wasm(flash_loan::WASM);

    let token_admin = Address::random(&env);
    let protocol = Address::random(&env);

    let token_address = env.register_stellar_asset_contract(token_admin);

    let factory_id = env.register_contract_wasm(&None, factory::WASM);
    let factory_client = factory::Client::new(&env, &factory_id);

    factory_client.initialize(&protocol, &flash_loan_wasm_hash, &vault_wasm_hash);
    factory_client.deploy_pair(
        &token_address,
        &(
            BytesN::from_array(&env, &[0; 32]),
            BytesN::from_array(&env, &[1; 32]),
        ),
    );

    let user = Address::random(&env);
    let amount = 1000 * 10_i128.pow(7);
    let token = token::Client::new(&env, &token_address);
    token.mint(&user, &amount);

    let vault = vault::Client::new(&env, &factory_client.get_vault_address(&token_address));

    vault.deposit(&user, &amount);

    assert_eq!(
        token.balance(&factory_client.get_flash_loan_address(&token_address)),
        amount
    );
}
