# SDKWork Drive Workspace Metadata

This directory stores repository/application development metadata for the
`sdkwork-drive` root. It is governed by
`../../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`.

`.sdkwork/` is not runtime state. Do not place user-private config, generated
SDK transport output, databases, logs, caches, secrets, or dependency source
checkouts here.

Authoritative local entries:

- `skills/`: repository/application-local agent and operator workflows.
- `plugins/`: repository/application-local agent plugin bundles.
- `.gitignore`: local-only metadata state that must stay out of source control.

Use `../AGENTS.md` for the execution order and `../../sdkwork-specs/SOUL.md`
for the shared SDKWork agent behavior.
