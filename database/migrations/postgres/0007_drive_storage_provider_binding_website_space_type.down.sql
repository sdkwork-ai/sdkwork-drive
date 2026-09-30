-- sdkwork:migration
-- id: 0007_drive_storage_provider_binding_website_space_type
-- engine: postgres
-- module: sdkwork-drive
-- purpose: Restore the ten-value binding purpose check.
-- reversible: false
-- rollback: restore-from-backup
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s
--
-- Precondition: no `website` space-type binding rows may exist, or narrowing the
-- check fails. That is deliberate — silently deleting an operator's binding to
-- make a rollback succeed would strand published-site objects behind a target
-- nobody chose. Re-point the website binding (or clear it) before rolling back.

BEGIN;

ALTER TABLE dr_drive_storage_provider_binding
    DROP CONSTRAINT IF EXISTS ck_dr_drive_storage_provider_binding_purpose;

ALTER TABLE dr_drive_storage_provider_binding
    ADD CONSTRAINT ck_dr_drive_storage_provider_binding_purpose
        CHECK (
            (binding_scope IN ('tenant', 'space') AND purpose = 'primary')
            OR (
                binding_scope = 'space_type'
                AND purpose IN (
                    'personal', 'team', 'knowledge_base', 'ai_generated', 'git_repository',
                    'deployment', 'app_upload', 'im', 'rtc', 'notary'
                )
            )
        );

COMMIT;
