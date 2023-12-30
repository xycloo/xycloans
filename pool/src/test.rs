use fixed_point_math::{FixedPoint, STROOP};

use crate::contract::{Pool, PoolClient};

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

    let pool_addr = env.register_contract(&None, Pool);
    let pool_client = PoolClient::new(&env, &pool_addr);

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

#[contract]
pub struct FlashLoanReceiver;

pub fn compute_fee(amount: &i128) -> i128 {
    amount.fixed_div_floor(1250, STROOP as i128).unwrap() // 0.08%, still TBD
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

        let total_amount = (100 * STROOP as i128) + compute_fee(&(100 * STROOP as i128));

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
