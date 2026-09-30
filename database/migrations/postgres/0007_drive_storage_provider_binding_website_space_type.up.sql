-- sdkwork:migration
-- id: 0007_drive_storage_provider_binding_website_space_type
-- engine: postgres
-- module: sdkwork-drive
-- purpose: Allow a `website` space type to carry its own storage binding.
--
--   `dr_drive_space.space_type` (and the denormalized `dr_drive_node.space_type`)
--   have always accepted eleven values, `website` among them, and the admin
--   storage route validates the same eleven. The binding table's
--   `ck_dr_drive_storage_provider_binding_purpose` check listed only ten, so a
--   `PATCH /drive/storage/bindings/default { spaceType: "website" }` passed route
--   validation and then failed the table check: published-site content could be
--   given no storage target of its own and silently fell through to the tenant
--   default, which is exactly the case an operator wants separated (a public,
--   CDN-fronted bucket rather than the tenant's private one).
--
--   Widening the check keeps the three space-type copies in agreement
--   (baseline DDL, this migration, `validate_storage_binding_space_type`) and is
--   backward compatible: every previously legal row stays legal.
-- reversible: true
-- rollback: down-migration
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

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
                    'deployment', 'app_upload', 'im', 'rtc', 'notary', 'website'
                )
            )
        );

COMMIT;
