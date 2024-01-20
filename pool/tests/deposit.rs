mod pool {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_pool.wasm");
}

use soroban_sdk::{testutils::Address as _, token, Address, Env};

#[test]
fn deposit() {
    let e: Env = Default::default();
    e.mock_all_auths();

    let admin1 = Address::generate(&e);

    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let user3 = Address::generate(&e);

    let token_id = e.register_stellar_asset_contract(admin1);
    let token_admin = token::StellarAssetClient::new(&e, &token_id);
    let token = token::Client::new(&e, &token_id);

    let pool_addr = e.register_contract_wasm(&None, pool::WASM); // 5;32
    let pool_client = pool::Client::new(&e, &pool_addr);

    pool_client.initialize(&token_id);

    token_admin.mint(&user1, &1000000000);
    token_admin.mint(&user2, &500000000);
    token_admin.mint(&user3, &500000000);

    pool_client.deposit(&user1, &1000000000);
    assert_eq!(token.balance(&user1), 0);
    assert_eq!(token.balance(&pool_addr), 1000000000);
}


#[should_panic(expected = "HostError: Error(Contract, #6)")]
#[test]
fn deposit_0() {
    let e: Env = Default::default();
    e.mock_all_auths();

    let admin1 = Address::generate(&e);

    let user1 = Address::generate(&e);

    let token_id = e.register_stellar_asset_contract(admin1);

    let pool_addr = e.register_contract_wasm(&None, pool::WASM); // 5;32
    let pool_client = pool::Client::new(&e, &pool_addr);

    pool_client.initialize(&token_id);

    pool_client.deposit(&user1, &0);
}
