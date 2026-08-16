# sdkwork-drive-sdk (Java)

Professional Java SDK for SDKWork API.

## Installation

Add to your `pom.xml`:

```xml
<dependency>
    <groupId>com.sdkwork</groupId>
    <artifactId>sdkwork-drive-sdk-generated-java</artifactId>
    <version>0.1.0</version>
</dependency>
```

Or with Gradle:

```groovy
implementation 'com.sdkwork:sdkwork-drive-sdk-generated-java:0.1.0'
```

## Quick Start

```java
import com.sdkwork.drive.sdk.generated.java.SdkworkCustomClient;
import com.sdkwork.common.core.Types;
import com.sdkwork.drive.sdk.generated.java.model.*;
import java.util.LinkedHashMap;
import java.util.Map;

public class Main {
    public static void main(String[] args) throws Exception {
        Types.SdkConfig config = new Types.SdkConfig("http://127.0.0.1:18082");
        SdkworkCustomClient client = new SdkworkCustomClient(config);
        client.setApiKey("your-api-key");

        // Use the SDK
        String token = "token";
        Map<String, Object> params = new LinkedHashMap<>();
        params.put("accessCode", "ok");
        OpenShareLinksRetrieveResponse result = client.getDrive().openShareLinksRetrieve(token, params);
        System.out.println(result);
    }
}
```

## Authentication Modes (Mutually Exclusive)

Choose exactly one mode for the same client instance.

### Mode A: API Key

```java
Types.SdkConfig config = new Types.SdkConfig("http://127.0.0.1:18082");
SdkworkCustomClient client = new SdkworkCustomClient(config);
client.setApiKey("your-api-key");
// Sends: X-Api-Key: <apiKey>
```

### Mode B: Dual Token

```java
Types.SdkConfig config = new Types.SdkConfig("http://127.0.0.1:18082");
SdkworkCustomClient client = new SdkworkCustomClient(config);
client.setAuthToken("your-auth-token");
client.setAccessToken("your-access-token");
// Sends:
// Authorization: Bearer <authToken>
// Access-Token: <accessToken>
```

> Do not call `setApiKey(...)` together with `setAuthToken(...)` + `setAccessToken(...)` on the same client.

## Configuration (Non-Auth)

```java
Types.SdkConfig config = new Types.SdkConfig("http://127.0.0.1:18082");
SdkworkCustomClient client = new SdkworkCustomClient(config);

// Set custom headers
client.getHttpClient().setHeader("X-Custom-Header", "value");
```

## API Modules

- `client.getDrive()` - drive API

## Usage Examples

### drive

```java
// GET /open/v3/api/drive/share_links/{token}
String token = "token";
Map<String, Object> params = new LinkedHashMap<>();
params.put("accessCode", "ok");
OpenShareLinksRetrieveResponse result = client.getDrive().openShareLinksRetrieve(token, params);
System.out.println(result);
```

## Error Handling

```java
try {
    String token = "token";
    Map<String, Object> params = new LinkedHashMap<>();
    params.put("accessCode", "ok");
    OpenShareLinksRetrieveResponse result = client.getDrive().openShareLinksRetrieve(token, params);
    System.out.println(result);
} catch (Exception e) {
    System.err.println("Error: " + e.getMessage());
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

> Use Maven `settings.xml` credentials and optional `MAVEN_PUBLISH_PROFILE`.

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
