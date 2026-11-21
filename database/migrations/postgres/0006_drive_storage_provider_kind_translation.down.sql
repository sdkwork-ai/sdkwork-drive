-- sdkwork:migration
-- id: 0006_drive_storage_provider_kind_translation
-- engine: postgres
-- module: sdkwork-drive
-- purpose: Rollback of the provider-kind translation table. The base catalog
--   keeps serving its locale-neutral display names after this rollback.
-- reversible: true
-- rollback: down-migration
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

DROP TABLE IF EXISTS dr_drive_storage_provider_kind_translation;

COMMIT;
