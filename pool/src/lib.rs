#![no_std]

use fixed_point_math::FixedPoint;

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
    amount.fixed_div_floor(*amount, 1250).unwrap() // 0.08%, still TBD
}
