-- sdkwork:migration
-- id: 0011_drive_space_type_certificate
-- engine: postgres
-- module: sdkwork-drive
-- purpose: Allow the dedicated certificate bundle space used by the deploy control plane.
-- reversible: true
-- rollback: 0011_drive_space_type_certificate.down.sql
-- transactional: true
-- lock: exclusive
-- lock_timeout: 2s
-- statement_timeout: 30s

ALTER TABLE dr_drive_space
    DROP CONSTRAINT IF EXISTS ck_dr_drive_space_space_type;

ALTER TABLE dr_drive_space
    ADD CONSTRAINT ck_dr_drive_space_space_type
    CHECK (space_type IN ('personal', 'team', 'knowledge_base', 'ai_generated', 'git_repository', 'deployment', 'app_upload', 'im', 'rtc', 'notary', 'website', 'documents', 'certificate'));
