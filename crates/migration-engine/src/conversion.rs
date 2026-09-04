use soroban_sdk::{
    xdr::ScVal, Address, Env, IntoVal, Map, Symbol, TryFromVal, Val, Vec as SorobanVec,
};

use state_engine::StateValue;

use crate::MigrationError;

/// Convert an ScVal (from ledger snapshot) to a StateValue.
pub fn scval_to_state_value(env: &Env, scval: &ScVal) -> Result<StateValue, MigrationError> {
    match scval {
        ScVal::Bool(b) => Ok(StateValue::Bool(*b)),
        ScVal::U32(v) => Ok(StateValue::U32(*v)),
        ScVal::U64(v) => Ok(StateValue::U64(*v)),
        ScVal::I32(v) => Ok(StateValue::U64(*v as u64)),
        ScVal::Address(_addr) => {
            let sdk_addr = Address::try_from_val(
                env,
                &Val::try_from_val(env, scval).map_err(|e| {
                    MigrationError::StateConversionFailure(format!(
                        "Failed to convert ScVal Address to Val: {:?}",
                        e
                    ))
                })?,
            )
            .map_err(|e| {
                MigrationError::StateConversionFailure(format!(
                    "Failed to convert Val to Address: {:?}",
                    e
                ))
            })?;
            Ok(StateValue::Address(format!("{}", sdk_addr.to_string())))
        }
        ScVal::Symbol(s) => Ok(StateValue::Symbol(s.to_utf8_string_lossy().to_string())),
        ScVal::Bytes(b) => Ok(StateValue::Bytes(hex::encode(b.to_vec()))),
        ScVal::Map(Some(sc_map)) => {
            let mut fields = std::collections::BTreeMap::new();
            for entry in sc_map.iter() {
                let field_name = match &entry.key {
                    ScVal::Symbol(s) => s.to_utf8_string_lossy().to_string(),
                    other => format!("{:?}", other),
                };
                let field_val = scval_to_state_value(env, &entry.val)?;
                fields.insert(field_name, field_val);
            }
            Ok(StateValue::Struct(fields))
        }
        ScVal::Vec(Some(sc_vec)) => {
            let mut items = Vec::new();
            for item in sc_vec.iter() {
                items.push(scval_to_state_value(env, item)?);
            }
            Ok(StateValue::Vec(items))
        }
        ScVal::Void => Ok(StateValue::Option(None)),
        other => Err(MigrationError::StateConversionFailure(format!(
            "Unsupported ScVal variant: {:?}",
            other
        ))),
    }
}

/// Build a Soroban storage key from a StateValue.
///
/// For `StateValue::Address`, builds: `Vec[Symbol(key_prefix), Address]`
/// For `StateValue::Struct`, builds: `Vec[Symbol(key_prefix), Map<Symbol, Val>]`
pub fn state_value_to_storage_key(
    env: &Env,
    key_value: &StateValue,
    key_prefix: &str,
) -> Result<SorobanVec<Val>, MigrationError> {
    let variant_symbol = Symbol::new(env, key_prefix);
    let variant_val: Val = variant_symbol.into_val(env);

    match key_value {
        StateValue::Address(addr_str) => {
            let sdk_str = soroban_sdk::String::from_str(env, addr_str);
            let addr = Address::from_string(&sdk_str);
            Ok(soroban_sdk::vec![env, variant_val, addr.into_val(env),])
        }
        StateValue::U32(v) => Ok(soroban_sdk::vec![env, variant_val, v.into_val(env),]),
        StateValue::U64(v) => Ok(soroban_sdk::vec![env, variant_val, v.into_val(env),]),
        StateValue::Symbol(s) => {
            let sym = Symbol::new(env, s);
            Ok(soroban_sdk::vec![env, variant_val, sym.into_val(env),])
        }
        other => Err(MigrationError::StateConversionFailure(format!(
            "Unsupported key type for storage key: {:?}",
            other
        ))),
    }
}

/// Convert a StateValue into a Soroban storage value (Val).
///
/// For `StateValue::Struct`, builds `Map<Symbol, Val>`.
/// For primitives, builds the corresponding Soroban value.
pub fn state_value_to_storage_value(env: &Env, sv: &StateValue) -> Result<Val, MigrationError> {
    match sv {
        StateValue::Bool(b) => Ok(b.into_val(env)),
        StateValue::U32(v) => Ok(v.into_val(env)),
        StateValue::U64(v) => Ok(v.into_val(env)),
        StateValue::Address(addr_str) => {
            let sdk_str = soroban_sdk::String::from_str(env, addr_str);
            let addr = Address::from_string(&sdk_str);
            Ok(addr.into_val(env))
        }
        StateValue::Symbol(s) => {
            let sym = Symbol::new(env, s);
            Ok(sym.into_val(env))
        }
        StateValue::Struct(fields) => {
            let mut map: Map<Val, Val> = Map::new(env);
            for (name, val) in fields {
                let key = Symbol::new(env, name);
                let sv = state_value_to_storage_value(env, val)?;
                map.set(key.into_val(env), sv);
            }
            Ok(map.into_val(env))
        }
        StateValue::Vec(items) => {
            let mut vec: SorobanVec<Val> = soroban_sdk::vec![env];
            for item in items {
                let val = state_value_to_storage_value(env, item)?;
                vec.push_back(val);
            }
            Ok(vec.into_val(env))
        }
        StateValue::Map(pairs) => {
            let mut map: Map<Val, Val> = Map::new(env);
            for (k, v) in pairs {
                let key = state_value_to_storage_value(env, k)?;
                let val = state_value_to_storage_value(env, v)?;
                map.set(key, val);
            }
            Ok(map.into_val(env))
        }
        StateValue::Option(Some(inner)) => state_value_to_storage_value(env, inner),
        StateValue::Option(None) => Ok(Val::from_void().into()),
        StateValue::Bytes(hex_str) => {
            let bytes = hex::decode(hex_str).map_err(|e| {
                MigrationError::StateConversionFailure(format!("Invalid hex bytes: {:?}", e))
            })?;
            let sdk_bytes = soroban_sdk::Bytes::from_slice(env, &bytes);
            Ok(sdk_bytes.into_val(env))
        }
    }
}
