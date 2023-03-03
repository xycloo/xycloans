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
    Batches(Address),
    Increment(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Generic = 1,
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
