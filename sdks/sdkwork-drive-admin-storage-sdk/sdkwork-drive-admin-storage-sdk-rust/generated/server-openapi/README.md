# sdkwork-drive-admin-storage-sdk (Rust)

Professional Rust SDK for SDKWork API.

## Installation

```bash
cargo add sdkwork-drive-admin-storage-sdk-generated-rust
```

## Quick Start

```rust
use sdkwork_drive_admin_storage_sdk_generated_rust::{SdkworkCustomClient, SdkworkConfig};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SdkworkCustomClient::new(SdkworkConfig::new("http://127.0.0.1:18080"))?;
    client.set_api_key("your-api-key");

    let result = client.drive().storage_provider_kinds_list().await?;
    println!("{result:?}");
    Ok(())
}
```

## Authentication Modes (Mutually Exclusive)

Choose exactly one mode for the same client instance.

### Mode A: API Key

```rust
let client = SdkworkCustomClient::new(SdkworkConfig::new("http://127.0.0.1:18080"))?;
client.set_api_key("your-api-key");
// Sends: Access-Token: <apiKey>
```

### Mode B: Dual Token

```rust
let client = SdkworkCustomClient::new(SdkworkConfig::new("http://127.0.0.1:18080"))?;
client.set_auth_token("your-auth-token");
client.set_access_token("your-access-token");
// Sends:
// Authorization: Bearer <authToken>
// Access-Token: <accessToken>
```

> Do not call `set_api_key(...)` together with `set_auth_token(...)` + `set_access_token(...)` on the same client.

## Configuration (Non-Auth)

```rust
let client = SdkworkCustomClient::new(SdkworkConfig::new("http://127.0.0.1:18080"))?;
client.set_header("X-Custom-Header", "value");
```

## API Modules

- `client.drive()` - drive API

## Usage Examples

### drive

```rust
// GET /backend/v3/api/drive/storage/provider-kinds
let result = client.drive().storage_provider_kinds_list().await?;
println!("{result:?}");
```

## Error Handling

```rust
use sdkwork_drive_admin_storage_sdk_generated_rust::{SdkworkCustomClient, SdkworkConfig};


let client = SdkworkCustomClient::new(SdkworkConfig::new("http://127.0.0.1:18080"))?;

let outcome: Result<(), _> = async {
    client.drive().storage_provider_kinds_list().await?;
    Ok(())
}.await;

match outcome {
    Ok(()) => println!("request completed"),
    Err(error) => eprintln!("request failed: {error}"),
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

> Set cargo registry credentials before `cargo publish` and use `--dry-run` first.

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
