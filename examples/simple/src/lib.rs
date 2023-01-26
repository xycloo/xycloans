#![no_std]
use receiver_interface::{Contract, ReceiverError};
use soroban_auth::{Identifier, Signature};
use soroban_sdk::{contractimpl, BytesN, Env};

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

fn compute_fee(amount: &i128) -> i128 {
    5 * amount / 10000 // 0.05%, still TBD
}

#[contractimpl]
impl receiver_interface::Contract for FlashLoanReceiverContract {
    fn exec_op(e: Env) -> Result<(), ReceiverError> {
        let token_client = token::Client::new(
            &e,
            &BytesN::from_array(
                &e,
                &[
                    78, 52, 121, 202, 209, 66, 106, 25, 193, 181, 10, 91, 46, 213, 58, 244, 217,
                    115, 23, 232, 144, 71, 210, 113, 57, 46, 203, 166, 210, 20, 155, 105,
                ],
            ),
        );

        /*
        Perform all your operations here
        */

        // Re-paying the loan + 0.05% interest
        let total_amount = 100000 + compute_fee(&100000);
        token_client.incr_allow(
            &Signature::Invoker,
            &0,
            &Identifier::Contract(BytesN::from_array(&e, &[5; 32])),
            &total_amount,
        );

        Ok(())
    }
}
