-- sdkwork:migration
-- id: 0009_drive_node_space_type_documents
-- engine: postgres
-- module: sdkwork-drive
-- purpose: Restore the pre-documents node space type constraint.
-- reversible: true
-- rollback: forward-fix
-- transactional: true
-- lock: exclusive
-- lock_timeout: 2s
-- statement_timeout: 30s

ALTER TABLE dr_drive_node
    DROP CONSTRAINT IF EXISTS ck_dr_drive_node_space_type;

ALTER TABLE dr_drive_node
    ADD CONSTRAINT ck_dr_drive_node_space_type
    CHECK (space_type IN ('personal', 'team', 'knowledge_base', 'ai_generated', 'git_repository', 'deployment', 'app_upload', 'im', 'rtc', 'notary', 'website'));
