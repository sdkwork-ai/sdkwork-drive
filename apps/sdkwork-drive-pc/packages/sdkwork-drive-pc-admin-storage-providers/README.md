# sdkwork-drive-pc-admin-storage-providers

Domain: drive
Capability: storage-providers
Package type: PC internal admin React package
Status: standard

This package owns internal operator UI for Drive storage provider configuration management. It consumes the generated Drive admin storage SDK through the Drive PC runtime service boundary and must not construct raw HTTP requests, manual auth headers, provider SDK clients, or generated SDK internals.

## Behaviour contracts

- **The provider rail filters on the credential.** Its three tabs (configured /
  missing / all) narrow the same loaded option set the rail's search box narrows,
  on the predicate the credential dot on each row already renders
  (`credentialConfigured`). The rail opens on *configured* — the list answers
  "which configurations can I browse right now" first, and *all* is one click
  away. Both filters run client-side over what is loaded — the rail reads one
  maximum-size page plus an explicit continuation — so the tab counts are loaded
  counts, and the rail says so while a continuation remains. Changing the tab
  moves the selection into that tab (never the other way round: an empty tab
  leaves the detail pane where it is), because a host that preselects the first
  configuration of the unfiltered option set would otherwise open on a row the
  rail does not show.
- **Built-in configuration names follow the console language.**
  `providerDisplayName` localizes a row that both came from the backend bootstrap
  (id prefixed `builtin-storage-provider-`) and still carries the name that
  bootstrap wrote; a configuration an operator named keeps that name verbatim, in
  whatever language it was typed. The registry's English copy mirrors the server's
  `provider_account_defaults.rs`, and `tests/providerKindCatalog.test.ts` fails
  when the two drift.

- **Timestamps cross the service boundary as ISO 8601.** `creationDateIso` and
  `lastModifiedIso` are what the views carry; the shared `formatDriveDate` /
  `formatDriveDateTime` helpers are the only formatters and they take the console
  language. A `toLocaleDateString()`-style call inside the service reads the *browser*
  locale, which is how a Chinese console came to render one language's dates inside an
  already-localized page (and how the object list ended up re-parsing a localized string).

## Public API

- `.`

## Required SDK Surface

- `@sdkwork/drive-admin-storage-sdk`

## Verification

- `pnpm test -- packages/sdkwork-drive-pc-admin-storage-providers/tests/storageProviderAdminService.test.ts`
- `pnpm typecheck`
