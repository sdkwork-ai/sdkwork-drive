# Common seed scripts

Locale-agnostic seed SQL referenced from `seeds/seed.manifest.json` profiles.

- `001_bootstrap.sql` — intentional no-op lifecycle smoke seed.
- `002_storage_provider_catalog.sql` — storage-center reference data used by
  `/admin/storage/providers` and `/admin/storage/defaultBuckets`:
  the 25 built-in provider kinds, one provider configuration per mainstream
  cloud vendor (plus the credential-free `local_filesystem` fallback) for the
  platform bootstrap tenant, and the tenant default binding pointing at Tencent
  Cloud COS. Every row is `ON CONFLICT DO NOTHING`, so an install or upgrade
  only adds what is missing and never rewrites an operator edit.

The file is registered in the `standard` seed profile, which
`database.manifest.json` selects by default (`seedOnBoot = true`). The Drive
database bootstrap applies it after migrations, so a fresh install and an
upgrade are complete without an operator running `pnpm db:seed` by hand.
