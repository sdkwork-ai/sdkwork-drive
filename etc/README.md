# SDKWork Drive Source Configuration

`sdkwork.deployment.config.json` is the source-controlled profile index for
SDKWork Drive. It selects a typed profile from `topology/`; the topology
contract is `../specs/topology.spec.json` and the global authority is
`../sdkwork-specs/SOURCE_CONFIG_SPEC.md`.

The canonical matrix contains `standalone|cloud` crossed with
`development|test|staging|production`.
Standalone development owns the local Drive standalone gateway. Cloud
development starts clients only and consumes the deployed
`platform.api-gateway` surface URL.

Additional safe templates:

- `drive.database.example.toml`: server/runtime database TOML example.
- `sdkwork-api-drive-standalone-gateway.development.toml.example`: standalone gateway development profile.
- `sdkwork-api-drive-standalone-gateway.production.toml.example`: standalone gateway production profile.

Host-local overrides such as `.env.postgres`, `.env.local`, and
`etc/*.local.toml` must stay out of source control. Secrets are injected by the
deployment platform or mounted from ignored secret files; they are never
committed under `etc/`. Installed runtime config is materialized to the paths
governed by `../sdkwork-specs/RUNTIME_DIRECTORY_SPEC.md`.

Validate this authority with:

```powershell
node ../sdkwork-specs/tools/check-source-config-standard.mjs --root .
pnpm check:client-env
pnpm topology:validate
```

<!-- SDKWORK-DEPLOY-LAYOUT: v1 -->
## Installed Runtime Paths

Authority: `APPLICATION_DEPLOY_LAYOUT_SPEC.md` (`../sdkwork-specs/`).

| Item | Value |
| --- | --- |
| `appId` | `sdkwork-drive` |
| `runtimeCode` | `drive` |
| Config root | `/etc/sdkwork/drive/` |
| Runtime TOML | `/etc/sdkwork/drive/config.toml` |
| Secrets | `/etc/sdkwork/drive/secrets/` |
| Override | `SDKWORK_DRIVE_CONFIG_FILE` |

Source profiles live under `etc/` (`sdkwork.deployment.config.json` index). Deploy manifest: `deployments/deploy.yaml`. Web data-plane source: `deployments/webserver/` (`SDKWORK_WEBSERVER_SPEC.md` layout v2).

```bash
node ../sdkwork-specs/tools/check-source-config-standard.mjs --root .
node ../sdkwork-specs/tools/check-application-deploy-layout.mjs --root .
node ../sdkwork-specs/tools/check-webserver-toml-standard.mjs --root deployments/webserver
```
<!-- /SDKWORK-DEPLOY-LAYOUT -->


