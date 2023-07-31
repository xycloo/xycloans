// Auth tests are not pertinent to contract implementations

/*

use soroban_sdk::{testutils::Address as _, Address, Env};
use soroban_sdk::{vec, IntoVal, RawVal, Symbol};

mod factory {
    use soroban_sdk::contractimport;
    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_factory.wasm");
}

mod flash_loan {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_flash_loan.wasm");
}

mod vault {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/xycloans_fl_vault.wasm");
}

#[test]
fn factory_admin_auth() {
    let e: Env = Default::default();
    e.mock_all_auths();
    let vault_wasm_hash = e.install_contract_wasm(vault::WASM);
    let flash_loan_wasm_hash = e.install_contract_wasm(flash_loan::WASM);

    let token_admin = Address::random(&e);
    let protocol = Address::random(&e);

    let token_id = e.register_stellar_asset_contract(token_admin);

    let factory_id = e.register_contract_wasm(&None, factory::WASM);
    let factory_client = factory::Client::new(&e, &factory_id);

    let vault_id = e.register_contract_wasm(&None, vault::WASM);
    let vault_client = vault::Client::new(&e, &vault_id);

    let flash_loan_id = e.register_contract_wasm(&None, flash_loan::WASM);
    let flash_loan_client = flash_loan::Client::new(&e, &flash_loan_id);

    factory_client.initialize(&protocol, &flash_loan_wasm_hash, &vault_wasm_hash);

    flash_loan_client.init(&token_id, &vault_id);
    vault_client.initialize(&factory_id, &token_id, &flash_loan_id);

    factory_client.set_vault(&token_id, &vault_id);
    let expected_auth: Vec<(Address, Address, Symbol, soroban_sdk::Vec<RawVal>)> = std::vec![(
        protocol.clone(),
        factory_id.clone(),
        Symbol::short("set_vault"),
        vec![&e, token_id.into_val(&e), vault_id.into_val(&e),],
    )];
    assert_eq!(e.auths().get(0).unwrap(), expected_auth.get(0).unwrap());

    factory_client.set_flash_loan(&token_id, &flash_loan_id);
    let expected_auth: Vec<(Address, Address, Symbol, soroban_sdk::Vec<RawVal>)> = std::vec![(
        protocol,
        factory_id,
        Symbol::new(&e, "set_flash_loan"),
        vec![&e, token_id.into_val(&e), flash_loan_id.into_val(&e),],
    )];

    assert_eq!(e.auths().get(0).unwrap(), expected_auth.get(0).unwrap());
}

/*
#[test]
fn factory_invalid_admin_auth() {
    let e: Env = Default::default();
    e.mock_all_auths();

    let token_admin = Address::random(&e);
    let protocol = Address::random(&e);
    let not_protocol = Address::random(&e);
    let lp = Address::random(&e);

    let token_id = e.register_stellar_asset_contract(token_admin.clone());
    let token = token::Client::new(&e, &token_id);
    token.mint(&lp, &10000000);

    let factory_id = e.register_contract_wasm(&None, factory::WASM);
    let factory_client = factory::Client::new(&e, &factory_id);

    let vault_id = e.register_contract_wasm(&None, vault::WASM);
    let vault_client = vault::Client::new(&e, &vault_id);

    let flash_loan_id = e.register_contract_wasm(&None, loan_ctr::WASM);
    let flash_loan_client = loan_ctr::Client::new(&e, &flash_loan_id);

    factory_client.initialize(&protocol);

    flash_loan_client.init(&token_id, &vault_id);
    vault_client.initialize(&factory_id, &token_id, &flash_loan_id);

    let _set_vault_res = factory_client.try_set_vault(&token_id, &vault_id);
    assert_eq!(e.auths(), []);

    let _set_flash_loan_res = factory_client.try_set_flash_loan(&token_id, &flash_loan_id);
    assert_eq!(e.auths(), []);
}
*/

*/