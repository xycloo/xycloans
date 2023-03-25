use soroban_sdk::{contracterror, contracttype, Address};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    TokenId,
    Admin,
    TotSupply,
    FlashLoan,
    FlashLoanB,
    InitialDep(Address),
    Batch(BatchKey),
    Increment(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    VaultAlreadyInitialized = 0,
    InvalidAdminAuth = 1,
    InvalidShareBalance = 2,
    BatchDoesntExist = 3,
}

#[derive(Clone)]
#[contracttype]
pub struct BatchKey(pub Address, pub i128);

#[derive(Clone)]
#[contracttype]
pub struct BatchObj {
    pub init_s: i128,
    pub deposit: i128,
    pub curr_s: i128,
}
