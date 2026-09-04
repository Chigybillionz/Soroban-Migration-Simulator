# Security Policy

## Reporting Security Issues

If you discover a security vulnerability in SMS, please report it responsibly:

1. **Do not** open a public GitHub issue for security vulnerabilities.
2. **Email** the maintainer directly with details of the vulnerability.
3. **Include** a description of the vulnerability, steps to reproduce, and potential impact.
4. **Allow** reasonable time for the maintainer to respond and address the issue before public disclosure.

## Important Disclaimers

- **SMS is an experimental simulation tool.** It is not production software and should not be relied upon for production decision-making without independent verification.
- **Simulation results depend on inputs.** Results depend on the supplied contract WASM, migration logic, state fixtures, and configuration assumptions.
- **SMS does not guarantee migration safety.** A passing test result from SMS does not constitute an absolute guarantee that a migration will succeed on-chain or that state will be preserved correctly in production.
- **Local simulation only.** SMS runs entirely locally. It does not interact with any production Stellar network, testnet, or RPC endpoint.
- **Simulated authorization.** SMS uses `mock_all_auths()` for local testing. This does not reflect real on-chain authorization behavior.

## Scope

Security issues relevant to this project include:

- Code execution vulnerabilities in the migration engine
- State capture bugs that could produce incorrect diffs
- Incorrect ScVal ↔ StateValue conversion leading to data loss
- Panics or crashes on malformed input (beyond expected error handling)

## Out of Scope

- Security of the underlying `soroban-sdk` or Stellar protocol
- Security of contracts being tested (SMS tests user-supplied code)
- Network security (SMS has no network component)
