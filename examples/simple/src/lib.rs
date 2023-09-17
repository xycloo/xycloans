#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, token, Address, Env};

#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum ReceiverError {
    InitFailed = 1,
    NotInitialized = 2,
}

#[doc = "Standard interface for FlashLoan receivers. Implementing `exec_op` is mandatory, but you can also extend the contract for better developer experience, for example, having an `init` function to store all the values instead of hard-coding them could be a good idea."]
pub trait FlashLoanReceiver {
    #[doc = "The method invoked by the FlashLoanLender contract. Here @dev should implement the logic behind how the borrowed amount is going to be used."]
    fn exec_op(env: Env) -> Result<(), ReceiverError>;
}

#[contracttype]
pub enum DataKey {
    Token,
    Amount,
    PoolAddress,
}

#[contract]
pub struct FlashLoanReceiverContract;

fn compute_fee(amount: &i128) -> i128 {
    amount / 1250 // 0.05%, still TBD
}

#[contractimpl]
impl FlashLoanReceiver for FlashLoanReceiverContract {
    fn exec_op(e: Env) -> Result<(), ReceiverError> {
        let token_client = if let Some(token) = &e
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::Token)
        {
            token::Client::new(&e, &token)
        } else {
            return Err(ReceiverError::NotInitialized);
        };

        let borrowed = e
            .storage()
            .instance()
            .get::<DataKey, i128>(&DataKey::Amount)
            .unwrap();

        let total_amount = borrowed + compute_fee(&borrowed);

        let flash_loan = e
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::PoolAddress)
            .unwrap();

        token_client.approve(
            &e.current_contract_address(),
            &flash_loan,
            &total_amount,
            &(e.ledger().sequence() + 1),
        );

        Ok(())
    }
}

#[contractimpl]
impl FlashLoanReceiverContract {
    pub fn init(
        e: Env,
        token_id: Address,
        fl_address: Address,
        amount: i128,
    ) -> Result<(), ReceiverError> {
        e.storage().instance().set(&DataKey::Token, &token_id);
        e.storage()
            .instance()
            .set(&DataKey::PoolAddress, &fl_address);
        e.storage().instance().set(&DataKey::Amount, &amount);

        Ok(())
    }
}

#[cfg(test)]
mod test;
