-- sdkwork:migration
-- id: 0011_drive_space_type_certificate
-- engine: postgres
-- module: sdkwork-drive
-- purpose: Restore the pre-certificate space type vocabulary.
-- reversible: true
-- rollback: forward-only data caveat
-- transactional: true
-- lock: exclusive
-- lock_timeout: 2s
-- statement_timeout: 30s
--
-- CAVEAT: this narrowing fails if any row already carries
-- space_type = 'certificate'. That is deliberate: the constraint refuses to
-- orphan (or silently delete) archived certificate bundles. An environment that
-- has issued certificates must keep the new constraint.

ALTER TABLE dr_drive_space
    DROP CONSTRAINT IF EXISTS ck_dr_drive_space_space_type;

ALTER TABLE dr_drive_space
    ADD CONSTRAINT ck_dr_drive_space_space_type
    CHECK (space_type IN ('personal', 'team', 'knowledge_base', 'ai_generated', 'git_repository', 'deployment', 'app_upload', 'im', 'rtc', 'notary', 'website', 'documents'));
