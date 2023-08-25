use soroban_sdk::{contracterror, contracttype, Address};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    ProtocolFees,
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
    AlreadyInitialized = 0,
    NotInitialized = 1,
    InvalidAdminAuth = 2,
    InvalidShareBalance = 3, // needs change
    NoFeesMatured = 4,
    LoanNotRepaid = 5,
}
