use soroban_sdk::{Address, Env, IntoVal, RawVal, Symbol};

pub(crate) fn invoke_receiver(e: &Env, id: &Address) {
    e.invoke_contract::<RawVal>(id, &Symbol::short("exec_op"), ().into_val(e));
}

