use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};
use soroban_sdk::{vec, IntoVal, RawVal, Symbol};

mod token {
    use soroban_sdk::contractimport;
    contractimport!(file = "../soroban_token_spec.wasm");
}

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
fn proxy_admin_auth() {
    let e: Env = Default::default();
    let token_admin = Address::random(&e);
    let protocol = Address::random(&e);
    let lp = Address::random(&e);

    let token_id = e.register_stellar_asset_contract(token_admin.clone());
    let token = token::Client::new(&e, &token_id);
    token.mint(&lp, &10000000);

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
    let expected_auth: Vec<(Address, BytesN<32>, Symbol, soroban_sdk::Vec<RawVal>)> = std::vec![(
        protocol.clone(),
        proxy_contract_id.clone(),
        Symbol::short("set_vault"),
        vec![
            &e,
            protocol.into_val(&e),
            token_id.into_val(&e),
            vault_contract_id.into_val(&e),
        ],
    )];
    assert_eq!(e.recorded_top_authorizations(), expected_auth);

    proxy_client.set_flash_loan(&protocol, &token_id, &flash_loan_contract_id);
    let expected_auth: Vec<(Address, BytesN<32>, Symbol, soroban_sdk::Vec<RawVal>)> = std::vec![(
        protocol.clone(),
        proxy_contract_id,
        Symbol::new(&e, "set_flash_loan"),
        vec![
            &e,
            protocol.into_val(&e),
            token_id.into_val(&e),
            flash_loan_contract_id.into_val(&e),
        ],
    )];
    assert_eq!(e.recorded_top_authorizations(), expected_auth);
}

#[test]
fn proxy_invalid_admin_auth() {
    let e: Env = Default::default();
    let token_admin = Address::random(&e);
    let protocol = Address::random(&e);
    let lp = Address::random(&e);

    let not_protocol = Address::random(&e);

    let token_id = e.register_stellar_asset_contract(token_admin.clone());
    let token = token::Client::new(&e, &token_id);
    token.mint(&lp, &10000000);

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

    let _set_vault_res = proxy_client.try_set_vault(&not_protocol, &token_id, &vault_contract_id);
    assert_eq!(e.recorded_top_authorizations(), []);

    let _set_flash_loan_res =
        proxy_client.try_set_flash_loan(&not_protocol, &token_id, &flash_loan_contract_id);
    assert_eq!(e.recorded_top_authorizations(), []);
}
