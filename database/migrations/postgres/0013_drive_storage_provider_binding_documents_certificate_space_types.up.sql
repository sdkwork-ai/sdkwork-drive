-- sdkwork:migration
-- id: 0013_drive_storage_provider_binding_documents_certificate_space_types
-- engine: postgres
-- module: sdkwork-drive
-- purpose: Let a storage binding target the documents and certificate space types.
--
--   The space-type binding check listed eleven types while the space and node
--   checks (and the route validator) already accepted `documents`, and
--   `certificate` was added to both. A binding row for either type therefore
--   failed at the database after the console offered it — the exact drift the
--   `spaceTypeCatalog` console test exists to catch. This migration makes the
--   fourth copy of the vocabulary agree with the other three.
--
-- reversible: true
-- rollback: 0013_drive_storage_provider_binding_documents_certificate_space_types.down.sql
-- transactional: true
-- lock: ACCESS EXCLUSIVE on dr_drive_storage_provider_binding
-- lock_timeout: 2s
-- statement_timeout: 30s

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
                'deployment', 'app_upload', 'im', 'rtc', 'notary', 'website', 'documents',
                'certificate'
            )
        )
    );
