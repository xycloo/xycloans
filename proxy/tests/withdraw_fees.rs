use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

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
fn fee_withdrawal() {
    let e: Env = Default::default();
    let token_admin = Address::random(&e);
    let protocol = Address::random(&e);
    let lp = Address::random(&e);

    let token_id = e.register_stellar_asset_contract(token_admin);
    let token = token::Client::new(&e, &token_id);
    token.mint(&lp, &40000000000);

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
    proxy_client.set_flash_loan(&protocol, &token_id, &flash_loan_contract_id);

    proxy_client.deposit(&lp, &token_id, &10000000000);

    assert_eq!(token.balance(&flash_loan_id), 10000000000);
    assert_eq!(token.balance(&vault_id), 0);

    let batch_0 = vault_client.get_shares(&lp, &0);
    assert_eq!(batch_0.deposit, 10000000000);
    assert_eq!(batch_0.curr_s, 10000000000);
    assert_eq!(batch_0.init_s, 10000000000);

    proxy_client.withdraw_fee(&lp, &token_id, &0, &100000000);

    let batch_0 = vault_client.get_shares(&lp, &0);
    assert_eq!(batch_0.deposit, 10000000000);
    assert_eq!(batch_0.curr_s, 9900000000);
    assert_eq!(batch_0.init_s, 10000000000);
}
