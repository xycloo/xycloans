use soroban_sdk::{testutils::Address as _, token, Address, Env};

mod loan_ctr {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_flash_loan.wasm");
}

#[test]
fn add_liquidity() {
    let env = Env::default();
    env.mock_all_auths();

    let u1 = Address::random(&env);
    let lp1 = Address::random(&env);

    let flash_loan_id = env.register_contract_wasm(&None, loan_ctr::WASM);
    let flash_loan_client = loan_ctr::Client::new(&env, &flash_loan_id);

    let id = env.register_stellar_asset_contract(u1);
    let token = token::Client::new(&env, &id);

    token.mint(&lp1, &1000000000);
    flash_loan_client.init(&id, &lp1);
    token.transfer(&lp1, &flash_loan_id, &1000000000);

    assert_eq!(token.balance(&flash_loan_id), 1000000000);
}
