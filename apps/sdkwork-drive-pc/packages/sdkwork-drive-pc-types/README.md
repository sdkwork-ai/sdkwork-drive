# sdkwork-drive-pc-types

Domain: drive
Capability: types
Package type: node-package
Status: compatibility facade

This README is the SDKWork module entrypoint for `sdkwork-drive-pc-types`. The package is a compatibility facade that re-exports the authoritative Drive PC UI/domain types from `sdkwork-drive-pc-core/types`, so `sdkwork-drive-pc-core` remains the core composition owner and does not depend on a feature or compatibility package. The machine-readable component contract is `specs/component.spec.json`; canonical standards are under `../../../../../sdkwork-specs/`.

## Public API

- `.` re-exports `sdkwork-drive-pc-core/types`

## Required SDK Surface

- None declared in `specs/component.spec.json`.

## Configuration

Configuration keys, runtime entrypoints, and integration contracts are declared in `specs/component.spec.json`. Shared modules must receive configuration through typed bootstrap or service boundaries rather than reading host-local environment state directly.

## SaaS/Private/Local Behavior

This component follows the deployment and runtime rules referenced by its `canonicalSpecs` entries. SaaS, private, and local behavior must stay compatible with the relevant SDKWork specs before implementation changes are made.

## Security

Do not add secrets, live tokens, manual auth headers, or app-local credential handling to this module. Protected API and SDK access must use the generated SDK or approved service boundary declared in the component contract.

## Extension Points

Extension points are limited to public exports, runtime entrypoints, SDK clients, events, and config keys declared in `specs/component.spec.json`.

## Verification

- `pnpm typecheck`

## Owner And Status

Owner and lifecycle status are tracked in `specs/component.spec.json`. Update that contract before changing public integration behavior.
