#![cfg(test)]

use crate::testutils::{register_test_contract as register_vault, VaultContract};
use crate::token::{self, TokenMetadata};
use rand::{thread_rng, RngCore};
use soroban_auth::{Identifier, Signature};
use soroban_sdk::bigint;
use soroban_sdk::{testutils::Accounts, AccountId, BigInt, BytesN, Env, IntoVal};

fn generate_contract_id() -> [u8; 32] {
    let mut id: [u8; 32] = Default::default();
    thread_rng().fill_bytes(&mut id);
    id
}

fn create_token_contract(e: &Env, admin: &AccountId) -> ([u8; 32], token::Client) {
    let id = e.register_contract_token(None);
    let token = token::Client::new(e, &id);
    // decimals, name, symbol don't matter in tests
    token.init(
        &Identifier::Account(admin.clone()),
        &TokenMetadata {
            name: "USD coin".into_val(e),
            symbol: "USDC".into_val(e),
            decimals: 7,
        },
    );
    (id.into(), token)
}

fn create_vault_contract(
    e: &Env,
    admin: &AccountId,
    token_id: &[u8; 32],
    max_supply: Option<BigInt>,
) -> ([u8; 32], VaultContract) {
    let id = generate_contract_id();
    register_vault(&e, &id);
    let vault = VaultContract::new(e, &id);
    vault.initialize(&Identifier::Account(admin.clone()), token_id, max_supply);
    (id, vault)
}

#[test]
fn test() {
    let e: Env = Default::default();
    let admin1 = e.accounts().generate(); // generating the usdc admin

    let loan_ctr = e.accounts().generate();
    let user1 = e.accounts().generate();
    let user2 = e.accounts().generate();
    let loan_ctr_id = Identifier::Account(loan_ctr.clone());
    let user1_id = Identifier::Account(user1.clone());
    let user2_id = Identifier::Account(user2.clone());

    let (contract1, usdc_token) = create_token_contract(&e, &admin1); // registered and initialized the usdc token contract
    let (contract_vault, vault) =
        create_vault_contract(&e, &loan_ctr, &contract1, Some(bigint!(&e, 10000))); // registered and initialized the vault token contract, with usdc as vault token

    let vault_id = Identifier::Contract(BytesN::from_array(&e, &contract_vault)); // the id of the vault

    // minting 1000 usdc to user2
    usdc_token.with_source_account(&admin1).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &user1_id,
        &BigInt::from_u32(&e, 1000),
    );

    usdc_token.with_source_account(&admin1).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &user2_id,
        &BigInt::from_u32(&e, 1000),
    );

    // user1 buys shares from the vault
    vault.deposit(&loan_ctr, user1_id.clone(), BigInt::from_i32(&e, 5));

    // user 1 deposits 5 usdc into vault
    usdc_token.with_source_account(&user1).xfer(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &loan_ctr_id,
        &BigInt::from_u32(&e, 5),
    );

    assert_eq!(usdc_token.balance(&loan_ctr_id), 5);
    assert_eq!(usdc_token.balance(&vault_id), 0);
    assert_eq!(vault.get_shares(&user1_id), 5);

    // user2 buys shares from the vault
    vault.deposit(&loan_ctr, user2_id.clone(), BigInt::from_i32(&e, 8));
    // user 2 deposits 8 usdc into vault
    usdc_token.with_source_account(&user2).xfer(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &loan_ctr_id,
        &BigInt::from_u32(&e, 8),
    );

    assert_eq!(vault.get_shares(&user2_id), 8);

    // fee yields
    usdc_token.with_source_account(&admin1).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &vault_id,
        &BigInt::from_u32(&e, 20),
    );

    // user1 withdraws from the vault
    vault.withdraw(&loan_ctr, user1_id.clone(), BigInt::from_i32(&e, 3));

    assert_eq!(usdc_token.balance(&loan_ctr_id), 13); // user 1 now has 1001 USDC and still has 2 shares in the vault.
    assert_eq!(usdc_token.balance(&vault_id), 20 - 4.875 as u32);
    assert_eq!(vault.get_shares(&user1_id), 2);

    // user1 buys shares from the vault
    vault.deposit(&loan_ctr, user1_id.clone(), BigInt::from_i32(&e, 5));

    // user 1 deposits 5 usdc into vault
    usdc_token.with_source_account(&user1).xfer(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &loan_ctr_id,
        &BigInt::from_u32(&e, 5),
    );
    assert_eq!(vault.get_shares(&user1_id), 2 + 3);
}

/*

lp deposit:
loanctr xfer_from lp to loanctr thorugh l_provide fn => adds amount to lp's deposit
loanctr calls vault's deposit to mint fee shares to lp.

lp withdrawal:
loanctr calls width_fee => lp receives fee money (and shares are burned)
loanctr reads user deposit => xfer to lp pool money
*/
