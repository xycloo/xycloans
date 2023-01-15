use soroban_sdk::{contracterror, contracttype, BytesN};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Vault(BytesN<32>),
    FlashLoan(BytesN<32>),
}

#[derive(Copy, Clone)]
#[contracterror]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 0,
    NotInitialized = 1,
    NotAdmin = 2,
    VaultDoesntExist = 3,
    FlashLoanDoesntExist = 4,
}
