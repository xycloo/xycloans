#![no_std]
use receiver_interface::{Contract, ReceiverError};
use soroban_sdk::{contractimpl, token, Address, BytesN, Env, Symbol, symbol_short};

mod receiver_interface {
    soroban_sdk::contractimport!(
        file =
            "../../target/wasm32-unknown-unknown/release/soroban_flash_loan_receiver_standard.wasm"
    );
}

pub struct FlashLoanReceiverContract;
pub struct FlashLoanReceiverContractExt;

fn compute_fee(amount: &i128) -> i128 {
    amount / 2000 // 0.05%, still TBD
}

#[contractimpl]
impl receiver_interface::Contract for FlashLoanReceiverContract {
    fn exec_op(e: Env) -> Result<(), ReceiverError> {
        let token_client = token::Client::new(
            &e,
            &e.storage()
                .get::<Symbol, Address>(&symbol_short!("T"))
                .unwrap()
                .unwrap(),
        );

        /*
        Perform all your operations here
        */

        // Re-paying the loan + 0.08% interest
        let borrowed = e
            .storage()
            .get::<Symbol, i128>(&symbol_short!("A"))
            .unwrap()
            .unwrap();
        let total_amount = borrowed + compute_fee(&borrowed);
        token_client.increase_allowance(
            &e.current_contract_address(),
            &e.storage()
                .get::<Symbol, Address>(&symbol_short!("FL"))
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
        token_id: Address,
        fl_address: Address,
        amount: i128,
    ) -> Result<(), ReceiverError> {
        e.storage().set(&symbol_short!("T"), &token_id);
        e.storage().set(&symbol_short!("FL"), &fl_address);
        e.storage().set(&symbol_short!("A"), &amount);
        Ok(())
    }
}
