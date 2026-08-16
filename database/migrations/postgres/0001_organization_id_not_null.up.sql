-- sdkwork:migration
-- id: 0001_organization_id_not_null
-- engine: postgres
-- module: sdkwork-drive
-- purpose: Enforce organization_id NOT NULL DEFAULT on all tables in the
--   consolidated baseline. NULL rows (pre-standard data anomalies) are
--   backfilled with the platform sentinel before NOT NULL is set, and
--   NOT NULL columns without an explicit default receive the sentinel
--   default, keeping existing deployments consistent with fresh baseline
--   installs.
-- reversible: false
-- rollback: forward-fix (sentinel backfill is the canonical fix; NULL
--   organization rows are data anomalies)
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

ALTER TABLE dr_drive_upload_item ADD COLUMN IF NOT EXISTS organization_id TEXT NOT NULL DEFAULT '0';
UPDATE dr_drive_upload_item SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE dr_drive_upload_item ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE dr_drive_upload_item ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE dr_drive_file_sensitive_operation ADD COLUMN IF NOT EXISTS organization_id TEXT NOT NULL DEFAULT '0';
UPDATE dr_drive_file_sensitive_operation SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE dr_drive_file_sensitive_operation ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE dr_drive_file_sensitive_operation ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE dr_drive_sandbox_volume ADD COLUMN IF NOT EXISTS organization_id TEXT NOT NULL DEFAULT '0';
UPDATE dr_drive_sandbox_volume SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE dr_drive_sandbox_volume ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE dr_drive_sandbox_volume ALTER COLUMN organization_id SET NOT NULL;

COMMIT;
