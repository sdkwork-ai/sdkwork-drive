# SDKWork Drive Agent Plugins

Repository/application agent plugins live in subdirectories here. This
directory is distinct from top-level `plugins/`, which is reserved for
product/runtime plugin source.

Installable Codex plugins must declare:

```text
.sdkwork/plugins/<plugin-name>/
  .codex-plugin/
    plugin.json
```

Plugin skills and scripts must call the canonical Drive commands and root
SDKWork specs instead of redefining API, SDK, security, runtime, or deployment
rules locally.
