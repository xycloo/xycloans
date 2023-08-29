use fixed_point_math::STROOP;

mod pool_math {
    use core::ops::{Add, Sub};

    use fixed_point_math::{FixedPoint, STROOP};

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
}

#[test]
fn fee_per_share() {
    let fps = pool_math::compute_fee_per_share(0, 500_000, 100 * STROOP as i128);
    assert_eq!(fps, 5000);

    let fees = pool_math::compute_fee_earned(20 * STROOP as i128, fps, 0);
    assert_eq!(fees, 100_000);
}
