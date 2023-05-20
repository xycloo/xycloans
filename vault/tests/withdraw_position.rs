mod vault {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_fl_vault.wasm");
}

mod loan_ctr {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_flash_loan.wasm");
}

mod receiver_interface {
    use soroban_sdk::contractimport;

    contractimport!(
        file = "../target/wasm32-unknown-unknown/release/soroban_flash_loan_receiver_standard.wasm"
    );
}

use fixed_point_math::STROOP;
use soroban_sdk::{contractimpl, testutils::Address as _, token, Address, BytesN, Env, Symbol};

use crate::flash_loan_receiver_standard::FlashLoanReceiverClient;

#[test]
fn withdraw_liquidity_position() {
    let e: Env = Default::default();
    e.mock_all_auths();

    let admin1 = Address::random(&e);

    let user1 = Address::random(&e);
    let user2 = Address::random(&e);

    let token_id = e.register_stellar_asset_contract(admin1);
    let token = token::Client::new(&e, &token_id);

    let vault_id = e.register_contract_wasm(&None, vault::WASM);
    let vault_client = vault::Client::new(&e, &vault_id);

    let flash_loan_id = e.register_contract_wasm(&None, loan_ctr::WASM);
    let flash_loan_client = loan_ctr::Client::new(&e, &flash_loan_id);

    let increment_id = e.register_contract(&None, BalIncrement); // 2;32
    let increment_client = BalIncrementClient::new(&e, &increment_id);

    let receiver_id =
        e.register_contract(None, crate::flash_loan_receiver_standard::FlashLoanReceiver);
    let receiver_client = FlashLoanReceiverClient::new(&e, &receiver_id);

    token.mint(&increment_id, &1000000000);
    receiver_client.init(&user1, &token_id, &increment_id, &flash_loan_id);
    increment_client.init(&user1, &token_id);
    //    let flash_loan_client = loan_ctr::Client::new(&e, &flash_loan_contract_id);

    //    flash_loan_client.init(&token_id, &vault_id);
    flash_loan_client.init(&token_id, &vault_id);
    vault_client.initialize(
        &user1,
        &token_id,
        &flash_loan_id,
        &BytesN::from_array(&e, &[0; 32]),
    ); // user1 is the vault's admin

    token.mint(&user1, &(100 * STROOP as i128));
    token.mint(&user2, &(100 * STROOP as i128));

    vault_client.deposit(&user1, &user1, &(50 * STROOP as i128));

    assert_eq!(token.balance(&user1), (50 * STROOP as i128));

    vault_client.deposit(&user1, &user2, &(100 * STROOP as i128));
    assert_eq!(token.balance(&user2), 0);

    vault_client.update_fee_rewards(&user2);
    let res = vault_client.try_withdraw_matured(&user2);
    assert!(res.is_err()); // error since no fees have been generated yet
    assert_eq!(token.balance(&user2), 0);

    // the flash loan is used and the receiver contract successfully re-pays the loan + a fee of half a tenth of a stroop
    assert_eq!(token.balance(&flash_loan_id), (150 * STROOP as i128));
    flash_loan_client.borrow(&receiver_id, &(100 * STROOP as i128));

    vault_client.update_fee_rewards(&user2);
    vault_client.withdraw_matured(&user2);

    let error = 33;

    assert!(
        2 * (STROOP / 10) as i128 / (2 * 3) - error <= token.balance(&user2)
            && token.balance(&user2) <= 2 * (STROOP / 10) as i128 / (2 * 3) + error
    ); // 2/3 of the fee from the borrow at line 87 => 2/3 of half a tenth of a stroop (0.05% of 100 * 1e7). We can tolerate a small error given by periodic numbers (it should be 333333 but it's actually 333300)

    vault_client.withdraw(&user1, &(25 * STROOP as i128));
    assert_eq!(token.balance(&flash_loan_id), (125 * STROOP) as i128);
    assert_eq!(token.balance(&user1), (75 * STROOP) as i128);

    vault_client.withdraw_matured(&user1);

    assert!(
        token.balance(&user1) >= 75 * STROOP as i128 + (STROOP / 10) as i128 / (2 * 3) - error
            && token.balance(&user1)
                <= 75 * STROOP as i128 + (STROOP / 10) as i128 / (2 * 3) + error
    ); // the deposit (75 * 1e7) + 1/3 of the fee from the borrow at line 86 => 1/3 of half a tenth of a stroop (0.05% of 100 * 1e7). We can tolerate a small error given by periodic numbers (it should be 750166666 but it's actually 750166650)

    // the flash loan is used and the receiver contract successfully re-pays the loan + a fee of half a tenth of a stroop
    assert_eq!(token.balance(&flash_loan_id), (125 * STROOP as i128));
    flash_loan_client.borrow(&receiver_id, &(100 * STROOP as i128));

    vault_client.update_fee_rewards(&user1);
    vault_client.withdraw_matured(&user1);

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

mod flash_loan_receiver_standard {
    use super::BalIncrementClient;
    use crate::{receiver_interface, token};
    use fixed_point_math::STROOP;
    use soroban_sdk::{contractimpl, Address, BytesN, Env, Symbol};

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
