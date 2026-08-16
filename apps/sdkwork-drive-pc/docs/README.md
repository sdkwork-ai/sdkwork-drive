# SDKWork Drive PC Documentation

Documentation for the PC application root follows [../../../sdkwork-specs/DOCUMENTATION_SPEC.md](../../../sdkwork-specs/DOCUMENTATION_SPEC.md). Repository-wide platform docs live under [../../../../docs/README.md](../../../../docs/README.md).

## Audience Routing

| I am… | Read first | Then read |
| --- | --- | --- |
| Product or business | [product/prd/PRD.md](product/prd/PRD.md) | [../../../../docs/product/prd/PRD.md](../../../../docs/product/prd/PRD.md) |
| Architect | [architecture/tech/TECH_ARCHITECTURE.md](architecture/tech/TECH_ARCHITECTURE.md) | [../../../../docs/architecture/tech/TECH_ARCHITECTURE.md](../../../../docs/architecture/tech/TECH_ARCHITECTURE.md) |
| Developer | [guides/developer/README.md](guides/developer/README.md) | application `README.md`, `specs/component.spec.json` |
| Operator | [guides/operator/README.md](guides/operator/README.md) | [../../../../docs/guides/operator/pre-launch-checklist.md](../../../../docs/guides/operator/pre-launch-checklist.md) |
| Agent | [../AGENTS.md](../AGENTS.md) | [INDEX.yaml](INDEX.yaml) |

## Canon Documents

| Document | Path |
| --- | --- |
| PC product PRD | [product/prd/PRD.md](product/prd/PRD.md) |
| PC technical architecture | [architecture/tech/TECH_ARCHITECTURE.md](architecture/tech/TECH_ARCHITECTURE.md) |

## Verification

```bash
pnpm test
pnpm typecheck
pnpm check:pc-standard
node ../../../sdkwork-specs/tools/check-repository-docs-standard.mjs --root .
```
