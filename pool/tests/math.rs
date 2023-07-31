use fixed_point_math::{FixedPoint, STROOP};

use xycloans_pool::math::{
    compute_fee_earned, 
    compute_fee_per_share
};

#[test]
fn fee_per_share() {
    let fps = compute_fee_per_share(0, 500_000, 100 * STROOP as i128);
    assert_eq!(fps, 5000);

    let fees = compute_fee_earned(20 * STROOP as i128, fps, 0);
    assert_eq!(fees, 100_000);
}
