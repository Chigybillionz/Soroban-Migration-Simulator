# Changelog

All notable changes to the Soroban Migration Simulator will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## Unreleased

### Added

- **State Engine** (`state-engine`)
  - `StateValue` enum with all Soroban value types (Address, U32, U64, Bool, Symbol, Bytes, Struct, Map, Vec, Option)
  - `ContractState` with canonical ordering, validation, and SHA-256 fingerprints
  - `StateDiff` engine with `ContractState::diff()` producing added/removed/modified/unchanged entries
  - Nested change detection for Struct (field-level), Vec (index-level), Map (key-level), and Option
  - `ModifiedEntry` with `before`, `after`, and `nested_changes` for structured diff output
  - Summary methods: `added_count`, `removed_count`, `modified_count`, `unchanged_count`, `has_changes`
  - 22 unit tests

- **WASM Analyzer** (`analyzer`)
  - Static analysis of Soroban WASM binaries
  - `contractspecv0` XDR extraction (functions, types, fields)
  - `contractmetav0` and `contractenvmetav0` extraction
  - 8 unit tests

- **WASM Diff** (`wasm-diff`)
  - Interface comparison between two `ContractAnalysis` results
  - Detection of added, removed, and changed functions and types
  - 1 unit test

- **Storage Analyzer** (`storage-analyzer`)
  - Storage model inference from contract interfaces
  - Key/value type detection from `get_*` function patterns
  - Storage diff comparison between V1 and V2
  - 5 unit tests

- **Migration Engine** (`migration-engine`)
  - `MigrationEngine::execute()` — full local Soroban migration pipeline
  - State seeding via `env.as_contract()` with correct storage key encoding
  - Ledger snapshot capture with contract ID filtering
  - `ScVal` ↔ `StateValue` conversion bridge
  - Panic recovery via `catch_unwind` for Soroban host errors
  - `MigrationInput`, `MigrationResult`, `ExecutionInfo` types
  - 10 integration tests (V1→V2, new/deleted entry discovery, failed migration, determinism, input validation, durability, multiple records)

- **Soroban Fixtures**
  - `migration_v1` — V1 contract with `create_record`, `get_record`, `update_record`
  - `migration_v2` — V2 contract with `migrate_record` (adds `version` field)
  - JSON state fixtures (`v1-state.json`, `v2-expected-state.json`)

- **Documentation**
  - Architecture overview (`docs/architecture.md`)
  - Getting started guide (`docs/getting-started.md`)
  - Migration simulation guide (`docs/migration-guide.md`)
  - Project README with capabilities, limitations, and quick start
  - Roadmap (`ROADMAP.md`)
  - Contributing guidelines (`CONTRIBUTING.md`)
  - Code of Conduct (`CODE_OF_CONDUCT.md`)
  - Security policy (`SECURITY.md`)
  - GitHub templates (PR template, bug report, feature request, migration case)

- **CI/CD**
  - GitHub Actions workflow (`ci.yml`) with fmt, check, clippy, and test
