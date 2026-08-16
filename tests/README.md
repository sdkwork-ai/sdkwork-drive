# Drive Tests

`tests/` is reserved for cross-package tests and shared fixtures for
SDKWork Drive. It is the test boundary governed by
`../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md` and
`../sdkwork-specs/TEST_SPEC.md`.

## Allowed Content

- Cross-package integration tests.
- Shared test fixtures and utilities.
- Contract tests that span multiple crates or packages.
- End-to-end test scenarios.

## Forbidden Content

- Unit tests (belong inside the owning crate or package).
- Runtime state, databases, logs, or caches.
- Test artifacts (ignored by `.gitignore`).

## Related Specs

- `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`
- `../sdkwork-specs/TEST_SPEC.md`

## Verification

- `pnpm test` (run repository-wide tests)
