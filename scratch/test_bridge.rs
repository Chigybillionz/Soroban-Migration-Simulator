use state_engine::StateValue;
use soroban_sdk::xdr::{ScVal, ScMap, ScMapEntry, ScVec, ScSymbol, StringM, BytesM};
use std::collections::BTreeMap;

pub fn state_to_scval(state: &StateValue) -> ScVal {
    match state {
        StateValue::U32(v) => ScVal::U32(*v),
        StateValue::U64(v) => ScVal::U64(*v),
        StateValue::Bool(b) => ScVal::B(*b),
        StateValue::Symbol(s) => ScVal::Sym(ScSymbol(s.as_str().try_into().unwrap())),
        StateValue::Address(_a) => unimplemented!(), // Need a better way
        _ => unimplemented!(),
    }
}
