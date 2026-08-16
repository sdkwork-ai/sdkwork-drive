# Drive Topology Profiles

Machine contract: `../../specs/topology.spec.json` (`schemaVersion: 5`,
archetype `application-http-gateway`).

Platform standard: `../../sdkwork-specs/APP_RUNTIME_TOPOLOGY_SPEC.md`.

| Profile id | Command |
| --- | --- |
| `standalone.development` | `pnpm dev`, `pnpm dev:standalone`, `pnpm dev:desktop` |
| `cloud.development` | `pnpm dev:cloud`, `pnpm dev:desktop:cloud` |
| `standalone.production` | `pnpm build:standalone` |
| `cloud.production` | `pnpm build` |

`cloud.development` declares only local browser/desktop clients. Both public
surfaces resolve to the deployed `platform.api-gateway` surface, and local gateway
autostart is disabled.

Loader: `../../scripts/lib/drive-topology.mjs` delegates to
`@sdkwork/app-topology`.
