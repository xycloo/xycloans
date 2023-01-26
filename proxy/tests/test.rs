#![cfg(test)]
use soroban_auth::{Identifier, Signature};
use soroban_sdk::testutils::{Ledger, LedgerInfo};
use soroban_sdk::{testutils::Accounts, BytesN, Env, IntoVal};

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
    let token_admin = e.accounts().generate();
    let protocol = e.accounts().generate();

    e.ledger().set(LedgerInfo {
        timestamp: 1666359075,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    let lp = e.accounts().generate();
    let lp_id = Identifier::Account(lp.clone());

    let token_id = e.register_contract_wasm(
        &BytesN::from_array(
            &e,
            &[
                78, 52, 121, 202, 209, 66, 106, 25, 193, 181, 10, 91, 46, 213, 58, 244, 217, 115,
                23, 232, 144, 71, 210, 113, 57, 46, 203, 166, 210, 20, 155, 105,
            ],
        ),
        token::WASM,
    );
    let usdc_token = token::Client::new(&e, &token_id);

    usdc_token.initialize(
        &Identifier::Account(token_admin.clone()),
        &7u32,
        &"name".into_val(&e),
        &"symbol".into_val(&e),
    );

    let proxy_contract_id =
        e.register_contract_wasm(&BytesN::from_array(&e, &[1; 32]), proxy::WASM);
    let proxy_client = proxy::Client::new(&e, &proxy_contract_id);
    let proxy_id = Identifier::Contract(proxy_contract_id);

    let vault_contract_id =
        e.register_contract_wasm(&BytesN::from_array(&e, &[8; 32]), vault::WASM);
    let vault_client = vault::Client::new(&e, &vault_contract_id);
    let vault_id = Identifier::Contract(vault_contract_id.clone());

    let flash_loan_contract_id =
        e.register_contract_wasm(&BytesN::from_array(&e, &[5; 32]), loan_ctr::WASM);
    let flash_loan_client = loan_ctr::Client::new(&e, &flash_loan_contract_id);
    let flash_loan_id = Identifier::Contract(flash_loan_contract_id.clone());

    proxy_client.initialize(&Identifier::Account(protocol.clone()));

    flash_loan_client.init(&token_id, &vault_id);
    vault_client.initialize(&proxy_id, &token_id, &flash_loan_contract_id);

    proxy_client.with_source_account(&protocol).set_vault(
        &Signature::Invoker,
        &token_id,
        &vault_contract_id,
    );

    proxy_client.with_source_account(&protocol).set_fl(
        &Signature::Invoker,
        &token_id,
        &flash_loan_contract_id,
    );

    usdc_token
        .with_source_account(&token_admin)
        .mint(&Signature::Invoker, &0, &lp_id, &1000000);

    usdc_token
        .with_source_account(&lp)
        .incr_allow(&Signature::Invoker, &0, &vault_id, &1000000);

    proxy_client
        .with_source_account(&lp)
        .deposit(&Signature::Invoker, &token_id, &1000000);

    assert_eq!(usdc_token.balance(&vault_id), 0);
    assert_eq!(usdc_token.balance(&flash_loan_id), 1000000);

    let receiver_contract = e.register_contract_wasm(None, receiver_ctr::WASM);
    // These `100 $USDC` below are the profits the receiver contract would make. We simply mint the contract some tokens without performing any cdp or arbitrage trading action since it's beyond the scope of the quickstart.
    usdc_token.with_source_account(&token_admin).mint(
        &Signature::Invoker,
        &0,
        &Identifier::Contract(receiver_contract.clone()),
        &100,
    );

    proxy_client.borrow(&token_id, &100000, &receiver_contract);
}
