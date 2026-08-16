# SDKWork Drive PC Source Configuration

`sdkwork.deployment.config.json` references the repository-level Drive profile
authority at `../../../etc/sdkwork.deployment.config.json`. The renderer does
not own a second set of public domains or SDK Base URLs.

`browser.runtime.json` declares only renderer-safe binding names. The example
files under `browser/`, `desktop/`, `container/`, and `server/` are safe source
templates; host-local overrides and secrets must not be committed here.

Validate from this application root with:

```powershell
node ../../../sdkwork-specs/tools/check-source-config-standard.mjs --root .
```
