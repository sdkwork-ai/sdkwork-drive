# Drive Audit Investigation Runbook

Status: active
Owner: SDKWork maintainers
Updated: 2026-07-07

## Purpose

Investigate tenant-scoped security and data-access events emitted by Drive workspace services and correlate them with IAM session context.

## Signals

- Structured logs: `sdkwork.drive` target with server `traceId` and audit `correlationId`
- Metrics: `drive_http_requests_by_route_total`, `drive_http_request_errors_total`
- Database: `dr_drive_audit_event`, `dr_drive_file_sensitive_operation`, `dr_drive_tenant_quota`

## Investigation Steps

1. Collect `traceId` from the API problem response and `correlationId` from the audit event or customer report when available.
2. Search application logs for the trace id and audit correlation id within the incident window.
3. Query audit tables for the tenant id and affected node or share-link ids.
4. Confirm whether the actor used `login_scope = ORGANIZATION` for admin actions.
5. If object egress is involved, verify download grant TTL and share-link access code policy.

## Escalation

- Cross-tenant data exposure: follow tenant isolation incident procedure and freeze affected tenant sessions.
- Provider egress abuse: disable provider binding and rotate storage credentials per [drive-production-operations.md](./drive-production-operations.md).

## Verification After Remediation

- `pnpm verify`
- Staging open-api smoke when secrets are configured
