# DRIVE Database Module

Canonical lifecycle assets for `sdkwork-drive` per `DATABASE_FRAMEWORK_SPEC.md`.

- moduleId: `drive`
- serviceCode: `DRIVE`
- tablePrefix: `dr_` (physical tables; manifest module prefix remains `drive_`)

## Initialization state

This module is in **initialization state** for greenfield deployments:

1. **Baseline** — `database/ddl/baseline/{engine}/0001_drive_baseline.sql` contains the full DDL snapshot.
2. **Migrations** — `database/migrations/{engine}/` is reserved for post-GA incremental schema changes only. It is intentionally empty at initialization.
3. **Drift** — run `pnpm db:drift:check` before release.

## Seed data

`database.manifest.json` declares `seedOnBoot = true` and the `standard` seed
profile. `bootstrap_drive_database` applies that profile after migrations on
every converge path (standalone serve, the CloudRouter federated assembly, and
`pnpm db:init|migrate`), so install and upgrade leave the database in the same
state as a fresh seed run.

The standard profile includes `common/002_storage_provider_catalog.sql`, which
installs the storage-center reference data the admin console reads:

- the 25 built-in storage provider kinds;
- one provider configuration per mainstream cloud vendor plus the
  credential-free `local_filesystem` fallback for the platform bootstrap tenant
  (`100001`), with path-style/TLS/SSE/storage-class defaults and an `env:`
  credential reference per vendor;
- the tenant default binding, pointing at Tencent Cloud COS.

Operators therefore only edit the credential environment or the provider's
endpoint/region/bucket to start using a vendor. Seeded values are never
rewritten by a later seed run.

## Commands

```bash
pnpm run db:validate
pnpm run db:materialize:contract
pnpm run db:plan
pnpm run db:init
pnpm run db:migrate
pnpm run db:seed
pnpm run db:status
pnpm run db:drift:check
```
