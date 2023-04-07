use crate::contract::SCALAR;

pub fn compute_deposit_ratio(deposit: i128, burned: i128, initial: i128) -> i128 {
    deposit * (burned * SCALAR / initial) / SCALAR
}

pub fn compute_shares_amount(deposited: i128, total_supply: i128, pool_balance: i128) -> i128 {
    (deposited * total_supply) / pool_balance
}

pub fn compute_fee_amount(
    ratio_deposit: i128,
    burned: i128,
    total_supply: i128,
    pool_balance: i128,
) -> i128 {
    ((pool_balance * burned) / total_supply) - ratio_deposit
}
