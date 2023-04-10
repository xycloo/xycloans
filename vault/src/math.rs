use core::ops::{Add, Sub};

use fixed_point_math::{FixedPoint, STROOP};

pub fn compute_deposit_ratio(deposit: i128, burned: i128, initial: i128) -> i128 {
    deposit
        .fixed_div_floor(
            initial.fixed_div_floor(burned, STROOP.into()).unwrap(),
            STROOP.into(),
        )
        .unwrap()
}

pub fn compute_shares_amount(deposited: i128, total_supply: i128, pool_balance: i128) -> i128 {
    deposited
        .fixed_mul_floor(total_supply, STROOP.into())
        .unwrap()
        .fixed_div_floor(pool_balance, STROOP.into())
        .unwrap()
}

pub fn compute_fee_amount(
    ratio_deposit: i128,
    burned: i128,
    total_supply: i128,
    pool_balance: i128,
) -> i128 {
    pool_balance
        .fixed_mul_floor(burned, STROOP.into())
        .unwrap()
        .fixed_div_floor(total_supply, STROOP.into())
        .unwrap()
        - ratio_deposit
}

pub fn compute_fee_per_share(
    fee_per_share_universal: i128,
    accrued_interest: i128,
    total_supply: i128,
) -> i128 {
    fee_per_share_universal.add(
        accrued_interest
            .fixed_div_floor(total_supply, STROOP.into())
            .unwrap(),
    )
}

pub fn compute_fee_earned(
    user_balance: i128,
    fee_per_share_universal: i128,
    fee_per_share_particular: i128,
) -> i128 {
    user_balance
        .fixed_mul_ceil(
            fee_per_share_universal.sub(fee_per_share_particular),
            STROOP.into(),
        )
        .unwrap()
}
