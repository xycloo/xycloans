use fixed_point_math::STROOP;

mod pool {
    use soroban_sdk::contractimport;
    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_pool.wasm");
}

use soroban_sdk::{
    contract, contractimpl, symbol_short, testutils::Address as _, token, Address, Env, Symbol,
};

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

    let receiver = env.register_contract(None, FlashLoanReceiver);
    let receiver_client = FlashLoanReceiverClient::new(&env, &receiver);

    // Initialize the flash loan receiver contract.
    receiver_client.init(&user1, &token_id, &pool_addr);
    pool_client.initialize(&token_id);

    token_admin.mint(&receiver, &(1000 * STROOP as i128));
    token_admin.mint(&user1, &(100 * STROOP as i128));

    // user 1 and 2 deposit into the pool.
    pool_client.deposit(&user1, &(100 * STROOP as i128));

    // Flash loan borrow occurs.
    // It generates yield which is held in the pool.
    pool_client.borrow(&receiver, &(100 * STROOP as i128));
    let expected_yield = 800_000;

    // Update fees and collect matured rewards for user 1
    pool_client.update_fee_rewards(&user1);
    pool_client.withdraw_matured(&user1);

    assert_eq!(token.balance(&user1), expected_yield);
}

// Tests that the yield amounts collected
// by liquidity providers is proportional
// to the amount of liquidity that they
// provided.
#[test]
fn collect_yield_amounts() {
    let env: Env = Default::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin1 = Address::generate(&env);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);
    let user4 = Address::generate(&env);

    let token_id = env.register_stellar_asset_contract(admin1);
    let token_admin = token::StellarAssetClient::new(&env, &token_id);
    let token = token::Client::new(&env, &token_id);

    let pool_addr = env.register_contract_wasm(&None, pool::WASM);
    let pool_client = pool::Client::new(&env, &pool_addr);

    let receiver = env.register_contract(None, FlashLoanReceiver);
    let receiver_client = FlashLoanReceiverClient::new(&env, &receiver);

    // Initialize the flash loan receiver contract.
    receiver_client.init(&user1, &token_id, &pool_addr);
    pool_client.initialize(&token_id);

    token_admin.mint(&receiver, &(1000 * STROOP as i128));
    token_admin.mint(&user1, &(100 * STROOP as i128));
    token_admin.mint(&user2, &(300 * STROOP as i128));
    token_admin.mint(&user3, &(150 * STROOP as i128));
    token_admin.mint(&user4, &(50 * STROOP as i128));

    // users deposit into the pool.
    pool_client.deposit(&user1, &(100 * STROOP as i128));
    pool_client.deposit(&user2, &(300 * STROOP as i128));
    pool_client.deposit(&user3, &(50 * STROOP as i128));
    pool_client.deposit(&user4, &(50 * STROOP as i128));
    let total_deposited = 500 * STROOP as i128;

    // 20 flash loan borrows of 100 TOKEN occur.
    // They generate yield which is held in the pool.
    for _ in 0..20 {
        pool_client.borrow(&receiver, &(100 * STROOP as i128));
    }

    // Update fees and collect matured rewards for the users
    pool_client.update_fee_rewards(&user1);
    pool_client.withdraw_matured(&user1);

    pool_client.update_fee_rewards(&user2);
    pool_client.withdraw_matured(&user2);

    pool_client.update_fee_rewards(&user3);
    pool_client.withdraw_matured(&user3);

    pool_client.update_fee_rewards(&user4);
    pool_client.withdraw_matured(&user4);

    assert_eq!(token.balance(&pool_addr), total_deposited);
}

// Testing that active liquidity generates the yield
// not also liquidity that was deposited after the yield
// was produced.
#[test]
fn yield_collect_sequence() {
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

    let receiver = env.register_contract(None, FlashLoanReceiver);
    let receiver_client = FlashLoanReceiverClient::new(&env, &receiver);

    // Initialize the flash loan receiver contract.
    receiver_client.init(&user1, &token_id, &pool_addr);
    pool_client.initialize(&token_id);

    token_admin.mint(&receiver, &(1000 * STROOP as i128));
    token_admin.mint(&user1, &(100 * STROOP as i128));
    token_admin.mint(&user2, &(400 * STROOP as i128));

    // user 1 and 2 deposit into the pool.
    pool_client.deposit(&user1, &(100 * STROOP as i128));
    pool_client.deposit(&user2, &(300 * STROOP as i128));

    assert_eq!(token.balance(&user1), 0);
    assert_eq!(token.balance(&user2), 100 * STROOP as i128);

    // Flash loan borrow occurs.
    // It generates yield which is held in the pool.
    pool_client.borrow(&receiver, &(400 * STROOP as i128));

    let expected_yield = 3_200_000;

    // user1 should receive 1/4 of the total yield since it owns
    // 1/4 of the liquidity.
    let expected_user1_yield = expected_yield / 4;

    // user2 should receive 3/4 of the total yield since it owns
    // 3/4 of the liquidity.
    let expected_user2_yield = (expected_yield / 4) * 3;

    // Update fees and collect matured rewards for users 1 and 2
    pool_client.update_fee_rewards(&user1);
    pool_client.update_fee_rewards(&user2);
    pool_client.withdraw_matured(&user2);
    pool_client.withdraw_matured(&user1);

    assert_eq!(token.balance(&user1), expected_user1_yield);
    assert_eq!(
        token.balance(&user2),
        (100 * STROOP as i128) + expected_user2_yield // user2 still has 100 TOKEN
                                                      // in balance.
    );

    // Flash loan borrow occurs.
    // It generates yield which is held in the pool.
    pool_client.borrow(&receiver, &(400 * STROOP as i128));

    // Since both users didn't collect their fees their balances
    // should remain the same as before.
    assert_eq!(token.balance(&user1), expected_user1_yield);
    assert_eq!(
        token.balance(&user2),
        (100 * STROOP as i128) + expected_user2_yield // user2 still has 100 TOKEN
                                                      // in balance.
    );

    // User2 deposits 100 TOKEN into the pool.
    pool_client.deposit(&user2, &(100 * STROOP as i128));

    // Update fees and collect matured rewards for user2
    pool_client.update_fee_rewards(&user2);
    pool_client.withdraw_matured(&user2);

    // Update fees and collect matured rewards for user1
    pool_client.update_fee_rewards(&user1);
    pool_client.withdraw_matured(&user1);

    // Since the latest deposit occured after the flash loan borrow
    // i.e that liquidity didn't contribute to the borrow,
    // we expect the expected yield to be the same as the borrow at
    // line 160.
    assert_eq!(
        token.balance(&user1),
        expected_user1_yield * 2 // multiplied by 2 since it's the second
                                 // yield that user1 collected.
    );
    assert_eq!(
        token.balance(&user2),
        expected_user2_yield * 2 // user2 doesn't have the 100 TOKEN
                                 // in balance anymore.
    );
}

#[contract]
pub struct FlashLoanReceiver;

fn compute_fee(amount: &i128) -> i128 {
    amount / 1250 // 0.08%, still TBD
}
extern crate std;
#[contractimpl]
impl FlashLoanReceiver {
    pub fn init(env: Env, admin: Address, token: Address, fl_addr: Address) {
        admin.require_auth();
        env.storage().instance().set(&symbol_short!("T"), &token);
        env.storage().instance().set(&symbol_short!("FL"), &fl_addr);
    }

    pub fn exec_op(env: Env) {
        let token_client = token::Client::new(
            &env,
            &env.storage()
                .instance()
                .get::<Symbol, Address>(&symbol_short!("T"))
                .unwrap(),
        );

        let total_amount = (400 * STROOP as i128) + compute_fee(&(400 * STROOP as i128)); // For simlicity we allow much more than we need sometimes.
                                                                                          // This should not be applied for production flash loans.

        token_client.approve(
            &env.current_contract_address(),
            &env.storage()
                .instance()
                .get::<Symbol, Address>(&symbol_short!("FL"))
                .unwrap(),
            &total_amount,
            &(env.ledger().sequence() + 1),
        );
    }
}
