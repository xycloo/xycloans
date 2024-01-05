use soroban_sdk::{testutils::Address as _, token, Address, BytesN, Env};

mod factory {
    use soroban_sdk::contractimport;
    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_factory.wasm");
}

mod pool {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_pool.wasm");
}

// Tests that the factory contract can deploy a pool contract.
#[test]
fn test_deployer() {
    let env: Env = Default::default();
    env.mock_all_auths();
    let pool_wasm_hash = env.deployer().upload_contract_wasm(pool::WASM);

    let token_admin = Address::generate(&env);
    let protocol = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract(token_admin);

    let factory_id = env.register_contract_wasm(&None, factory::WASM);
    let factory_client = factory::Client::new(&env, &factory_id);

    factory_client.initialize(&protocol, &pool_wasm_hash);
    factory_client.deploy_pool(&token_address, &BytesN::from_array(&env, &[0; 32]));

    assert!(factory_client.try_get_pool_address(&token_address).is_ok());
}

/// Below we test that the factory contract initializes the pair with the correct parameters
/// by trying to deposit into the newly deployed pair.
#[test]
fn test_deposit() {
    let env: Env = Default::default();
    env.mock_all_auths();
    let pool_wasm_hash = env.deployer().upload_contract_wasm(pool::WASM);

    let token_admin = Address::generate(&env);
    let protocol = Address::generate(&env);

    let token_id = env.register_stellar_asset_contract(token_admin);
    let token_admin_client = token::StellarAssetClient::new(&env, &token_id);
    let token = token::Client::new(&env, &token_id);

    let factory_id = env.register_contract_wasm(&None, factory::WASM);
    let factory_client = factory::Client::new(&env, &factory_id);

    factory_client.initialize(&protocol, &pool_wasm_hash);
    factory_client.deploy_pool(&token_id, &BytesN::from_array(&env, &[0; 32]));

    let user = Address::generate(&env);
    let amount = 1000 * 10_i128.pow(7);

    token_admin_client.mint(&user, &amount);

    let vault = pool::Client::new(&env, &factory_client.get_pool_address(&token_id));

    vault.deposit(&user, &amount);

    assert_eq!(
        token.balance(&factory_client.get_pool_address(&token_id)),
        amount
    );
}
