use soroban_auth::Identifier;
use soroban_sdk::contracttype;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    TokenId,
    Admin,
    TotSupply,
    FlashLoan,
    InitialDep(Identifier),
    Nonce(Identifier),
    Batch(BatchKey),
    Batches(Identifier),
}

#[derive(Clone)]
#[contracttype]
pub struct BatchKey(pub Identifier, pub u64);

#[derive(Clone)]
#[contracttype]
pub struct BatchObj {
    pub init_s: i128,
    pub deposit: i128,
    pub curr_s: i128,
}
