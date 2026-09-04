# Roadmap

## Phase 1 — Foundation ✅

Completed:

- Repository initialization and workspace setup
- Soroban V1/V2 migration fixtures
- Static WASM analyzer (contractspecv0 XDR extraction)
- WASM interface diffing
- Storage analyzer (storage model inference from interfaces)
- State engine (StateValue, ContractState, fingerprints, canonical ordering)

## Phase 2 — Migration Verification ✅

Completed:

- StateValue ↔ ScVal conversion bridge
- Ledger snapshot state capture (ContractData extraction with contract filtering)
- Local Soroban migration execution (MigrationEngine)
- State diff engine (StateDiff with nested change detection)
- V1 → V2 integration test with real contract execution
- Deterministic state fingerprints

## Phase 3 — Migration Verification (Future)

Potential future work:

- Deeper nested state diffing (arbitrary depth)
- Invariant validation engine (balance conservation, ownership rules)
- Migration safety reporting (structured safety analysis)
- Additional migration pattern support
- State diff serialization for CI/CD pipelines

## Phase 4 — Developer Experience (Future)

Potential future work:

- CLI (`soroban-migrate simulate`, `soroban-migrate diff`)
- Structured JSON output
- Human-readable migration reports
- Migration configuration files
- Interactive migration explorer

## Phase 5 — CI Integration (Future)

Potential future work:

- GitHub Action for pull-request migration checks
- Automated invariant evaluation on PRs
- Migration regression detection
- Badge generation for migration safety

## Phase 6 — Advanced Simulation (Future)

Potential future work:

- Production-state fixture acquisition (RPC integration)
- Broader migration patterns (cross-contract, upgrade orchestration)
- Full Soroban upgrade lifecycle simulation
- Multi-contract migration scenarios
- Gas/cost estimation for migrations
