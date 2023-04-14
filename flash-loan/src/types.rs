use soroban_sdk::{contracterror, contracttype};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    GenericLend = 3,
    LoanNotRepaid = 4,
    NotLP = 5,
    LPNotAContract = 6,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    TokenId,
    LP,
}
