# Drive Observability Event Dictionary

Status: active
Owner: SDKWork maintainers
Updated: 2026-06-26
Scope: backend observability route events, mandatory/common fields, and event-specific fields.

## 1. Objective

Define a stable event dictionary for `sdkwork-drive` route-level observability, so logs are:

- consistent across admin/app APIs,
- machine-parseable for metrics pipelines,
- enforceable by contract smoke tests.

## 2. Global Standard

- Logger target must be: `sdkwork.drive`
- Every route event must include:
  - `event`
  - `result` (`ok` or `err`)
  - `latency_ms` (non-negative integer)
- Event names are centralized in `sdkwork-drive-observability::events`.

## 3. Event List

### 3.1 Admin API

- `drive.audit_events.list`
- `drive.maintenance.jobs.list`
- `drive.maintenance.object_sweep`
- `drive.maintenance.upload_session_sweep`

### 3.2 App API

- `drive.app.spaces.list`
- `drive.app.spaces.create`
- `drive.app.spaces.get`
- `drive.app.spaces.update`
- `drive.app.spaces.delete`
- `drive.app.upload_sessions.create`
- `drive.app.download_urls.create`
- `drive.app.download_tokens.resolve`

## 4. Common Fields

All events must carry:

- `event`: stable event name
- `result`: `ok` or `err`
- `latency_ms`: elapsed time in milliseconds

## 5. Event-specific Required Fields

### `drive.audit_events.list`

- `filter_has_tenant_id`
- `filter_has_action`
- `filter_has_resource_type`
- `filter_has_resource_id`
- `filter_has_request_id`
- `filter_has_trace_id`
- `page`
- `page_size`
- `total`
- `returned_items`

### `drive.maintenance.jobs.list`

- `filter_has_job_type`
- `filter_has_status`
- `filter_has_operator_id`
- `page`
- `page_size`
- `total`
- `returned_items`

### `drive.maintenance.object_sweep`

- `dry_run`
- `limit`
- `operator_id`
- `has_request_id`
- `has_trace_id`

For `result=ok`, also require:

- `scanned_count`
- `affected_count`

For `result=err`, also require:

- `error_kind`

### `drive.maintenance.upload_session_sweep`

- `now_epoch_ms`
- `dry_run`
- `limit`
- `operator_id`
- `has_request_id`
- `has_trace_id`

For `result=ok`, also require:

- `scanned_count`
- `affected_count`

For `result=err`, also require:

- `error_kind`

### `drive.app.spaces.list`

- `filter_has_owner_subject_type`
- `filter_has_owner_subject_id`
- `returned_items`

### `drive.app.spaces.create`

- `space_type`
- `lifecycle_status`
- `version`

### `drive.app.spaces.get`

- `space_id`
- `space_type`
- `lifecycle_status`
- `version`

### `drive.app.spaces.update`

- `space_id`
- `lifecycle_status`
- `version`

### `drive.app.spaces.delete`

- `space_id`
- `lifecycle_status`
- `version`
- `deleted_node_count`

### `drive.app.upload_sessions.create`

- `state`
- `expires_at_epoch_ms`
- `version`

### `drive.app.download_urls.create`

- `requested_ttl_seconds`
- `expires_at_epoch_ms`
- `method`

### `drive.app.download_tokens.resolve`

- `method`

## 6. Security Notes

- Do not log raw credentials, access tokens, storage secret references, full object keys, or presigned URLs.
- Keep potentially high-cardinality fields controlled and intentional.
- `operator_id` is logged only in admin maintenance events for operational forensics.

## 7. Error Kind Dictionary

When `result=err`, `error_kind` must come from `sdkwork-drive-observability::error_kinds` and must be one of:

- `validation`
- `conflict`
- `not_found`
- `permission_denied`
- `internal`

Free-form `error_kind` values are not allowed. Any new value requires dictionary update plus contract test update in the same change.

## 8. Enforcement

- Event names must be validated by contract smoke test against:
  - `crates/sdkwork-drive-observability/src/lib.rs`
  - `crates/sdkwork-routes-drive-backend-api/src/lib.rs`
  - `crates/sdkwork-routes-drive-app-api/src/lib.rs`
- Route-level log behavior is covered by:
  - `crates/sdkwork-routes-drive-backend-api/tests/observability_routes.rs`
  - `crates/sdkwork-routes-drive-app-api/tests/observability_routes.rs`

