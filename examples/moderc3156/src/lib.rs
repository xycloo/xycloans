#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, Env
};

#[contracttype]
pub enum DataKey {
    Admin
}

#[contract]
pub struct FlashLoanReceiverModifiedERC3156;

#[contractimpl]
impl FlashLoanReceiverModifiedERC3156 {
    pub fn init(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn exec_op(env: Env, caller: Address, token: Address, amount: i128, fee: i128) {
        // require auth for the flash loan
        caller.require_auth(); // if you want to allow exec_op to be initiated by only a pool you can do so here.

        env.storage().instance().get::<DataKey, Address>(&DataKey::Admin).unwrap().require_auth();
        
        let token_client = token::Client::new(
            &env,
            &token
        );

        // perform operations here
        // ...
        
        let total_amount = amount + fee;
        
        token_client.approve(
            &env.current_contract_address(),
            &caller,
            &total_amount,
            &(env.ledger().sequence() + 1),
        );
    }
}
