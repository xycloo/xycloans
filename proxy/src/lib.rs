#![no_std]
use soroban_auth::{Identifier, Signature};
use soroban_sdk::{contracterror, contractimpl, contracttype, BytesN, Env};

mod vault {
    use soroban_sdk::contractimport;

    contractimport!(
        file = "../target/wasm32-unknown-unknown/release-with-logs/flash_loan_vault.wasm"
    );
}

mod flash_loan {
    use soroban_sdk::contractimport;

    contractimport!(
        file = "../target/wasm32-unknown-unknown/release/soroban_flash_loans_prototype.wasm"
    );
}

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

pub fn set_admin(env: &Env, admin: Identifier) {
    env.storage().set(DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Result<Identifier, Error> {
    if let Some(Ok(admin_id)) = env.storage().get(DataKey::Admin) {
        Ok(admin_id)
    } else {
        Err(Error::NotInitialized)
    }
}

pub fn has_admin(env: &Env) -> bool {
    env.storage().has(DataKey::Admin)
}

pub fn check_admin(env: &Env, sig: &Signature) -> Result<(), Error> {
    if sig.identifier(env) != get_admin(env)? {
        return Err(Error::NotAdmin);
    }

    Ok(())
}

pub fn set_vault(env: &Env, token_contract_id: BytesN<32>, vault_contract_id: BytesN<32>) {
    let key = DataKey::Vault(token_contract_id);
    env.storage().set(key, vault_contract_id);
}

pub fn get_vault(env: &Env, token_contract_id: BytesN<32>) -> Result<BytesN<32>, Error> {
    let key = DataKey::Vault(token_contract_id);
    if let Some(Ok(vault_contract_id)) = env.storage().get(key) {
        Ok(vault_contract_id)
    } else {
        Err(Error::VaultDoesntExist)
    }
}

pub fn set_flash_loan(
    env: &Env,
    token_contract_id: BytesN<32>,
    flash_loan_contract_id: BytesN<32>,
) {
    let key = DataKey::FlashLoan(token_contract_id);
    env.storage().set(key, flash_loan_contract_id);
}

pub fn get_flash_loan(env: &Env, token_contract_id: BytesN<32>) -> Result<BytesN<32>, Error> {
    let key = DataKey::FlashLoan(token_contract_id);
    if let Some(Ok(flash_loan_contract_id)) = env.storage().get(key) {
        Ok(flash_loan_contract_id)
    } else {
        Err(Error::FlashLoanDoesntExist)
    }
}

pub fn vault_deposit(
    env: &Env,
    provider: Identifier,
    token_contract_id: BytesN<32>,
    amount: i128,
) -> Result<(), Error> {
    let vault_client = vault::Client::new(env, get_vault(env, token_contract_id)?);
    vault_client.deposit(&provider, &amount);

    Ok(())
}

pub fn vault_withdraw_fees(
    env: &Env,
    provider: Identifier,
    token_contract_id: BytesN<32>,
    batch_ts: u64,
    shares: i128,
) -> Result<(), Error> {
    let vault_client = vault::Client::new(env, get_vault(env, token_contract_id)?);
    vault_client.fee_withd(&provider, &batch_ts, &shares);
    Ok(())
}

pub fn flash_loan_borrow(
    env: &Env,
    token_contract_id: BytesN<32>,
    amount: i128,
    receiver_contract_id: BytesN<32>,
) -> Result<(), Error> {
    let receiver_id = Identifier::Contract(receiver_contract_id);
    let flash_loan_client = flash_loan::Client::new(env, get_flash_loan(env, token_contract_id)?);
    flash_loan_client.borrow(&receiver_id, &amount);
    Ok(())
}

pub struct ProxyCommon;
pub struct ProxyLP;
pub struct ProxyBorrow;

pub trait AdminTrait {
    fn initialize(env: Env, admin: Identifier) -> Result<(), Error>;

    fn set_vault(
        env: Env,
        sig: Signature,
        token_contract_id: BytesN<32>,
        vault_contract_id: BytesN<32>,
    ) -> Result<(), Error>;

    fn set_fl(
        env: Env,
        sig: Signature,
        token_contract_id: BytesN<32>,
        flash_loan_contract_id: BytesN<32>,
    ) -> Result<(), Error>;
}

pub trait LPTrait {
    fn deposit(
        env: Env,
        sig: Signature,
        token_contract_id: BytesN<32>,
        amount: i128,
    ) -> Result<(), Error>;

    fn fee_width(
        env: Env,
        sig: Signature,
        token_contract_id: BytesN<32>,
        batch_ts: u64,
        amount: i128,
    ) -> Result<(), Error>;
}

pub trait BorrowTraait {
    fn borrow(
        env: Env,
        token_contract_id: BytesN<32>,
        amount: i128,
        receiver_contract_id: BytesN<32>,
    ) -> Result<(), Error>;
}

#[contractimpl]
impl AdminTrait for ProxyCommon {
    fn initialize(env: Env, admin: Identifier) -> Result<(), Error> {
        if has_admin(&env) {
            return Err(Error::AlreadyInitialized);
        }

        set_admin(&env, admin);
        Ok(())
    }

    fn set_vault(
        env: Env,
        sig: Signature,
        token_contract_id: BytesN<32>,
        vault_contract_id: BytesN<32>,
    ) -> Result<(), Error> {
        check_admin(&env, &sig)?;
        set_vault(&env, token_contract_id, vault_contract_id);
        Ok(())
    }

    fn set_fl(
        env: Env,
        sig: Signature,
        token_contract_id: BytesN<32>,
        flash_loan_contract_id: BytesN<32>,
    ) -> Result<(), Error> {
        check_admin(&env, &sig)?;
        set_flash_loan(&env, token_contract_id, flash_loan_contract_id);
        Ok(())
    }
}

#[contractimpl]
impl LPTrait for ProxyLP {
    fn deposit(
        env: Env,
        sig: Signature,
        token_contract_id: BytesN<32>,
        amount: i128,
    ) -> Result<(), Error> {
        let provider = sig.identifier(&env);
        vault_deposit(&env, provider, token_contract_id, amount)?;
        Ok(())
    }

    fn fee_width(
        env: Env,
        sig: Signature,
        token_contract_id: BytesN<32>,
        batch_ts: u64,
        shares: i128,
    ) -> Result<(), Error> {
        let provider = sig.identifier(&env);
        vault_withdraw_fees(&env, provider, token_contract_id, batch_ts, shares)?;
        Ok(())
    }
}

#[contractimpl]
impl BorrowTraait for ProxyBorrow {
    fn borrow(
        env: Env,
        token_contract_id: BytesN<32>,
        amount: i128,
        receiver_contract_id: BytesN<32>,
    ) -> Result<(), Error> {
        flash_loan_borrow(&env, token_contract_id, amount, receiver_contract_id)?;
        Ok(())
    }
}
