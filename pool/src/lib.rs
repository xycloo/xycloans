#![no_std]

use fixed_point_math::FixedPoint;

mod balance;
pub mod contract;
mod events;
mod execution;
pub mod math;
mod rewards;
mod storage;
mod token_utility;
mod types;

pub fn compute_fee(amount: &i128) -> i128 {
    amount.fixed_div_ceil(1250_0000000, 10_000_000).unwrap() // 0.08%, still TBD
}

const INSTANCE_LEDGER_LIFE: u32 = 172_800; // ~10 days. This is very conservative but only for now.
const PERSISTENT_LEDGER_LIFE: u32 = 345_600; // ~20 days.

#[cfg(test)]
mod test;
