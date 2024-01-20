use soroban_sdk::{contracterror, contracttype, Address};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    TokenId,
    TotSupply,
    FeePerShareUniversal,
    Dust,
    Balance(Address),
    FeePerShareParticular(Address),
    MaturedFeesParticular(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 0,
    NotInitialized = 1,
    InvalidShareBalance = 2,
    NoFeesMatured = 3,
    LoanNotRepaid = 4,
    BalanceLtSupply = 5,
    InvalidAmount = 6
}
