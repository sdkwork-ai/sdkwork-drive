# SDKWork Drive Production Operations Runbook

Status: active
Owner: SDKWork maintainers
Updated: 2026-07-08
Specs: OBSERVABILITY_SPEC.md, DEPLOYMENT_SPEC.md, RELEASE_SPEC.md

## 1. Service Health

| Endpoint | Purpose |
| --- | --- |
| `/healthz` | Liveness (process up) |
| `/readyz` or app-api `ready` | Readiness (database reachable) |
| `/metrics` | Prometheus scrape (`text/plain`) |

Key metrics:

- `drive_http_requests_total`, `drive_http_request_errors_total`
- `drive_http_request_duration_seconds` (histogram)
- `drive_http_rate_limited_total`
- `drive_health_status` (1 = serving)
- `drive_domain_outbox_pending_total`, `drive_domain_outbox_delivered_total`

## 2. Rate Limiting

1. **Edge**: Configure `deployments/nginx/drive-edge-rate-limit.conf.example` on the public ingress, or apply the `Ingress` resource in `deployments/kubernetes/drive-services.yaml` (`limit-rps: 120`, burst multiplier `2`).
2. **Cloud application limiter**: Kubernetes API Deployments must set `SDKWORK_DRIVE_RATE_LIMIT_BACKEND=redis`, source `SDKWORK_DRIVE_RATE_LIMIT_REDIS_URL` from the `sdkwork-drive-rate-limit` secret, and set `SDKWORK_DRIVE_RATE_LIMIT_FAIL_CLOSED=true`. This keeps enforcement global across replicas.
3. **Per-surface budgets**: Tune `SDKWORK_DRIVE_APP_API_RATE_LIMIT_*`, `SDKWORK_DRIVE_BACKEND_API_RATE_LIMIT_*`, `SDKWORK_DRIVE_ADMIN_STORAGE_API_RATE_LIMIT_*`, and `SDKWORK_DRIVE_OPEN_API_RATE_LIMIT_*`.
4. **Cloud outbox**: Set `SDKWORK_DRIVE_DOMAIN_OUTBOX_EMBEDDED_DISPATCH=false` on API pods; dispatch via install-worker only.
5. **Alert** when `drive_http_rate_limited_total` spikes alongside 429 responses on share-link paths, or when Redis limiter errors appear with fail-closed denials.

## 3. Trace Correlation

- Responses include server-owned `X-SdkWork-Trace-Id`; problem JSON bodies include numeric `code` and the same server `traceId`.
- Audit/event flows use `correlationId` when a business operation id is needed.
- Propagate W3C `traceparent` on downstream calls from gateways and BFF layers.
- HTTP spans use OpenTelemetry-compatible fields: `otel.name`, `http.request.method`, `http.route` (template), `deployment.profile`, `runtime.profile`.

### OTLP export

When `OTEL_EXPORTER_OTLP_ENDPOINT` or `SDKWORK_DRIVE_OTEL_EXPORTER_OTLP_ENDPOINT` is set, Drive services export spans over OTLP/HTTP (for example to Grafana Tempo or an OpenTelemetry Collector sidecar).

| Variable | Purpose |
| --- | --- |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | Standard OTLP HTTP endpoint (for example `http://otel-collector:4318/v1/traces`) |
| `SDKWORK_DRIVE_OTEL_EXPORTER_OTLP_ENDPOINT` | Drive-specific alias when a shared collector URL must differ per deployment |
| `OTEL_SERVICE_NAME` | Service name resource attribute (defaults to router binary name) |
| `SDKWORK_DRIVE_LOG` | Rust tracing filter (defaults to `info,sdkwork=debug`) |

Live Playwright smoke (optional staging):

```bash
DRIVE_E2E_OPEN_API_BASE_URL=https://drive.example.com \
DRIVE_E2E_SHARE_TOKEN=... \
DRIVE_E2E_SHARE_ACCESS_CODE=... \
pnpm test:e2e
```

Without the env vars, `pnpm test:e2e` skips live checks and passes locally/CI.

PC browser smoke (optional staging):

```bash
DRIVE_E2E_PC_BASE_URL=https://drive.example.com \
DRIVE_E2E_PC_SHARE_TOKEN=e2e-share-token \
pnpm test:e2e
```

Authenticated claim UI (optional; injects `sdkwork-drive-pc-session` before navigation):

```bash
DRIVE_E2E_PC_BASE_URL=https://drive.example.com \
DRIVE_E2E_PC_SESSION_JSON='{"authToken":"...","accessToken":"...","context":{"tenantId":"...","userId":"..."}}' \
DRIVE_E2E_PC_SHARE_TOKEN=e2e-share-token \
pnpm test:e2e
```

Share claim deep links use `/share/{token}` (not `/drive/shared/claim/...`).

## 4. Auth and Security

Production requires JWT HMAC or JWKS (`SDKWORK_DRIVE_JWT_HMAC_SECRET`, `SDKWORK_DRIVE_JWT_HMAC_SECRETS_JSON`, or `SDKWORK_DRIVE_JWT_JWKS_URL`). Startup fails if `SDKWORK_DRIVE_RUNTIME_PROFILE=production` without signing material.

Kubernetes deployment secret mounts:

| Secret | Consumers | Required keys |
| --- | --- | --- |
| `sdkwork-drive-database` | all API/worker Deployments | `SDKWORK_DATABASE_URL` or PostgreSQL field set |
| `sdkwork-drive-iam` | `app-api`, `backend-api`, `standalone-gateway` | `SDKWORK_DRIVE_JWT_HMAC_SECRET` and/or `SDKWORK_DRIVE_JWT_JWKS_URL` |
| `sdkwork-drive-rate-limit` | `app-api`, `backend-api`, `open-api`, `admin-storage-api` | `SDKWORK_DRIVE_RATE_LIMIT_REDIS_URL` |

Share links store hashed tokens and optional hashed extraction codes. Never log raw share tokens or extraction codes.

## 5. Outbox / Watch Channels

Domain outbox dispatch runs on `SDKWORK_DRIVE_DOMAIN_OUTBOX_DISPATCH_INTERVAL_SECONDS` (default 15s). Failed deliveries retry until `attempt_count` reaches 10, then mark `failed`. Investigate `last_error` on `dr_drive_domain_outbox`.

## 6. Upload Content Policy

Set `SDKWORK_DRIVE_UPLOAD_CONTENT_POLICY_MODE` to `enforce` in production Kubernetes manifests. This is MIME/extension policy, not antivirus scanning.

## 7. Release Gate

Before catalog publication:

```bash
pnpm release:package
pnpm release:validate
SDKWORK_RELEASE_VALIDATION=strict pnpm check:release-readiness
pnpm verify
```

`pnpm release:package` is the local/CI-equivalent build and staging path for release evidence. `pnpm release:evidence` may be used only to re-materialize evidence from already-built artifacts; it must not be used to claim missing packages, signatures, checksums, CDN media, or image digests exist.

Optional staging smoke (GitHub Actions `Drive Staging E2E` workflow_dispatch; configure repository secrets `DRIVE_E2E_OPEN_API_BASE_URL`, `DRIVE_E2E_SHARE_TOKEN`, `DRIVE_E2E_SHARE_ACCESS_CODE`, `DRIVE_E2E_PC_BASE_URL`, and optionally `DRIVE_E2E_PC_SESSION_JSON` / `DRIVE_E2E_PC_SHARE_TOKEN`).

## 8. Incident Response Checklist

1. Check `/readyz` and database connectivity.
2. Scrape `/metrics` for error and latency histogram shifts.
3. Verify storage provider bindings and S3 endpoint health.
4. For share-link abuse: tighten edge rate limits and revoke compromised links.
5. Roll back via `pnpm deploy:rollback` workflow or Kubernetes rollout undo.
