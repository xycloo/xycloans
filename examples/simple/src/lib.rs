#![no_std]
use receiver_interface::{Contract, ReceiverError};
use soroban_sdk::{contractimpl, symbol, Address, BytesN, Env, Symbol};

mod token {
    soroban_sdk::contractimport!(file = "../../soroban_token_spec.wasm");
}

mod receiver_interface {
    soroban_sdk::contractimport!(
        file =
            "../../target/wasm32-unknown-unknown/release/soroban_flash_loan_receiver_standard.wasm"
    );
}

pub struct FlashLoanReceiverContract;
pub struct FlashLoanReceiverContractExt;

fn compute_fee(amount: &i128) -> i128 {
    5 * amount / 10000 // 0.05%, still TBD
}

#[contractimpl]
impl receiver_interface::Contract for FlashLoanReceiverContract {
    fn exec_op(e: Env) -> Result<(), ReceiverError> {
        let token_client = token::Client::new(
            &e,
            &e.storage()
                .get::<Symbol, BytesN<32>>(&symbol!("T"))
                .unwrap()
                .unwrap(),
        );

        /*
        Perform all your operations here
        */

        // Re-paying the loan + 0.05% interest
        let borrowed = e
            .storage()
            .get::<Symbol, i128>(&symbol!("A"))
            .unwrap()
            .unwrap();
        let total_amount = borrowed + compute_fee(&borrowed);
        token_client.incr_allow(
            &e.current_contract_address(),
            &e.storage()
                .get::<Symbol, Address>(&symbol!("FL"))
                .unwrap()
                .unwrap(),
            &total_amount,
        );

        Ok(())
    }
}

#[contractimpl]
impl FlashLoanReceiverContractExt {
    pub fn init(
        e: Env,
        token_id: BytesN<32>,
        fl_address: Address,
        amount: i128,
    ) -> Result<(), ReceiverError> {
        e.storage().set(&symbol!("T"), &token_id);
        e.storage().set(&symbol!("FL"), &fl_address);
        e.storage().set(&symbol!("A"), &amount);
        Ok(())
    }
}
