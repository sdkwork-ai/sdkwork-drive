# ADR-20260625: Sanitize Internal HTTP 500 Responses

Status: accepted
Owner: SDKWork maintainers
Updated: 2026-07-07
Specs: SECURITY_SPEC.md §3, API_SPEC.md

## Context

Several Drive route handlers returned SQL and infrastructure error strings through `internal_problem(format!(...))`, leaking schema and constraint details to API clients.

## Decision

1. `internal_problem` in app-api and backend-api logs the internal detail server-side and returns a stable generic client message.
2. `internal_sql_error` and `DriveServiceError::Internal` mappings follow the same sanitization path.
3. Open-api route errors sanitize SQL and internal service failures consistently.

## Consequences

- Clients receive RFC 9457 problems with numeric `code = 50001` and generic detail text.
- Operators diagnose failures through structured logs, `traceId`, and audit `correlationId` values instead of response bodies.
- Route handlers may still pass internal detail into `internal_problem`; it is never forwarded to clients.

## Verification

- `cargo test -p sdkwork-routes-drive-app-api error::tests::internal_problem_does_not_expose_internal_detail_to_clients`
