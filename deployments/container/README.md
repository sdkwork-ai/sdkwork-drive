# SDKWork Drive Container Images

Build OCI images from repository root:

```bash
docker build -f deployments/docker/Dockerfile.app-api -t registry.sdkwork.com/apps/sdkwork-drive/app-api:local .
docker build -f deployments/docker/Dockerfile.backend-api -t registry.sdkwork.com/apps/sdkwork-drive/backend-api:local .
docker build -f deployments/docker/Dockerfile.open-api -t registry.sdkwork.com/apps/sdkwork-drive/open-api:local .
docker build -f deployments/docker/Dockerfile.admin-storage-api -t registry.sdkwork.com/apps/sdkwork-drive/admin-storage-api:local .
docker build -f deployments/docker/Dockerfile.install-worker -t registry.sdkwork.com/apps/sdkwork-drive/install-worker:local .
```

Release packaging records immutable digests in release evidence. Replace `REPLACE_WITH_RELEASE_DIGEST` placeholders in `deployments/kubernetes/drive-services.yaml` before applying manifests, then run `SDKWORK_DEPLOY_VALIDATION=strict pnpm deploy:validate` to reject any remaining placeholder or tag-only image references.

Mount secrets:

- `sdkwork-drive-database` — Drive PostgreSQL URL and pool settings
- `sdkwork-drive-iam` — IAM database URL and JWT signing material for protected routers

Cloud API pods must set `SDKWORK_DRIVE_DOMAIN_OUTBOX_EMBEDDED_DISPATCH=false` when install-worker is deployed.
