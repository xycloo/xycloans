use fixed_point_math::STROOP;

mod pool {
    use soroban_sdk::contractimport;
    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_pool_contract.wasm");
}

use soroban_sdk::{testutils::Address as _, token, Address, Env, contract, Symbol, contractimpl};

#[test]
fn fee_withdraw_multiple_users() {
    let e: Env = Default::default();
    e.mock_all_auths();
    e.budget().reset_unlimited();

    let admin1 = Address::random(&e);

    let user1 = Address::random(&e);
    let user2 = Address::random(&e);

    let token_id = e.register_stellar_asset_contract(admin1);
    let token_admin = token::AdminClient::new(&e, &token_id);
    let token = token::Client::new(&e, &token_id);

    let pool_addr = e.register_contract_wasm(&None, pool::WASM);
    let pool_client = pool::Client::new(&e, &pool_addr);

    let receiver = e.register_contract(None, FlashLoanReceiver);
    let receiver_client = FlashLoanReceiverClient::new(&e, &receiver);
    
    // Initialize the flash loan receiver contract. 
    receiver_client.init(&user1, &token_id, &pool_addr);
    pool_client.initialize(&user1, &token_id);

    token_admin.mint(&receiver, &(1000 * STROOP as i128));
    token_admin.mint(&user1, &(50 * STROOP as i128));
    token_admin.mint(&user2, &(100 * STROOP as i128));
    token_admin.mint(&user2, &(150 * STROOP as i128));

    // user 1 and 2 deposit into the pool.
    pool_client.deposit(&user1, &(50 * STROOP as i128));
    pool_client.deposit(&user2, &(100 * STROOP as i128));

    // Flash loan borrow occurs.
    // It generates yield which is held in the pool.
    pool_client.borrow(&receiver, &(100 * STROOP as i128));

    // Update fees and collect matured rewards for users 1 and 2
    pool_client.update_fee_rewards(&user1);
    pool_client.update_fee_rewards(&user2);
    pool_client.withdraw_matured(&user2);
    pool_client.withdraw_matured(&user1);

    assert_eq!(token.balance(&user1), 166650);
    assert_eq!(token.balance(&user2), 333300);

    // Flash loan borrow occurs.
    // It generates yield which is held in the pool.
    pool_client.borrow(&receiver, &(100 * STROOP as i128));

    assert_eq!(token.balance(&user1), 166650);
    assert_eq!(token.balance(&user2), 333300);
    
    // User2 deposits 150 into the pool.
    pool_client.deposit(&user2, &(150 * STROOP as i128));

    // Update fees and collect matured rewards for user2
    pool_client.update_fee_rewards(&user2);
    pool_client.withdraw_matured(&user2);

    // Update fees and collect matured rewards for user1
    pool_client.update_fee_rewards(&user1);
    pool_client.withdraw_matured(&user1);

    // Should receive 1/3 of the deposited fees ~= 3.3 * 1e7 
    // since at the time of the fees deposit user1 held 1/3 of 
    // the total supply.
    assert_eq!(token.balance(&user1), 333300); 

    // Should receive 2/3 of the deposited fees =~ 6.6 * 1e7 
    // since at the time of the fees deposit user1 held 2/3 of 
    // the total supply. The new deposit at line 76 shouldn't 
    // have infuence since the deposited liquidity didn't
    // contribute to the generation of the fees.
    assert_eq!(token.balance(&user2), 333300 * 2);
    
    // At the end of all the fees withdrawals the vault's 
    // balance should be ~= 0. We can tolerate a small error given 
    // by periodic numbers.
    let error: i128 = 100;
    assert_eq!(
        token.balance(&pool_addr), 
        (300 * STROOP as i128) + error
    );

}

#[contract]
pub struct FlashLoanReceiver;

fn compute_fee(amount: &i128) -> i128 {
    amount / 2000 // 0.05%, still TBD
}

#[contractimpl]
impl FlashLoanReceiver {
    pub fn init(e: Env, admin: Address, token: Address, fl_addr: Address) {
        admin.require_auth();
        e.storage().instance().set(&Symbol::short("T"), &token);
        e.storage().instance().set(&Symbol::short("FL"), &fl_addr);
    }

    pub fn exec_op(e: Env) {
        let token_client = token::Client::new(
            &e,
            &e.storage().instance()
                .get::<Symbol, Address>(&Symbol::short("T"))
                .unwrap(),
        );

        let total_amount = (100 * STROOP as i128) + compute_fee(&(100 * STROOP as i128));

        token_client.approve(
            &e.current_contract_address(),
            &e.storage().instance()
                .get::<Symbol, Address>(&Symbol::short("FL"))
                .unwrap(),
            &total_amount,
            &(e.ledger().sequence() + 1)
        );

    }
}
