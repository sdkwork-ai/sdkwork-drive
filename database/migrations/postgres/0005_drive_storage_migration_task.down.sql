-- sdkwork:migration
-- id: 0005_drive_storage_migration_task
-- engine: postgres
-- module: sdkwork-drive
-- purpose: Roll back the cross-provider storage migration tables.
-- reversible: false
-- rollback: restore-from-backup
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s
--
-- Dropping this table discards migration history: which objects were copied,
-- which failed, and which operator started the run. The objects themselves are
-- untouched — a completed migration already repointed the live rows — but the
-- audit trail of how they got there is gone and cannot be reconstructed from
-- the object store, because S3 keys carry no migration provenance. Take a
-- logical dump of both tables before applying this down-migration in any
-- environment whose history matters.

BEGIN;

DROP TABLE IF EXISTS dr_drive_storage_migration_item;
DROP TABLE IF EXISTS dr_drive_storage_migration;

COMMIT;
