# SDKWork Drive Skills

Repository/application skills live in subdirectories here when Drive needs a
checked-in workflow that is narrower than the root SDKWork standards.

Each real skill must use this shape:

```text
.sdkwork/skills/<skill-name>/
  SKILL.md
  references/
  scripts/
  assets/
```

Rules are inherited from `../../AGENTS.md`,
`../../../sdkwork-specs/SOUL.md`, and
`../../../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`.

Do not store product source, generated SDK output, runtime data, secrets,
private credentials, or dependency checkouts in this directory.
