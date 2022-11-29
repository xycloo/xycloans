#![no_std]
use core::convert;

use soroban_auth::{Identifier, Signature};
use soroban_sdk::{
    contracterror, contractimpl, contracttype, AccountId, Address, BigInt, BytesN, Env,
};

mod token {
    soroban_sdk::contractimport!(file = "./soroban_token_spec.wasm");
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    GenericErr = 1,
    ContractAlreadyInitialized = 2,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    TokenId,
}

pub struct FlashLoansContract;

pub trait FlashLoansContractTrait {
    fn init(e: Env, token_id: BytesN<32>) -> Result<(), Error>;

    fn borrow(e: Env, sig: Signature, amount: BigInt) -> Result<(), Error>;
}

/* This one would have been fancy to show/explain but feels kinda complex and quite an anti-pattern (wrapping the AccountId type)
#[contracttype]
pub struct AccountWrapper(pub AccountId);

impl convert::TryFrom<Address> for AccountWrapper {
    type Error = Error;

    fn try_from(value: Address) -> Result<Self, Self::Error> {
        match value {
            Address::Account(id) => Ok(Self(id)),
            _ => Err(Error::InvalidInvoker),
        }
    }
}
 */

fn to_account(address: Address) -> Result<AccountId, Error> {
    match address {
        Address::Account(id) => Ok(id),
        _ => Err(Error::GenericErr),
    }
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

    fn borrow(e: Env, sig: Signature, amount: BigInt) -> Result<(), Error> {
        let key = DataKey::TokenId;

        if !e.data().has(key.clone()) {
            return Err(Error::ContractAlreadyInitialized);
        }

        let token_id: BytesN<32> = e.data().get(key).unwrap().unwrap();
        let client = token::Client::new(&e, token_id);

        Ok(())
    }
}

/*

borrow:
0. calculate user balance (b1)
1. send the money to the user
1.5 calculate new user balance (b2)
2. user executes an action that immediately yields an interest
2.5 calculate new user balance (b3)
3. if b3 - (b2-b1) > 0


*/

mod test;
