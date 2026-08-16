# sdkwork-drive-sdk (Go)

Professional Go SDK for SDKWork API.

## Installation

```bash
go get sdkwork-drive-sdk-generated-go
```

## Quick Start

```go
package main

import (
    "fmt"
    "sdkwork-drive-sdk-generated-go"
    sdkhttp "sdkwork-drive-sdk-generated-go/http"

)

func main() {
    cfg := sdkhttp.NewDefaultConfig("http://127.0.0.1:18082")
    client := sdkwork-drive-sdk-generated-go.NewSdkworkCustomClientWithConfig(cfg)
    client.SetApiKey("your-api-key")
    
    // Use the SDK
    token := "token"
    params := map[string]interface{}{
        "accessCode": "accessCode",
    }
    result, err := client.Drive.OpenShareLinksRetrieve(token, params)
    if err != nil {
        panic(err)
    }
    fmt.Println(result)
}
```

## Authentication Modes (Mutually Exclusive)

Choose exactly one mode for the same client instance.

### Mode A: API Key

```go
cfg := sdkhttp.NewDefaultConfig("http://127.0.0.1:18082")
client := sdkwork-drive-sdk-generated-go.NewSdkworkCustomClientWithConfig(cfg)
client.SetApiKey("your-api-key")
// Sends: X-Api-Key: <apiKey>
```

### Mode B: Dual Token

```go
cfg := sdkhttp.NewDefaultConfig("http://127.0.0.1:18082")
client := sdkwork-drive-sdk-generated-go.NewSdkworkCustomClientWithConfig(cfg)
client.SetAuthToken("your-auth-token")
client.SetAccessToken("your-access-token")
// Sends:
// Authorization: Bearer <authToken>
// Access-Token: <accessToken>
```

> Do not call `SetApiKey(...)` together with `SetAuthToken(...)` + `SetAccessToken(...)` on the same client.

## Configuration (Non-Auth)

```go
cfg := sdkhttp.NewDefaultConfig("http://127.0.0.1:18082")
client := sdkwork-drive-sdk-generated-go.NewSdkworkCustomClientWithConfig(cfg)

// Set custom headers
client.SetHeader("X-Custom-Header", "value")
```

## API Modules

- `client.Drive` - drive API

## Usage Examples

### drive

```go
// GET /open/v3/api/drive/share_links/{token}
token := "token"
params := map[string]interface{}{
    "accessCode": "accessCode",
}
result, err := client.Drive.OpenShareLinksRetrieve(token, params)
if err != nil {
    panic(err)
}
fmt.Println(result)
```

## Error Handling

```go
token := "token"
params := map[string]interface{}{
    "accessCode": "accessCode",
}
_, err := client.Drive.OpenShareLinksRetrieve(token, params)
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
