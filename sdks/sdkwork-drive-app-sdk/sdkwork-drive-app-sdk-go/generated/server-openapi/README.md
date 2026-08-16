# sdkwork-drive-app-sdk (Go)

Generated SDKWork v3 dual-token transport SDK.

## Installation

```bash
go get sdkwork-drive-app-sdk-generated-go
```

## Quick Start

```go
package main

import (
    "fmt"
    "sdkwork-drive-app-sdk-generated-go"
    sdkhttp "sdkwork-drive-app-sdk-generated-go/http"

)

func main() {
    cfg := sdkhttp.NewDefaultConfig("http://127.0.0.1:18080")
    client := sdkwork-drive-app-sdk-generated-go.NewSdkworkAppClientWithConfig(cfg)
    client.SetAuthToken("your-auth-token")
client.SetAccessToken("your-access-token")
    
    // Use the SDK
    result, err := client.Drive.QuotasRetrieve()
    if err != nil {
        panic(err)
    }
    fmt.Println(result)
}
```

## Authentication

```text
Authorization: Bearer <authToken>
Access-Token: <accessToken>
```


## Configuration (Non-Auth)

```go
cfg := sdkhttp.NewDefaultConfig("http://127.0.0.1:18080")
client := sdkwork-drive-app-sdk-generated-go.NewSdkworkAppClientWithConfig(cfg)

// Set custom headers
client.SetHeader("X-Custom-Header", "value")
```

## API Modules

- `client.Drive` - drive API
- `client.Assets` - assets API

## Usage Examples

### drive

```go
// GET /app/v3/api/drive/quotas/summary
result, err := client.Drive.QuotasRetrieve()
if err != nil {
    panic(err)
}
fmt.Println(result)
```

### assets

```go
// List asset collections
params := map[string]interface{}{
    "cursor": "cursor",
    "page_size": 2,
}
result, err := client.Assets.AssetCollectionsList(params)
if err != nil {
    panic(err)
}
fmt.Println(result)
```

## Error Handling

```go
_, err := client.Drive.QuotasRetrieve()
if err != nil {
    // Handle error
    fmt.Println("Error:", err)
    return
}
```

## Publishing

This SDK includes cross-platform publish scripts in `bin/`:
- `bin/publish-core.mjs`
- `bin/publish.sh`
- `bin/publish.ps1`

### Check

```bash
./bin/publish.sh --action check
```

### Publish

```bash
./bin/publish.sh --action publish --channel release
```

```powershell
.\bin\publish.ps1 --action publish --channel test --dry-run
```

> Set `GO_RELEASE_TAG` (or `SDKWORK_RELEASE_TAG`) and push tag if needed.

## License

MIT

## Regeneration Contract

- HTTP/OpenAPI generator-owned files are tracked in `.sdkwork/sdkwork-generator-manifest.json`.
- HTTP/OpenAPI generation also writes `.sdkwork/sdkwork-generator-changes.json` so automation can inspect created, updated, deleted, unchanged, scaffolded, and backed-up files plus the classified impact areas, verification plan, and execution decision for the latest generation.
- HTTP/OpenAPI apply mode also writes `.sdkwork/sdkwork-generator-report.json` with the full execution report, including `schemaVersion`, `generator`, stable artifact paths, and the execution handoff commands that match CLI `--json` output.
- CLI JSON output also includes an execution handoff with concrete next commands, including reviewed apply commands for dry-run flows.
- Put HTTP/OpenAPI hand-written wrappers, adapters, and orchestration in `custom/`.
- Files scaffolded under `custom/` are created once and preserved across HTTP/OpenAPI regenerations.
- If an HTTP/OpenAPI generated-owned file was modified locally, its previous content is copied to `.sdkwork/manual-backups/` before overwrite or removal.
- RPC SDK source workspaces use convention-first evidence by default: RPC SDK family naming, language workspace naming, `rpc/*.manifest.json`, proto source references, generated client source, and native package manifests.
- Use `sdkgen inspect --protocol rpc` to verify RPC convention evidence. Request persisted generator evidence only with `--emit-control-plane` for release, CI, audit, or migration workflows; evidence paths are derived by generator convention.
