#![no_std]
use core::convert;

use soroban_auth::{Identifier, Signature};
use soroban_sdk::{
    contracterror, contractimpl, contracttype, symbol, vec, AccountId, Address, BigInt, BytesN,
    Env, IntoVal, RawVal, Symbol, Vec,
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

    fn borrow(
        e: Env,
        sig: Signature,
        amount: BigInt,
        action: Symbol,
        target_id: BytesN<32>,
        args: Vec<RawVal>,
    ) -> Result<(), Error>;
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

    fn borrow(
        e: Env,
        sig: Signature,
        amount: BigInt,
        action: Symbol,
        target_id: BytesN<32>,
        args: Vec<RawVal>,
    ) -> Result<(), Error> {
        let key = DataKey::TokenId;

        if !e.data().has(key.clone()) {
            return Err(Error::GenericErr);
        }

        let contract_id = Identifier::Contract(e.current_contract());

        let token_id: BytesN<32> = e.data().get(key).unwrap().unwrap();
        let client = token::Client::new(&e, token_id);

        let borrower_id = sig.identifier(&e);
        let b1 = client.balance(&contract_id);

        client.xfer(
            &Signature::Invoker,
            &BigInt::zero(&e),
            &Identifier::Contract(target_id.clone()),
            &amount,
        );
        e.invoke_contract::<RawVal>(&target_id, &action, args);

        let b3 = client.balance(&contract_id);
        let diff = b3 - b1;

        if diff < 0 {
            return Err(Error::GenericErr);
        }

        client.xfer(&Signature::Invoker, &BigInt::zero(&e), &borrower_id, &diff);

        Ok(())
    }
}

/*

borrow:
0. calculate contract balance (b1)
2. contract executes user-action that immediately yields an interest
3. calculate new contract balance (b2)
4. if b2-b1 < 0:
- revoke tx
4. pay b2-b1 to user.

*/

mod test;
