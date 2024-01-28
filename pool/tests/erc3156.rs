/// NB: Only testing "moderc" feature for borrow and not deposits and withdrawals
/// since only the borrow functionality is changed.

use fixed_point_math::STROOP;
mod pool {
    use soroban_sdk::contractimport;
    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_pool.wasm");
}
use soroban_sdk::{
    contract, contractimpl, contracttype, testutils::Address as _, token,  Address, Env
};
#[contracttype]
pub enum DataKey {
    Admin
}

#[contract]
pub struct FlashLoanReceiverModifiedERC3156;

#[contractimpl]
impl FlashLoanReceiverModifiedERC3156 {
    pub fn init(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn exec_op(env: Env, caller: Address, token: Address, amount: i128, fee: i128) {
        // require auth for the flash loan
        caller.require_auth(); // if you want to allow exec_op to be initiated by only a pool you can do so here.

        env.storage().instance().get::<DataKey, Address>(&DataKey::Admin).unwrap().require_auth();
        
        let token_client = token::Client::new(
            &env,
            &token
        );

        // perform operations here
        // ...
        
        let total_amount = amount + fee;
        
        token_client.approve(
            &env.current_contract_address(),
            &caller,
            &total_amount,
            &(env.ledger().sequence() + 1),
        );
    }
}

#[cfg(feature = "moderc3156")]
#[test]
fn collect_yield_amounts_moderc3156() {
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

    let receiver = env.register_contract(None, FlashLoanReceiverModifiedERC3156);
    let receiver_client = FlashLoanReceiverModifiedERC3156Client::new(&env, &receiver);

    // Initialize the flash loan receiver contract.
    receiver_client.init(&user1);
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
        pool_client.borrow_erc(&user1, &receiver, &(100 * STROOP as i128));
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

