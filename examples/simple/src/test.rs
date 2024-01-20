const STROOP: i128 = 10_000_000;

mod pool {
    use soroban_sdk::contractimport;
    contractimport!(file = "../../target/wasm32-unknown-unknown/release/xycloans_pool.wasm");
}

use soroban_sdk::{
    testutils::Address as _, token, Address, Env,
};

use crate::{FlashLoanReceiverContract, FlashLoanReceiverContractClient};

// Tests that an address that has deposited
// liquidity into a pool which has later produced
// yield can collect the generated yield.
#[test]
fn collect_yield_raw() {
    let env: Env = Default::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin1 = Address::generate(&env);

    let user1 = Address::generate(&env);

    let token_id = env.register_stellar_asset_contract(admin1);
    let token_admin = token::StellarAssetClient::new(&env, &token_id);
    let token = token::Client::new(&env, &token_id);

    let pool_addr = env.register_contract_wasm(&None, pool::WASM);
    let pool_client = pool::Client::new(&env, &pool_addr);

    let receiver = env.register_contract(None, FlashLoanReceiverContract);
    let receiver_client = FlashLoanReceiverContractClient::new(&env, &receiver);

    // Initialize the flash loan receiver contract.
    receiver_client.init(&token_id, &pool_addr, &(100 * STROOP));
    pool_client.initialize(&token_id);

    token_admin.mint(&receiver, &(1000 * STROOP));
    token_admin.mint(&user1, &(100 * STROOP));

    // user 1 and 2 deposit into the pool.
    pool_client.deposit(&user1, &(100 * STROOP));

    // Flash loan borrow occurs.
    // It generates yield which is held in the pool.
    pool_client.borrow(&receiver, &(100 * STROOP));
    let expected_yield = 800_000;

    // Update fees and collect matured rewards for user 1
    pool_client.update_fee_rewards(&user1);
    pool_client.withdraw_matured(&user1);

    assert_eq!(token.balance(&user1), expected_yield);
}
