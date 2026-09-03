use soroban_sdk::Env;

fn main() {
    let env = Env::default();
    // try to access ledger snapshot
    let snapshot = env.to_snapshot(); 
    // or env.host().get_ledger_entries()
}
