# Architecture

The Soroban Migration Simulator (SMS) is built around a composable architecture that separates static WASM interface analysis from dynamic state simulation and execution.

## Phase 1: Static WASM Analysis

The first layer of SMS is purely static. It inspects compiled `.wasm` binaries without ever instantiating a Soroban host environment or running contract code.

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

### Why Static Analysis?
We deliberately separate static interface analysis from later state simulation for security and determinism. 
1. **Security**: We do not want to execute arbitrary untrusted contract code merely to inspect its interface. The analyzer operates safely on the binary, extracting the embedded `contractspecv0` XDR.
2. **Determinism**: WASM interface diffing is a deterministic operation that quickly yields insights about function signature changes and schema upgrades before deeper execution pathways are involved.

## Phase 2: Static Storage Analysis

Building upon the `ContractAnalysis` layer, the **Storage Analyzer** attempts to extract persistent storage models. 
Because `contractspecv0` does not inherently expose raw ledger state or storage mechanisms like `env.storage()`, the analyzer employs heuristic pattern matching on public interfaces (e.g. mapping `get_record(owner: Address)` to an `Address` -> `Record` key-value association).

## Important Limitations

- **Interface Specification != Complete Ledger State**: The `contractspecv0` section only describes types used in the public interface of the contract. Internal state models and storage structures might not be fully visible if they aren't exposed through a public contract method.
- **Static analysis cannot enumerate all live Soroban ledger entries**: Static analysis can infer the shapes of storage keys and values, but cannot pull actual user data or iterate over live ledger state.
- **StorageAnalysis describes statically discoverable storage characteristics**: It is **not** a snapshot of blockchain state.
- **WASM Analysis Cannot Prove Migration Safety**: A structural WASM diff merely answers "What changed in the code's interface?". It does not answer "Will my old data still load safely?"
- **Storage Compatibility**: Storage compatibility requires additional deep state simulation and analysis that the WASM interface analyzer alone cannot perform.
- **A Successful WASM Diff Does Not Mean an Upgrade is Safe**: An upgrade might look identical at the interface level but contain catastrophic logic changes. Full validation requires the later invariant simulation layers of SMS.
