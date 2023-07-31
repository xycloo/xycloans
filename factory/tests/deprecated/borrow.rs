/*

DEPRECATED TEST

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};
use soroban_sdk::{token, vec, IntoVal, RawVal, Symbol};

mod loan_ctr {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_flash_loan.wasm");
}

mod vault {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_fl_vault.wasm");
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
fn successful_borrow() {
    let e: Env = Default::default();
    e.mock_all_auths();

    let token_admin = Address::random(&e);
    let protocol = Address::random(&e);

    let lp = Address::random(&e);

    let token_id = e.register_stellar_asset_contract(token_admin.clone());
    let usdc_token = token::Client::new(&e, &token_id);

    let proxy_id = e.register_contract_wasm(&None, proxy::WASM);
    let proxy_client = proxy::Client::new(&e, &proxy_id);

    let vault_id = e.register_contract_wasm(&None, vault::WASM);
    let vault_client = vault::Client::new(&e, &vault_id);

    let flash_loan_id = e.register_contract_wasm(&None, loan_ctr::WASM);
    let flash_loan_client = loan_ctr::Client::new(&e, &flash_loan_id);

    proxy_client.initialize(&protocol);

    flash_loan_client.init(&token_id, &vault_id);
    vault_client.initialize(&proxy_id, &token_id, &flash_loan_id);

    proxy_client.set_vault(&token_id, &vault_id);
    proxy_client.set_flash_loan(&token_id, &flash_loan_id);

    usdc_token.mint(&lp, &1000000);
    proxy_client.deposit(&lp, &token_id, &1000000);

    assert_eq!(usdc_token.balance(&vault_id), 0);
    assert_eq!(usdc_token.balance(&flash_loan_id), 1000000);

    let receiver_contract = e.register_contract_wasm(None, receiver_ctr::WASM);
    let receiver_client = receiver_ctr::Client::new(&e, &receiver_contract);

    receiver_client.init(&token_id, &flash_loan_id, &100000);

    // These `100 $USDC` below are the profits the receiver contract would make. We simply mint the contract some tokens without performing any cdp or arbitrage trading action since it's beyond the scope of the quickstart.
    usdc_token.mint(&receiver_contract, &100);

    proxy_client.borrow(&token_id, &100000, &receiver_contract);
}

#[test]
#[should_panic(expected = "Status(ContractError(4))")]
fn unsuccessful_borrow() {
    let e: Env = Default::default();
    e.mock_all_auths();

    let token_admin = Address::random(&e);
    let protocol = Address::random(&e);

    let lp = Address::random(&e);

    let token_id = e.register_stellar_asset_contract(token_admin.clone());
    let usdc_token = token::Client::new(&e, &token_id);

    let proxy_id = e.register_contract_wasm(&None, proxy::WASM);
    let proxy_client = proxy::Client::new(&e, &proxy_id);

    let vault_id = e.register_contract_wasm(&None, vault::WASM);
    let vault_client = vault::Client::new(&e, &vault_id);

    let flash_loan_id = e.register_contract_wasm(&None, loan_ctr::WASM);
    let flash_loan_client = loan_ctr::Client::new(&e, &flash_loan_id);

    proxy_client.initialize(&protocol);

    flash_loan_client.init(&token_id, &vault_id);
    vault_client.initialize(&proxy_id, &token_id, &flash_loan_id);

    proxy_client.set_vault(&token_id, &vault_id);
    proxy_client.set_flash_loan(&token_id, &flash_loan_id);

    usdc_token.mint(&lp, &1000000);
    proxy_client.deposit(&lp, &token_id, &1000000);

    assert_eq!(usdc_token.balance(&vault_id), 0);
    assert_eq!(usdc_token.balance(&flash_loan_id), 1000000);

    let receiver_contract = e.register_contract_wasm(None, receiver_ctr::WASM);
    let receiver_client = receiver_ctr::Client::new(&e, &receiver_contract);

    receiver_client.init(&token_id, &flash_loan_id, &100000);

    proxy_client.borrow(&token_id, &100000, &receiver_contract);
}

*/
