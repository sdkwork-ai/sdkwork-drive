-- sdkwork:migration
-- id: 0002_drive_storage_provider_account
-- engine: postgres
-- module: sdkwork-drive
-- purpose: Reverse 0002_drive_storage_provider_account.up.sql.
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

ALTER TABLE dr_drive_storage_provider
    DROP CONSTRAINT IF EXISTS ck_dr_drive_storage_provider_provider_account_id;

ALTER TABLE dr_drive_storage_provider
    DROP CONSTRAINT IF EXISTS ck_dr_drive_storage_provider_credential_ref;

ALTER TABLE dr_drive_storage_provider
    DROP CONSTRAINT IF EXISTS ck_dr_drive_storage_provider_credential_source;

DROP INDEX IF EXISTS ix_dr_drive_storage_provider_account;
DROP INDEX IF EXISTS ix_dr_drive_storage_provider_tenant_status;
DROP INDEX IF EXISTS ix_dr_drive_storage_provider_tenant_name;

-- provider_account_id is dropped outright: it is a pure reference with no
-- local data of its own. tenant_id is dropped last, after the indexes that
-- lead with it.
ALTER TABLE dr_drive_storage_provider
    DROP COLUMN IF EXISTS provider_account_id;

ALTER TABLE dr_drive_storage_provider
    DROP COLUMN IF EXISTS tenant_id;

COMMIT;
