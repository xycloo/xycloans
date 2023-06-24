use soroban_sdk::{contracterror, contracttype, Address, BytesN};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    FlashLoanHash,
    VaultHash,
    Vault(Address),
    FlashLoan(Address),
}

#[derive(Copy, Clone, Debug)]
#[contracterror]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 0,
    NotInitialized = 1,
    NotAdmin = 2,
    VaultDoesntExist = 3,
    FlashLoanDoesntExist = 4,
}
