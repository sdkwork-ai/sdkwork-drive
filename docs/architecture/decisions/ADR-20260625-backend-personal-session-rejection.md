# ADR-20260625: Reject Personal Sessions On Backend Admin Surfaces

Status: accepted
Owner: SDKWork maintainers
Updated: 2026-06-25
Specs: SECURITY_SPEC.md §1.1, IAM_SPEC.md, WEB_FRAMEWORK_SPEC.md

## Context

Drive exposes tenant administration through `backend-api` and `admin-storage-api`. IAM allows users with organization membership to operate in a personal (`login_scope = TENANT`) session. SECURITY_SPEC requires backend APIs to reject personal sessions even when the principal still carries admin permissions.

## Decision

1. `DriveBackendAuthorizationPolicy` and `DriveAdminStorageAuthorizationPolicy` reject `login_scope = TENANT` before permission checks.
2. Both surfaces wire `EnforcePrincipalTenantIsolationPolicy` from `sdkwork-web-core`.
3. IAM auth guard tests use `ORGANIZATION` sessions for allowed admin traffic and assert `403` for personal sessions.

## Consequences

- Tenant admins must select an organization session before using storage admin APIs or backend maintenance APIs.
- Personal app sessions remain valid on `app-api` for end-user file workflows.
- Gateway/admin routes classified as `gateway-api` still enforce personal-session rejection in Drive authorization policies.

## Verification

- `cargo test -p sdkwork-routes-drive-backend-api --test iam_auth_guard`
- `cargo test -p sdkwork-routes-storage-backend-api --test iam_auth_guard`
