use core::clone;

use soroban_auth::{Identifier, Signature};
use soroban_sdk::{contractimpl, BigInt, BytesN, Env};

use crate::{
    types::{DataKey, Error},
    utils::{invoke_receiver, try_repay, vault_xfer},
};

pub const STROOP_SCAL: u32 = 1000000;

pub struct FlashLoansContract;

pub trait FlashLoansContractTrait {
    fn init(e: Env, token_id: BytesN<32>) -> Result<(), Error>;

    fn borrow(e: Env, sig: Identifier, amount: BigInt) -> Result<(), Error>;
}

#[contractimpl]
impl FlashLoansContractTrait for FlashLoansContract {
    fn init(e: Env, token_id: BytesN<32>) -> Result<(), Error> {
        let token_key = DataKey::TokenId;
        if e.data().has(token_key.clone()) {
            return Err(Error::ContractAlreadyInitialized);
        }

        e.data().set(token_key, token_id);
        Ok(())
    }

    fn borrow(e: Env, receiver_id: Identifier, amount: BigInt) -> Result<(), Error> {
        // TODO: require that the contract was initialized

        if let Identifier::Contract(receiver_id_bytes) = &receiver_id {
            vault_xfer(&e, &receiver_id, &amount)?;
            invoke_receiver(&e, receiver_id_bytes);
            try_repay(&e, &receiver_id, &amount)?;
            Ok(())
        } else {
            Err(Error::Generic)
        }
    }
}
