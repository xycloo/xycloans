use soroban_sdk::{BytesN, Env, IntoVal, RawVal, Symbol};

pub fn invoke_receiver(e: &Env, id: &BytesN<32>) {
    e.invoke_contract::<RawVal>(id, &Symbol::short("exec_op"), ().into_val(e));
}
