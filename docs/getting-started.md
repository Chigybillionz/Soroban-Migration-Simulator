# Getting Started

A step-by-step guide for developers who want to understand and contribute to the Soroban Migration Simulator.

## Prerequisites

- **Rust** 1.96.0+ (stable toolchain)
- **Cargo** (bundled with Rust)
- **Git**
- **Soroban/Stellar CLI** (only needed for building WASM fixtures):
  ```bash
  cargo install --locked stellar-cli
  ```

## Clone and Setup

```bash
git clone https://github.com/Chigybillionz/Soroban-Migration-Simulator.git
cd Soroban-Migration-Simulator
```

## Workspace Structure

SMS is organized as a Cargo workspace with multiple crates:

| Crate | Purpose |
|---|---|
| `analyzer` | Static WASM binary analysis — extracts contract specs from `contractspecv0` XDR |
| `wasm-diff` | Compares two WASM analyses to detect interface changes |
| `storage-analyzer` | Infers storage models from contract interfaces |
| `state-engine` | State representation (`StateValue`, `ContractState`), fingerprints, and diff engine |
| `migration-engine` | Local Soroban execution — registers WASM, seeds state, invokes migration, captures snapshots |
| `invariant-engine` | Invariant validation (placeholder — not yet implemented) |
| `simulator` | Top-level orchestrator (placeholder — not yet implemented) |
| `report` | Report generation (placeholder — not yet implemented) |
| `cli` | Command-line interface (placeholder — not yet implemented) |

### Fixture Contracts

| Path | Description |
|---|---|
| `contracts/fixtures/migration_v1/` | V1 contract with `create_record`, `get_record`, `update_record` |
| `contracts/fixtures/migration_v2/` | V2 contract with `migrate_record` (adds `version` field) |
| `contracts/fixtures/state/` | JSON state fixtures (`v1-state.json`, `v2-expected-state.json`) |

## Build

```bash
# Build all workspace crates
cargo build --workspace

# Build Soroban WASM fixtures (requires stellar-cli)
cargo build --target wasm32v1-none -p migration_v1 -p migration_v2 --release
```

## Test

```bash
# Run all tests
cargo test --workspace

# Run migration-engine tests specifically
cargo test -p migration-engine

# Run with output
cargo test -p migration-engine -- --nocapture
```

## Quality Checks

Before opening a PR, run:

```bash
cargo fmt --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

All four must pass.

## How the Migration Fixture Works

The V1 → V2 migration demonstrates SMS's core capability:

1. **V1 Contract** (`migration_v1`): Stores `Record { owner: Address, value: u64 }` under `DataKey::Record(owner)`.

2. **V2 Contract** (`migration_v2`): Adds a `migrate_record(owner)` function that reads V1 records and writes V2 records with an additional `version: u32 = 2` field.

3. **Migration Engine Test**:
   - Creates a fresh `soroban_sdk::Env`
   - Registers the V2 WASM
   - Seeds V1 state into contract storage
   - Captures pre-state via `to_ledger_snapshot()`
   - Invokes `migrate_record(owner)`
   - Captures post-state via `to_ledger_snapshot()`
   - Compares pre vs post using `ContractState::diff()`

## Where to Find Key Logic

| What | Where |
|---|---|
| Migration execution | `crates/migration-engine/src/lib.rs` → `MigrationEngine::execute()` |
| Ledger snapshot capture | `crates/migration-engine/src/capture.rs` → `capture_state_from_snapshot()` |
| ScVal ↔ StateValue conversion | `crates/migration-engine/src/conversion.rs` |
| State representation | `crates/state-engine/src/lib.rs` → `StateValue`, `ContractState` |
| State diff engine | `crates/state-engine/src/lib.rs` → `ContractState::diff()`, `StateDiff` |
| WASM analysis | `crates/analyzer/src/lib.rs` → `Analyzer::analyze()` |
| Storage analysis | `crates/storage-analyzer/src/lib.rs` → `StorageAnalyzer::analyze()` |

## How to Create/Modify a Fixture

1. Create a new contract directory under `contracts/fixtures/` with `Cargo.toml` and `src/lib.rs`.
2. Add the contract to the workspace in the root `Cargo.toml` under `[workspace] members`.
3. Use `#[contract]`, `#[contractimpl]`, `#[contracttype]` from `soroban-sdk`.
4. Build with `cargo build --target wasm32v1-none -p <package-name> --release`.
5. Write tests in `crates/migration-engine/src/lib.rs` using `MigrationEngine::execute()`.

## How to Validate Changes Before Opening a PR

```bash
# 1. Format
cargo fmt

# 2. Check compilation
cargo check --workspace

# 3. Lint
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 4. Test
cargo test --workspace

# 5. Verify formatting is clean
cargo fmt --check
```
