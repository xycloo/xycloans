use soroban_auth::Identifier;
use soroban_sdk::{contracterror, contracttype};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Generic = 1,
    ContractAlreadyInitialized = 2,
    GenericLend = 3,
    GenericRepay = 4,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    TokenId,
    Nonce(Identifier),
    LP,
}
