#![no_std]

mod balance;
mod contract;
mod events;
mod execution;
pub mod math;
mod rewards;
mod storage;
mod token_utility;
mod types;

pub fn compute_fee(amount: &i128) -> i128 {
    amount / 2000 // 0.05%, still TBD
}
