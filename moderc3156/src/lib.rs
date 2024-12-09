#![no_std]

use soroban_sdk::{contractclient, Address, Env};

#[contractclient(name = "FlashLoanClient")]
pub trait ModErc3156 {
    fn exec_op(env: Env, caller: Address, token: Address, amount: i128, fee: i128);
}
