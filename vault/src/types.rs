use soroban_sdk::{contracterror, contracttype, Address};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    TokenId,
    Admin,
    TotSupply,
    FlashLoan,
    FlashLoanB,
    Balance(Address),
    //  CollectedLastRecorded, // deprecated, should be removed
    FeePerShareUniversal,
    FeePerShareParticular(Address),
    MaturedFeesParticular(Address),
    //InitialDep(Address), // deprecated, should be removed
    //    Batch(BatchKey),     // deprecated, should be removed
    //    Increment(Address), // deprecated, should be removed
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
/*
#[derive(Clone)]
#[contracttype]
pub struct BatchKey(pub Address, pub i128); // depreacated

#[derive(Clone)]
#[contracttype]
// deprecated
pub struct BatchObj {
    pub init_s: i128,
    pub deposit: i128,
    pub curr_s: i128,
}*/
