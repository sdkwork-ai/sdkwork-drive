-- sdkwork:migration
-- id: 0013_drive_storage_provider_binding_documents_certificate_space_types
-- engine: postgres
-- module: sdkwork-drive
-- purpose: Restore the pre-documents binding purpose vocabulary.
--
-- reversible: true
-- rollback: forward-migration
-- transactional: true
-- lock: ACCESS EXCLUSIVE on dr_drive_storage_provider_binding
-- lock_timeout: 2s
-- statement_timeout: 30s
--
-- CAVEAT: this narrowing fails while a `documents` or `certificate` space-type
-- binding row exists. That is deliberate: the constraint refuses to leave a
-- binding that the new vocabulary can no longer describe.

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
                'deployment', 'app_upload', 'im', 'rtc', 'notary', 'website'
            )
        )
    );
