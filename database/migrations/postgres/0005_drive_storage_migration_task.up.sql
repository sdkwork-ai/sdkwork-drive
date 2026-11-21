-- sdkwork:migration
-- id: 0005_drive_storage_migration_task
-- engine: postgres
-- module: sdkwork-drive
-- purpose: Record cross-provider storage migrations so an operator can move a
--   tenant's objects from one storage provider to another without downtime and
--   without stranding history. See the down-migration for the rationale.
-- reversible: true
-- rollback: down-migration
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

CREATE TABLE IF NOT EXISTS dr_drive_storage_migration (
    id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    -- The provider whose objects are being moved.
    source_provider_id VARCHAR(64) NOT NULL,
    -- The provider the objects are being moved to.
    target_provider_id VARCHAR(64) NOT NULL,
    -- Human label so an operator can tell two runs apart in a list.
    name VARCHAR(128) NOT NULL,
    -- `pending` -> `running` -> `succeeded` | `failed` | `cancelled`.
    -- `pending` runs are resumable: a worker picks the remaining objects up.
    status VARCHAR(32) NOT NULL DEFAULT 'pending',
    -- Copied objects are only re-pointed by the final manifest apply, so a
    -- partially copied run never changes where any object reads from.
    objects_total BIGINT NOT NULL DEFAULT 0,
    objects_copied BIGINT NOT NULL DEFAULT 0,
    objects_failed BIGINT NOT NULL DEFAULT 0,
    bytes_copied BIGINT NOT NULL DEFAULT 0,
    -- Optional target bucket override; NULL means the target provider's bucket.
    target_bucket VARCHAR(255),
    -- When true the applied run also repoints dr_drive_storage_object rows and
    -- the provider's bindings, completing the switch atomically.
    apply_binding_switch BOOLEAN NOT NULL DEFAULT FALSE,
    failure_message TEXT,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    CONSTRAINT fk_dr_drive_storage_migration_source_provider
        FOREIGN KEY (source_provider_id) REFERENCES dr_drive_storage_provider(id) ON DELETE RESTRICT,
    CONSTRAINT fk_dr_drive_storage_migration_target_provider
        FOREIGN KEY (target_provider_id) REFERENCES dr_drive_storage_provider(id) ON DELETE RESTRICT,
    CONSTRAINT ck_dr_drive_storage_migration_distinct_providers
        CHECK (source_provider_id <> target_provider_id),
    CONSTRAINT ck_dr_drive_storage_migration_status
        CHECK (status IN ('pending', 'running', 'succeeded', 'failed', 'cancelled')),
    CONSTRAINT ck_dr_drive_storage_migration_name
        CHECK (
            name = btrim(name)
            AND length(name) BETWEEN 1 AND 128
        ),
    CONSTRAINT ck_dr_drive_storage_migration_target_bucket
        CHECK (target_bucket IS NULL OR (
            target_bucket = btrim(target_bucket)
            AND length(target_bucket) BETWEEN 1 AND 255
            AND target_bucket ~ '^[A-Za-z0-9._-]+$'
        )),
    CONSTRAINT ck_dr_drive_storage_migration_counts
        CHECK (
            objects_total >= 0
            AND objects_copied >= 0
            AND objects_failed >= 0
            AND bytes_copied >= 0
        )
);

-- Per-object progress. One row per (migration, storage object) pair so a
-- resumed run can skip what already landed and an operator can see exactly
-- which keys are outstanding.
CREATE TABLE IF NOT EXISTS dr_drive_storage_migration_item (
    id VARCHAR(64) PRIMARY KEY,
    migration_id VARCHAR(64) NOT NULL,
    tenant_id VARCHAR(64) NOT NULL,
    storage_object_id VARCHAR(64) NOT NULL,
    -- Denormalized provider ids. A queue entry must stay self-describing: a
    -- retry after the run header is edited, or after the object is repointed,
    -- still has to know which two providers the copy was planned between.
    source_provider_id VARCHAR(64) NOT NULL,
    source_bucket VARCHAR(255) NOT NULL,
    source_object_key TEXT NOT NULL,
    target_provider_id VARCHAR(64) NOT NULL,
    target_bucket VARCHAR(255) NOT NULL,
    target_object_key TEXT NOT NULL,
    -- Needed to write the target object with the same content type, otherwise a
    -- migrated download would change its media type.
    content_type VARCHAR(255) NOT NULL,
    content_length BIGINT NOT NULL,
    -- SHA-256 of the source bytes, carried so verification compares against the
    -- recorded truth rather than re-reading the source.
    checksum_sha256_hex VARCHAR(255) NOT NULL,
    -- `pending` -> `copied` | `failed`. `copied` is terminal for the object
    -- because the copy is verified before the row is written.
    status VARCHAR(32) NOT NULL DEFAULT 'pending',
    -- Checksum observed on the target after the copy; NULL until verified.
    verified_checksum_sha256_hex VARCHAR(255),
    failure_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_storage_migration_item_migration
        FOREIGN KEY (migration_id) REFERENCES dr_drive_storage_migration(id) ON DELETE CASCADE,
    CONSTRAINT fk_dr_drive_storage_migration_item_object
        FOREIGN KEY (storage_object_id) REFERENCES dr_drive_storage_object(id) ON DELETE CASCADE,
    CONSTRAINT uq_dr_drive_storage_migration_item_object
        UNIQUE (migration_id, storage_object_id),
    CONSTRAINT ck_dr_drive_storage_migration_item_status
        CHECK (status IN ('pending', 'copied', 'failed')),
    CONSTRAINT ck_dr_drive_storage_migration_item_content_length
        CHECK (content_length >= 0)
);

CREATE INDEX IF NOT EXISTS idx_dr_drive_storage_migration_tenant_created
    ON dr_drive_storage_migration (tenant_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_dr_drive_storage_migration_status
    ON dr_drive_storage_migration (status);

CREATE INDEX IF NOT EXISTS idx_dr_drive_storage_migration_item_pending
    ON dr_drive_storage_migration_item (migration_id, status);

COMMIT;
