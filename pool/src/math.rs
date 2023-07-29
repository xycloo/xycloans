use core::ops::{Add, Sub};

use fixed_point_math::{FixedPoint, STROOP};

pub(crate) fn compute_fee_per_share(
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

pub(crate) fn compute_fee_earned(
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
