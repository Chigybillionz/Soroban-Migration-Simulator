# Contributing to Soroban Migration Simulator

Thank you for your interest in contributing to SMS! This guide will help you get started.

## Before You Start

1. **Read the [README](README.md)** to understand what SMS does and its current capabilities.
2. **Read the [architecture documentation](docs/architecture.md)** to understand the system design.
3. **Check [existing issues](https://github.com/Chigybillionz/Soroban-Migration-Simulator/issues)** to see what's being worked on.
4. **Avoid duplicating active work** — if an issue is assigned, pick a different one.

## Finding Work

Contributors should work through GitHub issues. Issues will contain:

- Description of the task
- Requirements and acceptance criteria
- Implementation guidance
- Testing requirements
- Branching instructions (where applicable)

Look for issues labeled:

- `good first issue` — Great for newcomers
- `help wanted` — The maintainer is looking for community help
- `documentation` — Documentation improvements
- `testing` — Test coverage improvements
- `research` — Investigation tasks

## Assignment Rule

> Contributors should wait until the maintainer assigns/approves the issue before beginning implementation when an issue is marked as requiring assignment.

For `good first issue` and `help wanted` items, you may start immediately after commenting on the issue to express interest.

## Development Workflow

```
Find issue
   ↓
Understand requirements
   ↓
Apply/comment on the issue
   ↓
Wait for assignment (if required)
   ↓
Create branch
   ↓
Implement changes
   ↓
Add/update tests
   ↓
Run validation checks
   ↓
Open PR
   ↓
Review
   ↓
Merge
```

## Branch Naming

Use lowercase and hyphens with a type prefix:

```
feat/<short-description>
fix/<short-description>
docs/<short-description>
test/<short-description>
refactor/<short-description>
research/<short-description>
```

Examples:
- `feat/add-v3-migration-fixture`
- `fix/state-diff-nested-map-comparison`
- `docs/improve-getting-started-guide`
- `test/add-cross-contract-migration-test`

## Commit Guidelines

Use conventional commit prefixes:

```
feat:     New feature
fix:      Bug fix
docs:     Documentation changes
test:     Adding or updating tests
refactor: Code restructuring without behavior change
chore:    Maintenance tasks (CI, dependencies, etc.)
```

Examples:
```
feat: add invariant validation for balance conservation
fix: handle empty ContractData in snapshot capture
docs: add migration guide for custom contracts
test: add nested Vec modification detection test
```

## Pull Request Process

1. **Keep PRs focused** — One logical change per PR.
2. **Update tests** — Add or update tests for your changes.
3. **Update documentation** — If your change affects the public API or user-facing behavior, update the relevant docs.
4. **Run validation** — Ensure all checks pass before opening the PR.
5. **Fill out the PR template** — Include a summary, related issue, and validation checklist.

## Validation Checklist

Before opening a PR, run:

```bash
cargo fmt --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

All four must pass.

## Code Style

- Follow standard Rust conventions (`rustfmt` + `clippy`)
- Use descriptive variable and function names
- Add doc comments for public APIs
- Keep functions focused and reasonably sized
- Prefer explicit error handling over panics in library code

## Testing Conventions

- Unit tests go in `#[cfg(test)] mod tests` within the same file
- Integration tests go in `crates/<crate>/tests/` or use `#[cfg(test)]` modules
- Test names should be descriptive: `test_v1_to_v2_migration_preserves_owner`
- Use `#[should_panic]` for expected panics
- Test both success and failure paths

## GitHub Labels

The following labels are used for issue organization:

| Label | Description |
|---|---|
| `good first issue` | Suitable for new contributors |
| `help wanted` | Community contributions welcome |
| `documentation` | Documentation improvements |
| `research` | Investigation or exploration tasks |
| `testing` | Test coverage improvements |
| `soroban` | Soroban-specific work |
| `state-engine` | State engine changes |
| `migration-engine` | Migration engine changes |
| `wasm-diff` | WASM diffing changes |
| `storage` | Storage analysis changes |
| `invariant-engine` | Invariant engine work |
| `cli` | CLI work |
| `bug` | Bug reports |
| `enhancement` | Feature requests |

## Questions?

If you have questions about contributing, open a [discussion](https://github.com/Chigybillionz/Soroban-Migration-Simulator/discussions) or comment on an existing issue.
