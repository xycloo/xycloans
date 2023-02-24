#![cfg(test)]
//use soroban_auth::{Identifier, Signature};
use soroban_sdk::testutils::{Ledger, LedgerInfo};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, IntoVal};

mod token {
    use soroban_sdk::contractimport;
    contractimport!(file = "../soroban_token_spec.wasm");
}

mod loan_ctr {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/flash_loan.wasm");
}

mod vault {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/flash_loan_vault.wasm");
}

mod proxy {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_proxy.wasm");
}

mod receiver_interface {
    use soroban_sdk::contractimport;

    contractimport!(
        file = "../target/wasm32-unknown-unknown/release/soroban_flash_loan_receiver_standard.wasm"
    );
}

mod receiver_ctr {
    use crate::receiver_interface::ReceiverError;
    use soroban_sdk::contractimport;
    contractimport!(file = "../target/wasm32-unknown-unknown/release/simple.wasm");
}

#[test]

fn workflow() {
    let e: Env = Default::default();
    let token_admin = Address::random(&e);
    let protocol = Address::random(&e);

    e.ledger().set(LedgerInfo {
        timestamp: 1666359075,
        protocol_version: 1,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
    });

    let lp = Address::random(&e);
    //    let lp_id = Identifier::Account(lp.clone());

    let token_id = e.register_stellar_asset_contract(token_admin.clone());
    let usdc_token = token::Client::new(&e, &token_id);

    let proxy_contract_id =
        e.register_contract_wasm(&BytesN::from_array(&e, &[1; 32]), proxy::WASM);
    let proxy_client = proxy::Client::new(&e, &proxy_contract_id);
    let proxy_id = Address::from_contract_id(&e, &proxy_contract_id);

    let vault_contract_id =
        e.register_contract_wasm(&BytesN::from_array(&e, &[8; 32]), vault::WASM);
    let vault_client = vault::Client::new(&e, &vault_contract_id);
    let vault_id = Address::from_contract_id(&e, &vault_contract_id);

    let flash_loan_contract_id =
        e.register_contract_wasm(&BytesN::from_array(&e, &[5; 32]), loan_ctr::WASM);
    let flash_loan_client = loan_ctr::Client::new(&e, &flash_loan_contract_id);
    let flash_loan_id = Address::from_contract_id(&e, &flash_loan_contract_id);

    proxy_client.initialize(&protocol);

    flash_loan_client.init(&token_id, &vault_id);
    vault_client.initialize(
        &proxy_id,
        &token_id,
        &flash_loan_id,
        &flash_loan_contract_id,
    );

    proxy_client.set_vault(&protocol, &token_id, &vault_contract_id);

    proxy_client.set_fl(&protocol, &token_id, &flash_loan_contract_id);

    usdc_token.mint(&token_admin, &lp, &1000000);

    usdc_token.incr_allow(&lp, &vault_id, &1000000);

    proxy_client.deposit(&lp, &token_id, &1000000);

    assert_eq!(usdc_token.balance(&vault_id), 0);
    assert_eq!(usdc_token.balance(&flash_loan_id), 1000000);

    let receiver_contract = e.register_contract_wasm(None, receiver_ctr::WASM);
    let receiver_client = receiver_ctr::Client::new(&e, &receiver_contract);

    receiver_client.init(&token_id, &flash_loan_id);

    // These `100 $USDC` below are the profits the receiver contract would make. We simply mint the contract some tokens without performing any cdp or arbitrage trading action since it's beyond the scope of the quickstart.
    usdc_token.mint(
        &token_admin,
        &Address::from_contract_id(&e, &receiver_contract),
        &100,
    );

    proxy_client.borrow(
        &token_id,
        &100000,
        &receiver_contract,
        &Address::from_contract_id(&e, &receiver_contract),
    );
}
