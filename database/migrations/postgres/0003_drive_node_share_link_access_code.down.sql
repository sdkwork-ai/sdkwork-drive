-- sdkwork:migration
-- id: 0003_drive_node_share_link_access_code
-- engine: postgres
-- module: sdkwork-drive
-- purpose: Roll back the access_code_hash column added to
--   dr_drive_node_share_link. Dropping it restores the pre-migration shape;
--   any stored access codes are intentionally discarded because without the
--   column the share-link read path never enforced them.
-- reversible: true
-- rollback: up-migration
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

ALTER TABLE dr_drive_node_share_link
    DROP COLUMN IF EXISTS access_code_hash;

COMMIT;
