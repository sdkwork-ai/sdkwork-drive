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
- `003_logical_storage_buckets.sql` — the logical storage plane on top of the
  physical one: `business`/`system`/`archive`/`temp` logical buckets, their
  provider bindings (all pointing at the default Tencent COS provider), and the
  business bindings for every space type plus the tenant fallback. A fresh
  install is fully routed before an operator changes anything; completing setup
  only requires editing the provider account/credential.

The file is registered in the `standard` seed profile, which
`database.manifest.json` selects by default (`seedOnBoot = true`). The Drive
database bootstrap applies it after migrations, so a fresh install and an
upgrade are complete without an operator running `pnpm db:seed` by hand.
