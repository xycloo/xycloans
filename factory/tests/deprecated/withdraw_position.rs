/*

DEPRECATED TEST


use soroban_sdk::{testutils::Address as _, token, Address, BytesN, Env};

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
fn liquidity_withdrawal() {
    let e: Env = Default::default();
    e.mock_all_auths();

    let token_admin = Address::random(&e);
    let protocol = Address::random(&e);
    let lp = Address::random(&e);

    let token_id = e.register_stellar_asset_contract(token_admin.clone());
    let token = token::Client::new(&e, &token_id);
    token.mint(&lp, &50000000000);

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

    proxy_client.deposit(&lp, &token_id, &40000000000);
    proxy_client.deposit(&lp, &token_id, &10000000000);

    assert_eq!(token.balance(&lp), 0);
    assert_eq!(token.balance(&flash_loan_id), 50000000000);

    proxy_client.withdraw_liquidity(&lp, &token_id, &50000000000);

    assert_eq!(token.balance(&lp), 50000000000);
}
*/
