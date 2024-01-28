use core::ops::{Add, Sub};
use fixed_point_math::{FixedPoint, STROOP};

// Alias to return the dust along with the result
pub(crate) type I128WithDust = (i128, i128); // result, dust

pub fn compute_fee_per_share(
    fee_per_share_universal: i128,
    accrued_interest: i128,
    total_supply: i128,
) -> I128WithDust {
    let interest_by_supply = accrued_interest.fixed_div_floor(total_supply, STROOP.into()).unwrap();
    let computed_floored = fee_per_share_universal.add(interest_by_supply);
    let dust = accrued_interest - interest_by_supply.fixed_mul_ceil(total_supply, STROOP.into()).unwrap();

    (computed_floored, dust)
}

pub fn compute_fee_earned(
    user_balance: i128,
    fee_per_share_universal: i128,
    fee_per_share_particular: i128,
) -> i128 {
    user_balance
        .fixed_mul_floor(
            fee_per_share_universal.sub(fee_per_share_particular),
            STROOP.into(),
        )
        .unwrap()
}

#[test]
fn test_dust() {
    compute_fee_per_share(3 * STROOP as i128, 50 * STROOP as i128, 200001 * STROOP as i128);
}
