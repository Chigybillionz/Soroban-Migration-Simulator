# Soroban Migration Fixtures

These fixtures provide deterministic V1 and V2 Soroban contracts to test the Migration Simulator's abilities.

## The Fixtures

### `migration_v1`
A simple key-value record storage contract. 
- **Schema**: `Record { owner: Address, value: u64 }`
- **Purpose**: Represents an initial application deployment and acts as the "before" state for simulation.

### `migration_v2`
An upgraded version of the `migration_v1` contract.
- **Schema**: `RecordV2 { owner: Address, value: u64, version: u32 }`
- **Purpose**: Introduces a schema change (the addition of a `version` field) that necessitates a migration.
- **Migration Logic**: Contains a `migrate_record` function that explicitly translates the old V1 record into a V2 record in storage.

## Architecture

```text
V1 Contract
    │
    │ existing state
    ▼
Migration
    │
    │ transformed state
    ▼
V2 Contract
```

## Invariants

When the migration is simulated, the following invariants **must** hold true:
1. `before.owner == after.owner`
2. `before.value == after.value`

## Building and Testing

These contracts are standard Soroban workspaces. You can test them using:

```powershell
cargo test --workspace
```

To build the Wasm binaries:

```powershell
stellar contract build
```

This will produce `migration_v1.wasm` and `migration_v2.wasm` in your `target/wasm32-unknown-unknown/release/` directory.

## Deterministic Data

A JSON file (`migration_scenario.json`) is included to model the exact expected before/after state transition for the simulator state engine.
