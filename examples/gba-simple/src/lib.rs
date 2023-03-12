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
impl FlashLoanReceiverContractExt {
    pub fn init(e: Env, params: (BytesN<32>, Address, i128)) {
        e.storage().set(&0, &params.0);
        e.storage().set(&1, &params.1);
        e.storage().set(&2, &params.2);
    }
}

#[contractimpl]
impl receiver_interface::Contract for FlashLoanReceiverContract {
    fn exec_op(e: Env) -> Result<(), ReceiverError> {
        let token_id_0: BytesN<32> = e.storage().get(&0).unwrap().unwrap();
        let flash_loan_0: Address = e.storage().get(&1).unwrap().unwrap();
        let amount_0: i128 = e.storage().get(&2).unwrap().unwrap();

        let token_client_0 = token::Client::new(&e, &token_id_0);

        let total_0 = amount_0 + compute_fee(&amount_0);

        // increment the allowance to the flash loan to re-pay the flash loan
        token_client_0.incr_allow(&e.current_contract_address(), &flash_loan_0, &total_0);

        /*
        Perform all your operations here
        */

        Ok(())
    }
}
