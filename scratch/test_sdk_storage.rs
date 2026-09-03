use soroban_sdk::{Env, Address, Val, IntoVal, TryFromVal, xdr::ScVal};

fn main() {
    let env = Env::default();
    let contract_id = Address::generate(&env);
    
    // Create an ScVal
    let sc_val = ScVal::U32(100);
    // Convert to Val
    let val = Val::try_from_val(&env, &sc_val).unwrap();
    
    env.as_contract(&contract_id, || {
        env.storage().persistent().set(&val, &val);
    });
}
