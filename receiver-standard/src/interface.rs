use crate::types::ReceiverError;
use soroban_sdk::{contractimpl, Env};

#[doc = "Standard interface for FlashLoan receivers. Implementing `exec_op` is mandatory, but you can also extend the contract for better developer experience, for example, having an `init` function to store all the values instead of hard-coding them could be a good idea."]
pub trait FlashLoanReceiverTrait {
    #[doc = "The method invoked by the FlashLoanLender contract. Here @dev should implement the logic behind how the borrowed amount is going to be used."]
    fn exec_op(env: Env) -> Result<(), ReceiverError>;
}

pub struct FlashLoanReceiver;

#[contractimpl]
impl FlashLoanReceiverTrait for FlashLoanReceiver {
    fn exec_op(_env: Env) -> Result<(), ReceiverError> {
        unimplemented!()
    }
}
