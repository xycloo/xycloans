use soroban_sdk::{contracterror, contracttype, Address};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    TokenId,
    Admin,
    TotSupply,
    TotalDeposited,
    FlashLoan,
    FlashLoanB,
    Balance(Address),
    FeePerShareUniversal,
    FeePerShareParticular(Address),
    MaturedFeesParticular(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    VaultAlreadyInitialized = 0,
    InvalidAdminAuth = 1,
    InvalidShareBalance = 2, // needs change
    NoFeesMatured = 3,
}
