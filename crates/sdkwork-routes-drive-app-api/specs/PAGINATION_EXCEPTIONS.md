# Pagination Exceptions

## `sandboxEntries.list`

- ID: `EX-2026-0718-DRIVE-SANDBOX-DIRECTORY-PAGE-SIZE`
- Spec: `PAGINATION_SPEC.md` section 3.1.
- Rule: standard `page_size` maximum `200`.
- Owner: `sdkwork-drive`
- Scope: `GET /app/v3/api/drive/sandboxes/{sandboxId}/entries` only.
- Exception: `page_size` supports `1..1000`; all other Drive list operations retain their declared limits.
- Reason: interactive desktop file-system browsing should render most direct-child directories in one request instead of making ordinary files appear absent below an implicit first page.
- Risk: larger response payloads, provider enumeration work, and frontend DOM growth for directories near the limit.
- Expires: `2027-07-18`.
- Controls: a hard 1000-entry page limit, opaque cursor continuation, stable bounded provider windows, frontend request deduplication, automatic continuation, and manual retry after failures.
- Review/removal: reassess after directory-list virtualization and production latency/payload telemetry are available; reduce toward the platform default when evidence shows the smaller limit preserves file-browser completeness and responsiveness.
