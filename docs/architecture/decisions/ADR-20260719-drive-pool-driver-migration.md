# ADR-20260719: Drive Pool Driver Migration

- Status: temporary exception
- Owner: sdkwork-drive maintainers
- Removal milestone: before the next Drive production release
- Canonical standard: `../../../../sdkwork-specs/DATABASE_SPEC_PROCESS_SHARED_POOL.md`

## Decision

Every Drive database-owning process enables the canonical process-shared pool before lifecycle or
repository bootstrap. The Drive lifecycle host owns a typed `sqlx::PgPool` for PostgreSQL profiles,
and every lifecycle consumer resolves that installed handle.

Drive repositories currently expose `sqlx::AnyPool` because the same authored repository surface
supports explicit SQLite application profiles. Until the PostgreSQL production repository surface
migrates to `sqlx::PgPool`, the framework owns one identity-checked compatibility `AnyPool` per
process. `SDKWORK_DATABASE_TEMPORARY_ANY_POOL_EXCEPTION=true` must be present before the canonical
pool is created.

The configured `SDKWORK_DATABASE_MAX_CONNECTIONS` value is the combined process budget. The
database framework divides it between the canonical pool and compatibility pool; an odd connection
is assigned to the canonical pool. Modules cannot expand the compatibility allocation.

This exception is migration debt and is not strict single-driver pool compliance.

## Removal Criteria

1. PostgreSQL Drive repositories accept the installed typed process pool.
2. PostgreSQL gateway and worker paths no longer construct or resolve `sqlx::AnyPool`.
3. SQLite is compiled or isolated as an explicit independent application process.
4. The temporary exception and environment switch are removed from Drive profiles and contracts.
5. Live PostgreSQL evidence shows one process pool and clean shutdown releases every connection.
