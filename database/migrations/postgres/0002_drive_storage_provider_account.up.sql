-- sdkwork:migration
-- id: 0002_drive_storage_provider_account
-- engine: postgres
-- module: sdkwork-drive
-- purpose: Give dr_drive_storage_provider the tenant dimension every other
--   drive table already has, and let a provider reference a reusable
--   service-provider account held by the IAM platform account center
--   (iam_provider_account) instead of carrying its own credential string.
--   Existing rows are pinned to the platform sentinel tenant '0' so their
--   behaviour is unchanged, and the previously unconstrained credential_ref
--   column gets a scheme whitelist plus a mutual-exclusion rule against
--   provider_account_id so a provider can never read from two credential
--   sources at once.
-- reversible: true
-- rollback: down-migration
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

ALTER TABLE dr_drive_storage_provider
    ADD COLUMN IF NOT EXISTS tenant_id VARCHAR(64) NOT NULL DEFAULT '0';

ALTER TABLE dr_drive_storage_provider
    ADD COLUMN IF NOT EXISTS provider_account_id VARCHAR(128);

CREATE INDEX IF NOT EXISTS ix_dr_drive_storage_provider_tenant_name
    ON dr_drive_storage_provider (tenant_id, name);

CREATE INDEX IF NOT EXISTS ix_dr_drive_storage_provider_tenant_status
    ON dr_drive_storage_provider (tenant_id, status, updated_at DESC);

CREATE INDEX IF NOT EXISTS ix_dr_drive_storage_provider_account
    ON dr_drive_storage_provider (provider_account_id)
    WHERE provider_account_id IS NOT NULL;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'ck_dr_drive_storage_provider_credential_source'
    ) THEN
        ALTER TABLE dr_drive_storage_provider
            ADD CONSTRAINT ck_dr_drive_storage_provider_credential_source
            CHECK (NOT (provider_account_id IS NOT NULL AND credential_ref IS NOT NULL));
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'ck_dr_drive_storage_provider_credential_ref'
    ) THEN
        ALTER TABLE dr_drive_storage_provider
            ADD CONSTRAINT ck_dr_drive_storage_provider_credential_ref
            CHECK (
                credential_ref IS NULL
                OR credential_ref ~ '^(plain|env|secret|kms|vault):[^[:space:]]+$'
            );
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'ck_dr_drive_storage_provider_provider_account_id'
    ) THEN
        ALTER TABLE dr_drive_storage_provider
            ADD CONSTRAINT ck_dr_drive_storage_provider_provider_account_id
            CHECK (
                provider_account_id IS NULL
                OR provider_account_id ~ '^[A-Za-z0-9][A-Za-z0-9_.:-]{1,127}$'
            );
    END IF;
END $$;

COMMIT;
