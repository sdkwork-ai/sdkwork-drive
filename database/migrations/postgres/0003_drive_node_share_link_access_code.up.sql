-- sdkwork:migration
-- id: 0003_drive_node_share_link_access_code
-- engine: postgres
-- module: sdkwork-drive
-- purpose: dr_drive_node_share_link is queried for access_code_hash by the
--   drive open-api share-link path (find_active_share_link) and by the app-api
--   collaboration surface, but the baseline DDL never carried the column and
--   the runtime bootstrap's self-heal block never added it. Every protected
--   share-link read therefore failed with
--   `column sl.access_code_hash does not exist`; because the call sites read
--   the value through `try_get(...).ok().flatten()`, the error was swallowed
--   and the access code silently stopped being enforced.
--   This migration adds the column the code and the runtime bootstrap both
--   already assume.
-- reversible: true
-- rollback: down-migration
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

ALTER TABLE dr_drive_node_share_link
    ADD COLUMN IF NOT EXISTS access_code_hash VARCHAR(80);

COMMENT ON COLUMN dr_drive_node_share_link.access_code_hash IS
    'sha256:<hex> digest of the share access code; NULL means the link needs no code';

COMMIT;
