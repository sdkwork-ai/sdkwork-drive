-- sdkwork:migration
-- id: 0010_drive_logical_storage_buckets
-- engine: postgres
-- module: sdkwork-drive
-- purpose: Drop the logical storage plane tables (children first).
-- reversible: true
-- rollback: down-migration
-- transactional: true
-- lock: exclusive
-- lock_timeout: 2s
-- statement_timeout: 30s
-- Roll back the logical storage bucket layer. Provider, bucket and object data
-- are untouched; only the routing tables created by the up migration are
-- removed, children before parents.
DROP TABLE IF EXISTS dr_drive_business_storage_binding;
DROP TABLE IF EXISTS dr_drive_logical_bucket_provider_binding;
DROP TABLE IF EXISTS dr_drive_logical_bucket;
