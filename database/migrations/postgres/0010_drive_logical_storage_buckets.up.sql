-- sdkwork:migration
-- id: 0010_drive_logical_storage_buckets
-- engine: postgres
-- module: sdkwork-drive
-- purpose: Add the logical storage plane (logical buckets, provider bindings, business bindings).
-- reversible: true
-- rollback: down-migration
-- transactional: true
-- lock: exclusive
-- lock_timeout: 2s
-- statement_timeout: 30s
-- Logical storage buckets and the two bindings that sit on top of the
-- provider/bucket physical plane:
--
--   business scope -> logical bucket -> provider configuration -> physical bucket
--
-- Business rows never name a provider. Switching the provider account (or the
-- vendor) is one edit on the logical bucket, and every business that routes to
-- it follows. This is what lets a fresh install seed all business routes and an
-- operator finish setup by editing only the provider account credentials.

CREATE TABLE IF NOT EXISTS dr_drive_logical_bucket (
    id VARCHAR(192) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    bucket_key VARCHAR(64) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    object_prefix TEXT NOT NULL DEFAULT '',
    lifecycle_status VARCHAR(32) NOT NULL DEFAULT 'active',
    version BIGINT NOT NULL DEFAULT 1,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT ck_dr_drive_logical_bucket_key
        CHECK (
            bucket_key = btrim(bucket_key)
            AND bucket_key ~ '^[a-z][a-z0-9_-]{1,63}$'
        ),
    CONSTRAINT ck_dr_drive_logical_bucket_display_name
        CHECK (
            display_name = btrim(display_name)
            AND char_length(display_name) BETWEEN 1 AND 255
        ),
    CONSTRAINT ck_dr_drive_logical_bucket_object_prefix
        CHECK (
            object_prefix = btrim(object_prefix)
            AND octet_length(object_prefix) <= 512
            AND (
                object_prefix = ''
                OR object_prefix !~ '(^/|/$|//|(^|/)[.](?:/|$)|(^|/)[.][.](?:/|$))'
            )
        ),
    CONSTRAINT ck_dr_drive_logical_bucket_lifecycle_status
        CHECK (lifecycle_status IN ('active', 'archived', 'deleted')),
    CONSTRAINT ck_dr_drive_logical_bucket_version
        CHECK (version >= 1)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_logical_bucket_tenant_key
    ON dr_drive_logical_bucket (tenant_id, bucket_key)
    WHERE lifecycle_status <> 'deleted';
CREATE INDEX IF NOT EXISTS ix_dr_drive_logical_bucket_tenant_status
    ON dr_drive_logical_bucket (tenant_id, lifecycle_status, updated_at DESC);

CREATE TABLE IF NOT EXISTS dr_drive_logical_bucket_provider_binding (
    id VARCHAR(192) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    logical_bucket_id VARCHAR(192) NOT NULL,
    provider_id VARCHAR(64) NOT NULL,
    storage_root_prefix TEXT NOT NULL,
    lifecycle_status VARCHAR(32) NOT NULL DEFAULT 'active',
    version BIGINT NOT NULL DEFAULT 1,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_logical_bucket_provider_binding_bucket
        FOREIGN KEY (logical_bucket_id) REFERENCES dr_drive_logical_bucket(id) ON DELETE CASCADE,
    CONSTRAINT fk_dr_drive_logical_bucket_provider_binding_provider
        FOREIGN KEY (provider_id) REFERENCES dr_drive_storage_provider(id) ON DELETE RESTRICT,
    CONSTRAINT ck_dr_drive_logical_bucket_provider_binding_storage_root_prefix
        CHECK (
            storage_root_prefix = btrim(storage_root_prefix)
            AND octet_length(storage_root_prefix) BETWEEN 1 AND 512
            AND storage_root_prefix !~ '(^/|/$|//|(^|/)[.](?:/|$)|(^|/)[.][.](?:/|$))'
        ),
    CONSTRAINT ck_dr_drive_logical_bucket_provider_binding_lifecycle_status
        CHECK (lifecycle_status IN ('active', 'disabled', 'deleted')),
    CONSTRAINT ck_dr_drive_logical_bucket_provider_binding_version
        CHECK (version >= 1)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_logical_bucket_provider_binding_active
    ON dr_drive_logical_bucket_provider_binding (tenant_id, logical_bucket_id)
    WHERE lifecycle_status = 'active';
CREATE INDEX IF NOT EXISTS ix_dr_drive_logical_bucket_provider_binding_provider
    ON dr_drive_logical_bucket_provider_binding (provider_id, lifecycle_status);

CREATE TABLE IF NOT EXISTS dr_drive_business_storage_binding (
    id VARCHAR(192) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    scope_type VARCHAR(32) NOT NULL,
    scope_id VARCHAR(64) NOT NULL,
    logical_bucket_id VARCHAR(192) NOT NULL,
    write_prefix TEXT NOT NULL DEFAULT '',
    lifecycle_status VARCHAR(32) NOT NULL DEFAULT 'active',
    version BIGINT NOT NULL DEFAULT 1,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_business_storage_binding_bucket
        FOREIGN KEY (logical_bucket_id) REFERENCES dr_drive_logical_bucket(id) ON DELETE RESTRICT,
    CONSTRAINT ck_dr_drive_business_storage_binding_scope_type
        CHECK (scope_type IN ('tenant', 'space_type', 'logical_scope')),
    CONSTRAINT ck_dr_drive_business_storage_binding_scope_id
        CHECK (
            scope_id = btrim(scope_id)
            AND char_length(scope_id) BETWEEN 1 AND 64
        ),
    CONSTRAINT ck_dr_drive_business_storage_binding_write_prefix
        CHECK (
            write_prefix = btrim(write_prefix)
            AND octet_length(write_prefix) <= 512
            AND (
                write_prefix = ''
                OR write_prefix !~ '(^/|/$|//|(^|/)[.](?:/|$)|(^|/)[.][.](?:/|$))'
            )
        ),
    CONSTRAINT ck_dr_drive_business_storage_binding_lifecycle_status
        CHECK (lifecycle_status IN ('active', 'disabled', 'deleted')),
    CONSTRAINT ck_dr_drive_business_storage_binding_version
        CHECK (version >= 1)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_business_storage_binding_scope_active
    ON dr_drive_business_storage_binding (tenant_id, scope_type, scope_id)
    WHERE lifecycle_status = 'active';
CREATE INDEX IF NOT EXISTS ix_dr_drive_business_storage_binding_bucket
    ON dr_drive_business_storage_binding (logical_bucket_id, lifecycle_status);
