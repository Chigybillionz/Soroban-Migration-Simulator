# Migration Guide

This document explains how SMS simulates Soroban contract migrations and the distinction between migration simulation and full contract upgrade simulation.

## Migration Simulation Model

SMS currently supports **migration simulation**: executing migration logic locally and inspecting the resulting state changes. This is the core workflow:

```
Old contract/state
       ↓
State preparation (seed ContractData)
       ↓
Migration execution (invoke contract function)
       ↓
Ledger snapshot (to_ledger_snapshot)
       ↓
Post-state capture (extract ContractData)
       ↓
State diff (compare pre vs post)
```

### What This Proves

- The migration function executes without panicking
- Storage mutations happen as expected
- New fields appear, old fields are preserved or removed
- The resulting state matches expectations

### What This Does NOT Prove

- On-chain WASM replacement will succeed
- The full upgrade lifecycle will complete
- Production authorization will pass
- Network-level upgrade mechanics work correctly

## Full Contract Upgrade Simulation

Full Soroban contract upgrade simulation would involve:

1. **WASM hash replacement** — Updating the contract's executable hash on-chain
2. **Constructor invocation** — Running the new contract's constructor
3. **Ledger entry updates** — Modifying the contract instance entry
4. **Upgrade orchestration** — Managing the sequence of on-chain operations

SMS does **not** currently implement this. The current milestone focuses on migration execution and state transformation.

## The Pipeline in Detail

### 1. State Preparation

The migration engine seeds actual Soroban `ContractData` entries into a local test environment:

```rust
let input = MigrationInput {
    contract_id: "my-contract".to_string(),
    wasm: v2_wasm_bytes,
    initial_state: vec![StateEntry {
        durability: Durability::Persistent,
        key: StateValue::Address("GAAAA...".to_string()),
        value: StateValue::Struct(fields),
    }],
    key_prefix: "Record".to_string(),  // matches DataKey::Record variant
    migration_fn: "migrate_record".to_string(),
    migration_args: None,
};
```

The `key_prefix` parameter tells the engine how to construct the Soroban storage key. For a contract with `DataKey::Record(Address)`, the engine builds `Vec[Symbol("Record"), Address]`.

### 2. Migration Execution

The engine registers the V2 WASM contract and invokes the migration function:

```rust
let result = MigrationEngine::execute(&input)?;
```

This uses `env.invoke_contract()` to call the actual contract code. Real Soroban host execution occurs — no faking.

### 3. Ledger Snapshot

Before and after migration, the engine captures the full ledger state:

```rust
let snapshot = env.to_ledger_snapshot();
```

This produces a `LedgerSnapshot` containing all `ContractData` entries, which the engine filters by contract ID, durability, and entry type.

### 4. State Capture

The engine converts `ScVal` (XDR) to `StateValue` (normalized representation):

```
ScVal::Address → StateValue::Address
ScVal::U64     → StateValue::U64
ScVal::Map     → StateValue::Struct
ScVal::Vec     → StateValue::Vec
...
```

### 5. State Diff

The engine compares pre-state and post-state using `ContractState::diff()`:

```rust
let diff = pre_state.diff(&post_state);
// diff.added     — entries only in post
// diff.removed   — entries only in pre
// diff.modified  — entries with changed values (includes nested_changes)
// diff.unchanged — identical entries
```

## Creating a Custom Migration Test

1. **Define your contract** with `#[contract]` and `#[contractimpl]`.
2. **Include a migration function** that reads old state and writes new state.
3. **Build the WASM**: `cargo build --target wasm32v1-none -p your-contract --release`
4. **Write a test** using `MigrationEngine::execute()` with appropriate `MigrationInput`.
5. **Assert on the diff** to verify expected state changes.

## StateValue Types

| Type | Soroban Equivalent | Example |
|---|---|---|
| `Address(String)` | `ScVal::Address` | Account or contract address |
| `U32(u32)` | `ScVal::U32` | Small integers |
| `U64(u64)` | `ScVal::U64` | Large integers, timestamps |
| `Bool(bool)` | `ScVal::Bool` | Flags |
| `Symbol(String)` | `ScVal::Symbol` | Short identifiers |
| `Bytes(String)` | `ScVal::Bytes` | Raw hex-encoded bytes |
| `Struct(BTreeMap)` | `ScVal::Map` | Contract types (Record, etc.) |
| `Map(Vec<(SV,SV)>)` | `ScVal::Map` | Generic key-value maps |
| `Vec(Vec<SV>)` | `ScVal::Vec` | Ordered collections |
| `Option(Option<Box<SV>>)` | `ScVal::Void` / inner | Nullable values |
