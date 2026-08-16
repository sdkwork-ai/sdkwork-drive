# Drive Sandbox Explorer SDK Adapter

Maps a host-injected client port that is structurally compatible with the authenticated
composed `@sdkwork/drive-app-sdk` client to the narrow `SandboxExplorerPort` from
`@sdkwork/drive-pc-sandbox-contracts`. The React `@sdkwork/drive-pc-sandbox-explorer`
package consumes the same framework-neutral port.

The host owns client construction, runtime configuration, TokenManager wiring, and
session lifecycle. This package deliberately does not import or create a concrete SDK;
it does not read credentials, assemble headers, call raw HTTP, or expose provider physical paths.

```ts
const port = createDriveSandboxExplorerSdkPort({ client: driveAppClient });
configureDriveSandboxExplorerRuntime({ port });
```
