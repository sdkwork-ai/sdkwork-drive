# Drive Deployments

`deployments/` contains deployment descriptors and topology examples for
SDKWork Drive. It is the deployment boundary governed by
`../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md` and
`../sdkwork-specs/DEPLOYMENT_SPEC.md`.

## Allowed Content

- `deployments/deploy.yaml` per `SDKWORK_DEPLOY_SPEC.md` (deployctl / SDKWork Deploy Server contract).
- Dockerfile and container build descriptors under `deployments/docker/`.
- Container packaging notes under `deployments/container/`.
- Kubernetes manifests and Helm charts.
- Terraform or infrastructure-as-code examples.
- Deployment topology examples.

## Forbidden Content

- Runtime config templates (those live in `etc/`).
- Secrets, credentials, or environment-specific overrides.
- Runtime state, databases, logs, or caches.

## Related Specs

- `../sdkwork-specs/DEPLOYMENT_SPEC.md`
- `../sdkwork-specs/SDKWORK_DEPLOY_SPEC.md`
- `../sdkwork-specs/APP_RUNTIME_TOPOLOGY_SPEC.md`

## Verification

- `pnpm deploy:validate` (validates `deployments/deploy.yaml` + Kubernetes/systemd descriptors)
- `SDKWORK_DEPLOY_VALIDATION=strict pnpm deploy:validate` (release/GA gate; rejects placeholder or non-immutable Kubernetes image digests)

## Cloud deployment notes

- Replace every `REPLACE_WITH_RELEASE_DIGEST` image reference with the immutable digest from release evidence before applying manifests.
- Strict deployment validation fails until every Kubernetes `image:` reference uses a real `@sha256:<64 hex>` digest.
- API pods should set `SDKWORK_DRIVE_DEPLOYMENT_PROFILE=cloud` and `SDKWORK_DRIVE_RUNTIME_TARGET=server`.
- When `sdkwork-drive-install-worker` is deployed, set `SDKWORK_DRIVE_DOMAIN_OUTBOX_EMBEDDED_DISPATCH=false` on API pods so only the worker runs the periodic outbox loop.
- `install-worker` uses PostgreSQL advisory locks so only one replica runs maintenance tasks at a time; SQLite standalone runs maintenance locally.
- `sdkwork-drive-knowledgebase-events` must contain key `ingress-token`. The install worker mounts
  it as `/run/secrets/sdkwork/knowledgebase-event-ingress-token` and attaches it only to `kbraw:*`
  deliveries. `SDKWORK_DRIVE_KNOWLEDGEBASE_EVENT_CALLBACK_URL` must exactly match the Knowledgebase
  `application.public-ingress` Internal API callback.
- Cloud multi-instance rate limiting uses the Redis-backed application limiter (`SDKWORK_DRIVE_RATE_LIMIT_BACKEND=redis`, `SDKWORK_DRIVE_RATE_LIMIT_FAIL_CLOSED=true`, and `SDKWORK_DRIVE_RATE_LIMIT_REDIS_URL` from the `sdkwork-drive-rate-limit` secret). Ingress `limit-rps` remains the first edge protection layer.
- Use `/readyz` for readiness probes on HTTP services; `/healthz` remains the liveness probe.
