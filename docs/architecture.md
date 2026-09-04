# Architecture

The Soroban Migration Simulator (SMS) is built around a composable architecture that separates static WASM interface analysis from dynamic state simulation and execution.

## Phase 1: Static WASM Analysis

The first layer of SMS is purely static. It inspects compiled `.wasm` binaries without ever instantiating a Soroban host environment or running contract code.

```
                 ┌─────────────────┐
                 │   Old WASM      │
                 └────────┬────────┘
                          │
                          ▼
                  ┌───────────────┐
                  │ WASM Analyzer │
                  └───────┬───────┘
                          │
                          ▼
                  ContractAnalysis
                          │
                          ▼
                  ┌───────────────┐
                  │    Storage    │
                  │    Analyzer   │
                  └───────┬───────┘
                          │
                          ▼
                   StorageAnalysis


                 ┌─────────────────┐
                 │   New WASM      │
                 └────────┬────────┘
                          │
                          ▼
                  ┌───────────────┐
                  │ WASM Analyzer │
                  └───────┬───────┘
                          │
                          ▼
                  ContractAnalysis
                          │
                          ▼
                  ┌───────────────┐
                  │    Storage    │
                  │    Analyzer   │
                  └───────┬───────┘
                          │
                          ▼
                   StorageAnalysis

                          │
                          ▼
                    StorageDiff
```

### Why Static Analysis?
We deliberately separate static interface analysis from later state simulation for security and determinism.
1. **Security**: We do not want to execute arbitrary untrusted contract code merely to inspect its interface. The analyzer operates safely on the binary, extracting the embedded `contractspecv0` XDR.
2. **Determinism**: WASM interface diffing is a deterministic operation that quickly yields insights about function signature changes and schema upgrades before deeper execution pathways are involved.

## Phase 2: Static Storage Analysis

Building upon the `ContractAnalysis` layer, the **Storage Analyzer** attempts to extract persistent storage models.
Because `contractspecv0` does not inherently expose raw ledger state or storage mechanisms like `env.storage()`, the analyzer employs heuristic pattern matching on public interfaces (e.g. mapping `get_record(owner: Address)` to an `Address` -> `Record` key-value association).

## Phase 3: Local Soroban Migration Execution and State Capture

The **migration engine** performs actual local Soroban contract execution and captures real ledger state. This is the dynamic simulation layer of SMS.

```
OLD STATE (StateValue)
        │
        ▼
Local Soroban Env
  ├─ env.register(WASM, ())
  ├─ env.as_contract(|| { storage.set(...) })    ← seed state
  ├─ env.to_ledger_snapshot()                     ← capture PRE-STATE
  ├─ env.invoke_contract(migrate_fn, args)        ← execute migration
  └─ env.to_ledger_snapshot()                     ← capture POST-STATE
        │
        ▼
ContractState (pre)
ContractState (post)
```

### What the migration engine does:

1. **Creates** a fresh `soroban_sdk::Env` with mocked authorization (simulation-only).
2. **Registers** the migration WASM contract into the local environment.
3. **Seeds** actual Soroban `ContractData` entries from `StateValue` input, using the correct contract-specific storage key encoding.
4. **Captures PRE-STATE** by inspecting `env.to_ledger_snapshot()` and extracting `LedgerEntryData::ContractData` entries.
5. **Invokes** the actual migration function through the local Soroban host — real contract code executes real storage mutations.
6. **Captures POST-STATE** from the resulting ledger snapshot.
7. **Returns** a `MigrationResult` containing pre/post states, success flag, and timing information.

### What the migration engine does NOT do (yet):

- **Contract executable upgrade simulation**: The current milestone simulates migration execution and state transformation locally; full Soroban executable upgrade lifecycle simulation is a future capability.
- **CLI interface**: The engine is library-only; CLI comes in a later task.
- **Invariant checking**: Post-migration invariant validation is a separate subsystem (future task).
- **State diff engine**: Formal state diffing between pre/post states is a separate subsystem (future task).

## Phase 4: State Diff Engine

The **state diff engine** compares two `ContractState` snapshots and produces a structured, deterministic description of what changed.

```
Pre-State (ContractState)
    │
    ▼
State Diff Engine (ContractState::diff())
    │
    ▼
StateDiff
  ├─ added:      Vec<StateEntry>       (entries only in post)
  ├─ removed:    Vec<StateEntry>       (entries only in pre)
  ├─ modified:   Vec<ModifiedEntry>    (entries in both, value changed)
  └─ unchanged:  Vec<StateEntry>       (entries in both, identical)
        │
        ▼
Future: Invariant Evaluation
```

### Diff semantics:

- **Added**: A logical storage entry exists in post-state but not pre-state.
- **Removed**: A logical storage entry exists in pre-state but not post-state.
- **Modified**: The logical entry exists in both states but its value changed. Each `ModifiedEntry` includes `before`, `after`, and `nested_changes`.
- **Unchanged**: The logical entry exists in both states and its value is identical.

### Key semantics:

Entries are matched by their canonical `(durability, key)` pair. The engine builds BTreeMap lookups from both states and performs set operations. This is deterministic regardless of input ordering.

### Nested value diffing:

For modified entries, the engine provides structured `NestedChange` information:

- **Struct**: Field-level comparison — detects added, removed, and modified fields.
- **Vec**: Index-level comparison — detects added, removed, and modified items.
- **Map**: Key-level comparison — detects added, removed, and modified entries by key.
- **Option**: Detects transition between Some/None.
- **Leaf types** (Address, U32, U64, Bool, Symbol, Bytes): No nested decomposition; the before/after values are the complete description.

### Deterministic ordering:

The output is sorted by canonical `(durability, key)` order within each category. Since `StateValue` derives `Ord` and `BTreeMap` provides ordered iteration, the diff is deterministic regardless of input ordering.

## Pipeline Summary

```
Static Analysis → Storage Analysis → Migration Execution → State Diff → Invariant Validation
     (Phase 1)        (Phase 2)           (Phase 3)          (Phase 4)      (Future)
```

### Separation of concerns:

- **State representation**: `StateValue`, `StateEntry`, `ContractState` (state-engine)
- **State capture**: `capture_state_from_snapshot()` (migration-engine)
- **State comparison**: `ContractState::diff()` → `StateDiff` (state-engine)
- **Invariant validation**: Future subsystem

## Important Limitations

- **Interface Specification != Complete Ledger State**: The `contractspecv0` section only describes types used in the public interface of the contract. Internal state models and storage structures might not be fully visible if they aren't exposed through a public contract method.
- **Static analysis cannot enumerate all live Soroban ledger entries**: Static analysis can infer the shapes of storage keys and values, but cannot pull actual user data or iterate over live ledger state.
- **StorageAnalysis describes statically discoverable storage characteristics**: It is **not** a snapshot of blockchain state.
- **WASM Analysis Cannot Prove Migration Safety**: A structural WASM diff merely answers "What changed in the code's interface?". It does not answer "Will my old data still load safely?"
- **Storage Compatibility**: Storage compatibility requires additional deep state simulation and analysis that the WASM interface analyzer alone cannot perform.
- **A Successful WASM Diff Does Not Mean an Upgrade is Safe**: An upgrade might look identical at the interface level but contain catastrophic logic changes. Full validation requires the later invariant simulation layers of SMS.
- **No network**: SMS operates entirely locally. No RPC, Horizon, testnet, or mainnet interactions.
- **Simulation-only authorization**: `env.mock_all_auths()` is used for local simulation only. Production authorization would require real signatures.
- **State diff nesting boundary**: The diff engine decomposes Struct, Vec, and Map at one level of nesting. Deeply nested structures show before/after values but not fully decomposed nested changes at arbitrary depth.
