// TODO: probably rewrite this whole test
// there are a couple of things that don't add up.
// we should split the test into smaller functionality tests
// and then simulate a whole sequence.

mod pool {
    use soroban_sdk::contractimport;
    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_pool_contract.wasm");
}

use fixed_point_math::STROOP;
use soroban_sdk::{contractimpl, testutils::Address as _, token, Address, Env, Symbol, contract};

#[test]
fn withdraw_liquidity_position() {
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
    
    receiver_client.init(&user1, &token_id, &pool_addr);
    
    pool_client.initialize(&user1, &token_id); // user1 is the vault's admin

    token_admin.mint(&receiver, &(100 * STROOP as i128));
    token_admin.mint(&user1, &(100 * STROOP as i128));
    token_admin.mint(&user2, &(100 * STROOP as i128));

    pool_client.deposit(&user1, &(50 * STROOP as i128));

    assert_eq!(token.balance(&user1), (50 * STROOP as i128));

    pool_client.deposit(&user2, &(100 * STROOP as i128));
    assert_eq!(token.balance(&user2), 0);

    pool_client.update_fee_rewards(&user2);
    let res = pool_client.try_withdraw_matured(&user2);
    assert!(res.is_err()); // error since no fees have been generated yet
    assert_eq!(token.balance(&user2), 0);

    // the flash loan is used and the receiver contract successfully re-pays the loan + a fee of half a tenth of a stroop
    pool_client.borrow(&receiver, &(100 * STROOP as i128));

    pool_client.update_fee_rewards(&user2);
    pool_client.withdraw_matured(&user2);

    let error = 33;

    assert!(
        2 * (STROOP / 10) as i128 / (2 * 3) - error <= token.balance(&user2)
            && token.balance(&user2) <= 2 * (STROOP / 10) as i128 / (2 * 3) + error
    ); // 2/3 of the fee from the borrow at line 87 => 2/3 of half a tenth of a stroop (0.05% of 100 * 1e7). We can tolerate a small error given by periodic numbers (it should be 333333 but it's actually 333300)

    let previous_balance = token.balance(&pool_addr);
    pool_client.withdraw(&user1, &(25 * STROOP as i128));
    assert_eq!(token.balance(&pool_addr), previous_balance - (25 * STROOP as i128));
    assert_eq!(token.balance(&user1), (75 * STROOP) as i128);

    pool_client.update_fee_rewards(&user1);
    pool_client.withdraw_matured(&user1);

    assert!(
        token.balance(&user1) >= 75 * STROOP as i128 + (STROOP / 10) as i128 / (2 * 3) - error
            && token.balance(&user1)
                <= 75 * STROOP as i128 + (STROOP / 10) as i128 / (2 * 3) + error
    ); // the deposit (75 * 1e7) + 1/3 of the fee from the borrow at line 86 => 1/3 of half a tenth of a stroop (0.05% of 100 * 1e7). We can tolerate a small error given by periodic numbers (it should be 750166666 but it's actually 750166650)

    // the flash loan is used and the receiver contract successfully re-pays the loan + a fee of half a tenth of a stroop
    assert_eq!(token.balance(&pool_addr), (125 * STROOP as i128));
    pool_client.borrow(&receiver, &(100 * STROOP as i128));

    pool_client.update_fee_rewards(&user1);
    pool_client.withdraw_matured(&user1);

    assert!(
        token.balance(&user1)
            >= 75 * STROOP as i128
                + (STROOP / 10) as i128 / (2 * 3)
                + (STROOP / 10) as i128 / (2 * 5)
                - error // ideally we double the error but here it isn't needed since it wasn't calibrated for this op anyways
            && token.balance(&user1)
                <= 75 * STROOP as i128
                    + (STROOP / 10) as i128 / (2 * 3)
                    + (STROOP / 10) as i128 / (2 * 5)
                    + error
    ); // the deposit (75 * 1e7) + 1/3 (50 shares of 100) of the fee from the borrow at line 86 + 1/5 (25 shares of 125) of the fee from the borrow at line 112
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
