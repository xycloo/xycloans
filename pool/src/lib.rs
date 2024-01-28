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
mod checks;

pub fn compute_fee(amount: &i128) -> i128 {
    amount.fixed_div_ceil(1250_0000000, 10_000_000).unwrap() // 0.08%, still TBD
}

// These numbers are conservative but considering the SACs numbers (SACs are much more likely to be invoked)
// they seem reasonable.

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_LEDGER_LIFE: u32 = 30 * DAY_IN_LEDGERS; // ~30 days.
pub(crate) const INSTANCE_LEDGER_TTL_THRESHOLD: u32 = INSTANCE_LEDGER_LIFE - DAY_IN_LEDGERS;

pub(crate) const PERSISTENT_LEDGER_LIFE: u32 = 90 * DAY_IN_LEDGERS; // ~90 days.
pub(crate) const PERSISTENT_LEDGER_TTL_THRESHOLD: u32 = PERSISTENT_LEDGER_LIFE - DAY_IN_LEDGERS;

#[cfg(test)]
mod test;
