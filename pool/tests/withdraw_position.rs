// TODO: probably rewrite this whole test
// there are a couple of things that don't add up.
// we should split the test into smaller functionality tests
// and then simulate a whole sequence.

mod pool {
    use soroban_sdk::contractimport;
    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_pool.wasm");
}

use fixed_point_math::STROOP;
use soroban_sdk::{
    contract, contractimpl, symbol_short, testutils::Address as _, token, Address, Env, Symbol,
};

// Only tests the barebones liquidity withdrawal functionality.
#[test]
fn withdraw_liquidity_raw() {
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

    // initialize the pool.
    pool_client.initialize(&token_id);

    token_admin.mint(&user1, &(100 * STROOP as i128));

    // user1 deposits 50 TOKEN into the pool.
    pool_client.deposit(&user1, &(50 * STROOP as i128));
    assert_eq!(token.balance(&user1), (50 * STROOP as i128));
    assert_eq!(token.balance(&pool_addr), (50 * STROOP as i128));

    pool_client.withdraw(&user1, &(30 * STROOP as i128));
    assert_eq!(token.balance(&user1), (80 * STROOP as i128));
    assert_eq!(token.balance(&pool_addr), (20 * STROOP as i128));
}

#[should_panic(expected = "HostError: Error(Contract, #6)")]
#[test]
fn withdraw_liquidity_0() {
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

    // initialize the pool.
    pool_client.initialize(&token_id);

    token_admin.mint(&user1, &(100 * STROOP as i128));

    // user1 deposits 50 TOKEN into the pool.
    pool_client.deposit(&user1, &(50 * STROOP as i128));
    assert_eq!(token.balance(&user1), (50 * STROOP as i128));
    assert_eq!(token.balance(&pool_addr), (50 * STROOP as i128));

    pool_client.withdraw(&user1, &0);
}


// Tests the liquidity withdrawal functionality after having
// matured yield. Expected behaviour is for the withdrawal
// to not also withdraw matured yield, only to update the
// rewards amount.
#[test]
fn withdraw_liquidity_with_yield_raw() {
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

    // Register, initialize and fund the receiver contract.
    // The receiver contract is needed to generate yield
    // since it borrows a flash loan and repays it + interest.
    let receiver = env.register_contract(None, FlashLoanReceiver);
    let receiver_client = FlashLoanReceiverClient::new(&env, &receiver);
    receiver_client.init(&user1, &token_id, &pool_addr);
    token_admin.mint(&receiver, &(100 * STROOP as i128));

    // initialize the pool.
    pool_client.initialize(&token_id);

    token_admin.mint(&user1, &(100 * STROOP as i128));

    // user1 deposits 50 TOKEN into the pool.
    pool_client.deposit(&user1, &(50 * STROOP as i128));
    assert_eq!(token.balance(&user1), (50 * STROOP as i128));
    assert_eq!(token.balance(&pool_addr), (50 * STROOP as i128));

    // Flash loan borrow occurs.
    // It generates yield which is held in the pool.
    pool_client.borrow(&receiver, &(50 * STROOP as i128));

    // Expected 0.05% fee on the borrow.
    let expected_yield = 400_000;

    pool_client.withdraw(&user1, &(30 * STROOP as i128));
    assert_eq!(token.balance(&user1), (80 * STROOP as i128));
    assert_eq!(
        token.balance(&pool_addr),
        (20 * STROOP as i128) + expected_yield
    );

    assert_eq!(pool_client.matured(&user1), expected_yield)
}

// Tests that once an address' liquidity is out of the pool
// their matured yield doesn't grow. In simpler terms, we're
// checking that the rewards formula works as expected.
#[test]
fn yield_availability() {
    let env: Env = Default::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin1 = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let token_id = env.register_stellar_asset_contract(admin1);
    let token_admin = token::StellarAssetClient::new(&env, &token_id);
    let token = token::Client::new(&env, &token_id);

    let pool_addr = env.register_contract_wasm(&None, pool::WASM);
    let pool_client = pool::Client::new(&env, &pool_addr);

    // Register, initialize and fund the receiver contract.
    // The receiver contract is needed to generate yield
    // since it borrows a flash loan and repays it + interest.
    let receiver = env.register_contract(None, FlashLoanReceiver);
    let receiver_client = FlashLoanReceiverClient::new(&env, &receiver);
    receiver_client.init(&user1, &token_id, &pool_addr);
    token_admin.mint(&receiver, &(100 * STROOP as i128));

    // initialize the pool.
    pool_client.initialize(&token_id);

    token_admin.mint(&user1, &(100 * STROOP as i128));
    token_admin.mint(&user2, &(100 * STROOP as i128));

    // user1 deposits 50 TOKEN into the pool.
    pool_client.deposit(&user1, &(50 * STROOP as i128));
    assert_eq!(token.balance(&user1), (50 * STROOP as i128));
    assert_eq!(token.balance(&pool_addr), (50 * STROOP as i128));

    // user2 deposits 50 TOKEN into the pool.
    pool_client.deposit(&user2, &(50 * STROOP as i128));
    assert_eq!(token.balance(&user2), (50 * STROOP as i128));
    assert_eq!(token.balance(&pool_addr), (100 * STROOP as i128));

    // Flash loan borrow occurs.
    // It generates yield which is held in the pool.
    pool_client.borrow(&receiver, &(50 * STROOP as i128));

    // Expected 0.05% fee on the borrow.
    let expected_yield = 400_000;
    let expected_yield_per_user = expected_yield / 2;

    pool_client.update_fee_rewards(&user1);
    pool_client.withdraw(&user1, &(50 * STROOP as i128));
    assert_eq!(token.balance(&user1), (100 * STROOP as i128));
    assert_eq!(
        token.balance(&pool_addr),
        (50 * STROOP as i128) + expected_yield
    );

    assert_eq!(pool_client.matured(&user1), expected_yield_per_user);

    // Flash loan borrow occurs.
    // It generates yield which is held in the pool.
    pool_client.borrow(&receiver, &(50 * STROOP as i128));
}

#[contract]
pub struct FlashLoanReceiver;

fn compute_fee(amount: &i128) -> i128 {
    amount / 1250
}

#[contractimpl]
impl FlashLoanReceiver {
    pub fn init(e: Env, admin: Address, token: Address, fl_addr: Address) {
        admin.require_auth();
        e.storage().instance().set(&symbol_short!("T"), &token);
        e.storage().instance().set(&symbol_short!("FL"), &fl_addr);
    }

    pub fn exec_op(e: Env) {
        let token_client = token::Client::new(
            &e,
            &e.storage()
                .instance()
                .get::<Symbol, Address>(&symbol_short!("T"))
                .unwrap(),
        );

        let total_amount = (100 * STROOP as i128) + compute_fee(&(100 * STROOP as i128));

        token_client.approve(
            &e.current_contract_address(),
            &e.storage()
                .instance()
                .get::<Symbol, Address>(&symbol_short!("FL"))
                .unwrap(),
            &total_amount,
            &(e.ledger().sequence() + 1),
        );
    }
}
