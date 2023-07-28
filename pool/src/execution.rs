use soroban_sdk::{symbol_short, Address, Env, IntoVal, Symbol};

pub(crate) fn invoke_receiver(e: &Env, id: &Address) {
    e.invoke_contract::<()>(
        id, 
        &symbol_short!("exec_op"), 
        ().into_val(e)
    );
}

