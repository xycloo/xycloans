// Auth tests are deprecated

mod pool {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_pool.wasm");
}

mod receiver_interface {
    use soroban_sdk::contractimport;

    contractimport!(
        file = "../target/wasm32-unknown-unknown/release/soroban_flash_loan_receiver_standard.wasm"
    );
}

use soroban_sdk::{contractimpl, token, vec, IntoVal, Symbol, contract};
use soroban_sdk::{testutils::Address as _, Address, Env};
use soroban_sdk::token::StellarAssetAdminInterface;

#[test]
fn vault_admin_auth() {
    let e: Env = Default::default();
    e.mock_all_auths();

    let admin1 = Address::random(&e);

    let user1 = Address::random(&e);
    let user2 = Address::random(&e);

    let token_id = e.register_stellar_asset_contract(admin1);
    let token = token::Client::new(&e, &token_id);

    let pool = e.register_contract_wasm(&None, pool::WASM);
    let pool_client = pool::Client::new(&e, &pool);

//    let increment_id = e.register_contract(&None, BalIncrement); // 2;32
//    let increment_client = BalIncrementClient::new(&e, &increment_id);

    let receiver_contract =
        e.register_contract(None, FlashLoanReceiver);
    let receiver_client = FlashLoanReceiverClient::new(&e, &receiver_contract);

    token.mint(&receiver_contract, &1000000000);

    receiver_client.init(&user1, &token_id, &pool);
    // increment_client.init(&user1, &token_id);

    pool_client.initialize(&user1, &token_id);

    token.mint(&user1, &(10 * STROOP as i128));
    token.mint(&user2, &(10 * STROOP as i128));

    vault_client.deposit(&user1, &(10 * STROOP as i128));
    let expected_auth: Vec<(Address, Address, Symbol, soroban_sdk::Vec<RawVal>)> = std::vec![(
        user1.clone(),
        vault_id.clone(),
        Symbol::short("deposit"),
        vec![&e, user1.into_val(&e), (10 * STROOP as i128).into_val(&e),],
    )];

    assert_eq!(e.auths().get(0).unwrap(), expected_auth.get(0).unwrap());
}


/*
    vault_client.update_fee_rewards(&user1);

    vault_client.withdraw_matured(&user1);
    let expected_auth: Vec<(Address, BytesN<32>, Symbol, soroban_sdk::Vec<RawVal>)> = std::vec![(
        user1.clone(),
        vault_contract_id.clone(),
        Symbol::new(&e, "withdraw_matured"),
        vec![&e, user1.into_val(&e),],
    )];
    assert_eq!(e.recorded_top_authorizations(), expected_auth);

    vault_client.withdraw(&user1, &0);
    let expected_auth: Vec<(Address, BytesN<32>, Symbol, soroban_sdk::Vec<RawVal>)> = std::vec![(
        user1.clone(),
        vault_contract_id,
        Symbol::short("withdraw"),
        vec![&e, user1.into_val(&e), (0_i128).into_val(&e)],
    )];
    assert_eq!(e.recorded_top_authorizations(), expected_auth);
}

#[test]
fn vault_admin_invalid_auth() {
    let e: Env = Default::default();
    let admin1 = Address::random(&e);

    let user1 = Address::random(&e);
    let not_user1 = Address::random(&e);

    let token_id = e.register_stellar_asset_contract(admin1);
    let token = token::Client::new(&e, &token_id);

    let vault_contract_id =
        e.register_contract_wasm(&BytesN::from_array(&e, &[5; 32]), vault::WASM);
    let vault_client = vault::Client::new(&e, &vault_contract_id);
    let vault_id = Address::from_contract_id(&e, &vault_contract_id);

    let flash_loan_contract_id =
        e.register_contract_wasm(&BytesN::from_array(&e, &[23; 32]), loan_ctr::WASM);
    let flash_loan_client = loan_ctr::Client::new(&e, &flash_loan_contract_id);
    let flash_loan_id = Address::from_contract_id(&e, &flash_loan_contract_id);

    let increment_contract = e.register_contract(&BytesN::from_array(&e, &[2; 32]), BalIncrement);
    let increment_contract_id = Address::from_contract_id(&e, &increment_contract);
    let increment_client = BalIncrementClient::new(&e, &increment_contract);

    let receiver_contract =
        e.register_contract(None, crate::flash_loan_receiver_standard::FlashLoanReceiver);
    let receiver_client = FlashLoanReceiverClient::new(&e, &receiver_contract);

    token.mint(&increment_contract_id, &1000000000);

    receiver_client.init(&user1, &token_id);
    increment_client.init(&user1, &token_id);

    flash_loan_client.init(&token_id, &vault_id);
    vault_client.initialize(&user1, &token_id, &flash_loan_id, &flash_loan_contract_id);

    token.mint(&user1, &(10 * STROOP as i128));

    let _res = vault_client.try_deposit(&not_user1, &user1, &(5 * STROOP as i128));
    assert_eq!(e.recorded_top_authorizations(), []);

    let _res = vault_client.try_withdraw_matured(&not_user1);
    assert_eq!(e.recorded_top_authorizations(), []);

    let _res = vault_client.try_withdraw(&not_user1, &0);
    assert_eq!(e.recorded_top_authorizations(), []);
}
*/

/*

use fixed_point_math::STROOP;
mod flash_loan_receiver_standard {
    use super::BalIncrementClient;
    use crate::{receiver_interface, token};
    use fixed_point_math::STROOP;
    use soroban_sdk::{contractimpl, Address, Env, Symbol};

    pub struct FlashLoanReceiver;

    fn compute_fee(amount: &i128) -> i128 {
        amount / 2000 // 0.05%, still TBD
    }

    #[contractimpl]
    impl FlashLoanReceiver {
        pub fn init(e: Env, admin: Address, token: Address, bal_addr: Address, fl_addr: Address) {
            admin.require_auth();
            e.storage().set(&Symbol::short("T"), &token);
            e.storage().set(&Symbol::short("BAL"), &bal_addr);
            e.storage().set(&Symbol::short("FL"), &fl_addr);
        }

        pub fn exec_op(e: Env) -> Result<(), receiver_interface::ReceiverError> {
            let token_client = token::Client::new(
                &e,
                &e.storage()
                    .get::<Symbol, Address>(&Symbol::short("T"))
                    .unwrap()
                    .unwrap(),
            );
            let client = BalIncrementClient::new(
                &e,
                &e.storage()
                    .get::<Symbol, Address>(&Symbol::short("BAL"))
                    .unwrap()
                    .unwrap(),
            );

            token_client.transfer(
                &e.current_contract_address(),
                &e.storage()
                    .get::<Symbol, Address>(&Symbol::short("BAL"))
                    .unwrap()
                    .unwrap(),
                &(100 * STROOP as i128),
            );
            client.increment(&e.current_contract_address(), &(100 * STROOP as i128));

            let total_amount = (100 * STROOP as i128) + compute_fee(&(100 * STROOP as i128));

            token_client.increase_allowance(
                &e.current_contract_address(),
                &e.storage()
                    .get::<Symbol, Address>(&Symbol::short("FL"))
                    .unwrap()
                    .unwrap(),
                &total_amount,
            );

            Ok(())
        }
    }
}

pub struct BalIncrement;

#[contractimpl]
impl BalIncrement {
    pub fn init(e: Env, admin: Address, token: Address) {
        admin.require_auth();
        e.storage().set(&Symbol::short("T"), &token);
    }

    pub fn increment(e: Env, id: Address, amount: i128) {
        let token_client = token::Client::new(
            &e,
            &e.storage()
                .get::<Symbol, Address>(&Symbol::short("T"))
                .unwrap()
                .unwrap(),
        );

        token_client.transfer(
            &e.current_contract_address(),
            &id,
            &(amount + STROOP as i128),
        )
    }

    pub fn decrement(e: Env, id: Address, amount: i128) {
        let token_client = token::Client::new(
            &e,
            &e.storage()
                .get::<Symbol, Address>(&Symbol::short("T"))
                .unwrap()
                .unwrap(),
        );

        token_client.transfer(
            &e.current_contract_address(),
            &id,
            &(amount - STROOP as i128),
        )
    }
}

*/

use fixed_point_math::STROOP;

#[contract]
pub struct FlashLoanReceiver;

#[contractimpl]
impl FlashLoanReceiver {
    pub fn init(e: Env, admin: Address, token: Address, fl_addr: Address) {
        admin.require_auth();
        e.storage().instance().set(&Symbol::short("T"), &token);
        e.storage().instance().set(&Symbol::short("FL"), &fl_addr);
    }

    pub fn exec_op(e: Env) -> Result<(), receiver_interface::ReceiverError> {
        let token_client = token::Client::new(
            &e,
            &e.storage()
                .instance()
                .get::<Symbol, Address>(&Symbol::short("T"))
                .unwrap()
        );
    
        token_client.transfer(
            &e.current_contract_address(),
            &e.storage().instance()
                .get::<Symbol, Address>(&Symbol::short("BAL"))
                .unwrap(),
            &(100 * STROOP as i128),
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

        Ok(())
    }
}
