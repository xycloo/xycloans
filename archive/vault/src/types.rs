use soroban_auth::Identifier;
use soroban_sdk::{contracterror, contracttype};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum VaultError {
    Generic = 1,
    NotAdmin = 2,
    SharesExceeded = 3,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    TokenId,
    Admin,
    TotSupply,
    MaxSupply,
    Balance(Identifier),
    Nonce(Identifier),
}
