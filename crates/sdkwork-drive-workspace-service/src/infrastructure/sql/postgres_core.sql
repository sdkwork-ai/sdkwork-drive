CREATE TABLE IF NOT EXISTS dr_drive_space (
    id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    owner_subject_type VARCHAR(32) NOT NULL,
    owner_subject_id VARCHAR(128) NOT NULL,
    space_type VARCHAR(32) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    presentation_icon VARCHAR(128),
    presentation_color VARCHAR(64),
    description TEXT,
    lifecycle_status VARCHAR(32) NOT NULL DEFAULT 'active',
    version BIGINT NOT NULL DEFAULT 1,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT ck_dr_drive_space_owner_subject_type
        CHECK (owner_subject_type IN ('user', 'group', 'organization', 'app')),
    CONSTRAINT ck_dr_drive_space_space_type
        CHECK (space_type IN ('personal', 'team', 'knowledge_base', 'ai_generated', 'git_repository', 'deployment', 'app_upload', 'im', 'rtc', 'notary', 'website')),
    CONSTRAINT ck_dr_drive_space_git_repository_owner_user
        CHECK (space_type != 'git_repository' OR owner_subject_type = 'user'),
    CONSTRAINT ck_dr_drive_space_rtc_owner_user
        CHECK (space_type != 'rtc' OR owner_subject_type = 'user'),
    CONSTRAINT ck_dr_drive_space_lifecycle_status
        CHECK (lifecycle_status IN ('active', 'archived', 'deleted')),
    CONSTRAINT ck_dr_drive_space_version
        CHECK (version >= 1)
);

DROP INDEX IF EXISTS ux_dr_drive_space_tenant_owner_type;
CREATE UNIQUE INDEX ux_dr_drive_space_tenant_owner_type
    ON dr_drive_space (tenant_id, owner_subject_type, owner_subject_id, space_type)
    WHERE space_type != 'website' AND lifecycle_status = 'active';
CREATE INDEX IF NOT EXISTS ix_dr_drive_space_tenant_status
    ON dr_drive_space (tenant_id, lifecycle_status, updated_at DESC);

CREATE TABLE IF NOT EXISTS dr_drive_space_knowledge_profile (
    space_id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    knowledge_base_id VARCHAR(128) NOT NULL,
    ingestion_policy_code VARCHAR(64) NOT NULL DEFAULT 'default',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_space_knowledge_profile_space_id
        FOREIGN KEY (space_id) REFERENCES dr_drive_space(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_space_knowledge_profile_tenant_kb
    ON dr_drive_space_knowledge_profile (tenant_id, knowledge_base_id);

CREATE TABLE IF NOT EXISTS dr_drive_space_ai_generation_profile (
    space_id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    generation_scope VARCHAR(64) NOT NULL,
    retention_policy_code VARCHAR(64) NOT NULL DEFAULT 'default',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_space_ai_generation_profile_space_id
        FOREIGN KEY (space_id) REFERENCES dr_drive_space(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS ix_dr_drive_space_ai_generation_profile_tenant_scope
    ON dr_drive_space_ai_generation_profile (tenant_id, generation_scope);

CREATE TABLE IF NOT EXISTS dr_drive_space_app_upload_profile (
    space_id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    app_id VARCHAR(128) NOT NULL,
    app_resource_type VARCHAR(64) NOT NULL,
    app_resource_id VARCHAR(128) NOT NULL,
    upload_policy_code VARCHAR(64) NOT NULL DEFAULT 'default',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_space_app_upload_profile_space_id
        FOREIGN KEY (space_id) REFERENCES dr_drive_space(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_space_app_upload_profile_binding
    ON dr_drive_space_app_upload_profile (
        tenant_id,
        app_id,
        app_resource_type,
        app_resource_id
    );

CREATE TABLE IF NOT EXISTS dr_drive_space_rtc_profile (
    space_id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    user_id VARCHAR(128) NOT NULL,
    retention_policy_code VARCHAR(64) NOT NULL DEFAULT 'default',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_space_rtc_profile_space_id
        FOREIGN KEY (space_id) REFERENCES dr_drive_space(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_space_rtc_profile_user
    ON dr_drive_space_rtc_profile (tenant_id, user_id);

CREATE TABLE IF NOT EXISTS dr_drive_node (
    id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    space_id VARCHAR(64) NOT NULL,
    space_type VARCHAR(32) NOT NULL DEFAULT 'personal',
    parent_node_id VARCHAR(64),
    shortcut_target_node_id VARCHAR(64),
    node_type VARCHAR(32) NOT NULL,
    node_name VARCHAR(255) NOT NULL,
    scene VARCHAR(128),
    source VARCHAR(128),
    content_state VARCHAR(32) NOT NULL DEFAULT 'empty',
    file_extension VARCHAR(64),
    head_content_type VARCHAR(255),
    head_content_type_group VARCHAR(64),
    head_content_length BIGINT,
    head_version_no BIGINT,
    head_checksum_sha256_hex VARCHAR(255),
    lifecycle_status VARCHAR(32) NOT NULL DEFAULT 'active',
    version BIGINT NOT NULL DEFAULT 1,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_node_space_id
        FOREIGN KEY (space_id) REFERENCES dr_drive_space(id) ON DELETE CASCADE,
    CONSTRAINT fk_dr_drive_node_parent_node_id
        FOREIGN KEY (parent_node_id) REFERENCES dr_drive_node(id) ON DELETE SET NULL,
    CONSTRAINT fk_dr_drive_node_shortcut_target_node_id
        FOREIGN KEY (shortcut_target_node_id) REFERENCES dr_drive_node(id) ON DELETE SET NULL
        ,
    CONSTRAINT ck_dr_drive_node_node_type
        CHECK (node_type IN ('file', 'folder', 'shortcut', 'virtual_reference')),
    CONSTRAINT ck_dr_drive_node_space_type
        CHECK (space_type IN ('personal', 'team', 'knowledge_base', 'ai_generated', 'git_repository', 'deployment', 'app_upload', 'im', 'rtc', 'notary', 'website')),
    CONSTRAINT ck_dr_drive_node_scene
        CHECK (scene IS NULL OR (
            scene = btrim(scene)
            AND length(scene) BETWEEN 1 AND 128
            AND scene ~ '^[A-Za-z0-9._:@-]+$'
        )),
    CONSTRAINT ck_dr_drive_node_source
        CHECK (source IS NULL OR (
            source = btrim(source)
            AND length(source) BETWEEN 1 AND 128
            AND source ~ '^[A-Za-z0-9._:@-]+$'
        )),
    CONSTRAINT ck_dr_drive_node_content_state
        CHECK (content_state IN ('empty', 'uploading', 'ready', 'failed')),
    CONSTRAINT ck_dr_drive_node_file_extension
        CHECK (file_extension IS NULL OR (
            file_extension = lower(btrim(file_extension))
            AND length(file_extension) BETWEEN 1 AND 64
            AND file_extension ~ '^[a-z0-9]+(?:\\.[a-z0-9]+)*$'
        )),
    CONSTRAINT ck_dr_drive_node_head_content_type
        CHECK (head_content_type IS NULL OR (
            head_content_type = btrim(head_content_type)
            AND length(head_content_type) BETWEEN 3 AND 255
            AND head_content_type ~ '^[^[:space:]/]+/[^[:space:]/]+$'
        )),
    CONSTRAINT ck_dr_drive_node_head_content_type_group
        CHECK (head_content_type_group IS NULL OR head_content_type_group IN (
            'image', 'video', 'audio', 'text', 'document', 'archive', 'binary'
        )),
    CONSTRAINT ck_dr_drive_node_head_content_length
        CHECK (head_content_length IS NULL OR head_content_length >= 0),
    CONSTRAINT ck_dr_drive_node_head_version_no
        CHECK (head_version_no IS NULL OR head_version_no >= 1),
    CONSTRAINT ck_dr_drive_node_head_checksum_sha256_hex
        CHECK (head_checksum_sha256_hex IS NULL OR head_checksum_sha256_hex ~ '^sha256:[0-9a-f]{64}$'),
    CONSTRAINT ck_dr_drive_node_head_metadata_shape
        CHECK (
            (
                node_type IN ('folder', 'shortcut', 'virtual_reference')
                AND head_content_type IS NULL
                AND head_content_type_group IS NULL
                AND head_content_length IS NULL
                AND head_version_no IS NULL
                AND head_checksum_sha256_hex IS NULL
            )
            OR node_type = 'file'
        ),
    CONSTRAINT ck_dr_drive_node_file_ready_head
        CHECK (
            node_type != 'file'
            OR content_state != 'ready'
            OR (
                head_content_type IS NOT NULL
                AND head_content_type_group IS NOT NULL
                AND head_content_length IS NOT NULL
                AND head_version_no IS NOT NULL
            )
        ),
    CONSTRAINT ck_dr_drive_node_lifecycle_status
        CHECK (lifecycle_status IN ('active', 'trashed', 'deleted')),
    CONSTRAINT ck_dr_drive_node_version
        CHECK (version >= 1)
);

-- Backward compatible: older databases may have created `dr_drive_node` without head-metadata
-- columns, but current API queries select them via NODE_API_SELECT_COLUMNS.
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = current_schema() AND table_name = 'dr_drive_node' AND column_name = 'content_state'
    ) THEN
        ALTER TABLE dr_drive_node ADD COLUMN content_state VARCHAR(32) NOT NULL DEFAULT 'empty';
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = current_schema() AND table_name = 'dr_drive_node' AND column_name = 'file_extension'
    ) THEN
        ALTER TABLE dr_drive_node ADD COLUMN file_extension VARCHAR(64);
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = current_schema() AND table_name = 'dr_drive_node' AND column_name = 'head_content_type'
    ) THEN
        ALTER TABLE dr_drive_node ADD COLUMN head_content_type VARCHAR(255);
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = current_schema() AND table_name = 'dr_drive_node' AND column_name = 'head_content_type_group'
    ) THEN
        ALTER TABLE dr_drive_node ADD COLUMN head_content_type_group VARCHAR(64);
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = current_schema() AND table_name = 'dr_drive_node' AND column_name = 'head_content_length'
    ) THEN
        ALTER TABLE dr_drive_node ADD COLUMN head_content_length BIGINT;
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = current_schema() AND table_name = 'dr_drive_node' AND column_name = 'head_version_no'
    ) THEN
        ALTER TABLE dr_drive_node ADD COLUMN head_version_no BIGINT;
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = current_schema() AND table_name = 'dr_drive_node' AND column_name = 'head_checksum_sha256_hex'
    ) THEN
        ALTER TABLE dr_drive_node ADD COLUMN head_checksum_sha256_hex VARCHAR(255);
    END IF;
END $$;

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_node_root_name_live
    ON dr_drive_node (tenant_id, space_id, node_name)
    WHERE parent_node_id IS NULL AND lifecycle_status = 'active';
CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_node_child_name_live
    ON dr_drive_node (tenant_id, space_id, parent_node_id, node_name)
    WHERE parent_node_id IS NOT NULL AND lifecycle_status = 'active';
CREATE INDEX IF NOT EXISTS ix_dr_drive_node_space_parent
    ON dr_drive_node (tenant_id, space_id, parent_node_id, updated_at DESC);
CREATE INDEX IF NOT EXISTS ix_dr_drive_node_space_type_parent
    ON dr_drive_node (tenant_id, space_type, space_id, parent_node_id, lifecycle_status, updated_at DESC);
CREATE INDEX IF NOT EXISTS ix_dr_drive_node_shortcut_target
    ON dr_drive_node (tenant_id, shortcut_target_node_id, lifecycle_status);
CREATE INDEX IF NOT EXISTS ix_dr_drive_node_asset_list
    ON dr_drive_node (tenant_id, node_type, lifecycle_status, updated_at DESC, id);
CREATE INDEX IF NOT EXISTS ix_dr_drive_node_asset_scene_source
    ON dr_drive_node (tenant_id, node_type, scene, source, lifecycle_status, updated_at DESC, id);
CREATE INDEX IF NOT EXISTS ix_dr_drive_node_space_parent_type
    ON dr_drive_node (tenant_id, space_id, parent_node_id, head_content_type_group, updated_at DESC)
    WHERE lifecycle_status = 'active' AND node_type = 'file';

CREATE OR REPLACE FUNCTION dr_drive_sync_node_space_type()
RETURNS TRIGGER AS $$
BEGIN
    SELECT space_type
      INTO NEW.space_type
      FROM dr_drive_space
     WHERE tenant_id = NEW.tenant_id
       AND id = NEW.space_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS tr_dr_drive_node_space_type_sync_insert ON dr_drive_node;
CREATE TRIGGER tr_dr_drive_node_space_type_sync_insert
BEFORE INSERT ON dr_drive_node
FOR EACH ROW
EXECUTE FUNCTION dr_drive_sync_node_space_type();

DROP TRIGGER IF EXISTS tr_dr_drive_node_space_type_sync_update ON dr_drive_node;
CREATE TRIGGER tr_dr_drive_node_space_type_sync_update
BEFORE UPDATE OF tenant_id, space_id, space_type ON dr_drive_node
FOR EACH ROW
EXECUTE FUNCTION dr_drive_sync_node_space_type();

CREATE TABLE IF NOT EXISTS dr_drive_website_root (
    id VARCHAR(64) PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id VARCHAR(64) NOT NULL,
    space_id VARCHAR(64) NOT NULL,
    root_key VARCHAR(128) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    source_root_mode VARCHAR(32) NOT NULL,
    selected_folder_node_id VARCHAR(64),
    selector_key VARCHAR(160) NOT NULL,
    content_mode VARCHAR(32) NOT NULL,
    active_node_id VARCHAR(64) NOT NULL,
    active_generation BIGINT NOT NULL DEFAULT 1,
    root_status VARCHAR(32) NOT NULL DEFAULT 'active',
    last_switch_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_switch_by VARCHAR(128) NOT NULL,
    version BIGINT NOT NULL DEFAULT 1,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_website_root_space_id
        FOREIGN KEY (space_id) REFERENCES dr_drive_space(id) ON DELETE CASCADE,
    CONSTRAINT fk_dr_drive_website_root_selected_folder_node_id
        FOREIGN KEY (selected_folder_node_id) REFERENCES dr_drive_node(id) ON DELETE RESTRICT,
    CONSTRAINT fk_dr_drive_website_root_active_node_id
        FOREIGN KEY (active_node_id) REFERENCES dr_drive_node(id) ON DELETE RESTRICT,
    CONSTRAINT ck_dr_drive_website_root_source_root_mode
        CHECK (source_root_mode IN ('space_root', 'folder')),
    CONSTRAINT ck_dr_drive_website_root_selector
        CHECK (
            (source_root_mode = 'space_root' AND selected_folder_node_id IS NULL AND selector_key = 'space_root')
            OR
            (source_root_mode = 'folder' AND selected_folder_node_id IS NOT NULL AND selector_key = 'folder:' || selected_folder_node_id)
        ),
    CONSTRAINT ck_dr_drive_website_root_content_mode
        CHECK (content_mode IN ('live_tree', 'atomic_generation')),
    CONSTRAINT ck_dr_drive_website_root_active_generation
        CHECK (active_generation >= 1),
    CONSTRAINT ck_dr_drive_website_root_status
        CHECK (root_status IN ('active', 'suspended', 'archived', 'invalid')),
    CONSTRAINT ck_dr_drive_website_root_version
        CHECK (version >= 1)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_website_root_uuid
    ON dr_drive_website_root (uuid);
CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_website_root_space_key
    ON dr_drive_website_root (tenant_id, space_id, root_key);
CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_website_root_selector_active
    ON dr_drive_website_root (tenant_id, space_id, selector_key)
    WHERE root_status IN ('active', 'suspended', 'invalid');
CREATE INDEX IF NOT EXISTS ix_dr_drive_website_root_active_node
    ON dr_drive_website_root (tenant_id, space_id, active_node_id, root_status);

CREATE TABLE IF NOT EXISTS dr_drive_website_root_generation (
    id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    website_root_id VARCHAR(64) NOT NULL,
    generation_no BIGINT NOT NULL,
    root_node_id VARCHAR(64) NOT NULL,
    source_sync_id VARCHAR(64),
    manifest_sha256 VARCHAR(71),
    file_count BIGINT NOT NULL DEFAULT 0,
    total_bytes BIGINT NOT NULL DEFAULT 0,
    generation_status VARCHAR(32) NOT NULL,
    activated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    activated_by VARCHAR(128) NOT NULL,
    retention_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_website_root_generation_root_id
        FOREIGN KEY (website_root_id) REFERENCES dr_drive_website_root(id) ON DELETE CASCADE,
    CONSTRAINT fk_dr_drive_website_root_generation_node_id
        FOREIGN KEY (root_node_id) REFERENCES dr_drive_node(id) ON DELETE RESTRICT,
    CONSTRAINT ck_dr_drive_website_root_generation_no
        CHECK (generation_no >= 1),
    CONSTRAINT ck_dr_drive_website_root_generation_counts
        CHECK (file_count >= 0 AND total_bytes >= 0),
    CONSTRAINT ck_dr_drive_website_root_generation_manifest
        CHECK (manifest_sha256 IS NULL OR manifest_sha256 ~ '^sha256:[0-9a-f]{64}$'),
    CONSTRAINT ck_dr_drive_website_root_generation_status
        CHECK (generation_status IN ('current', 'retained', 'expired', 'deleting', 'deleted', 'invalid'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_website_root_generation_no
    ON dr_drive_website_root_generation (website_root_id, generation_no);
CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_website_root_generation_current
    ON dr_drive_website_root_generation (website_root_id)
    WHERE generation_status = 'current';
CREATE INDEX IF NOT EXISTS ix_dr_drive_website_root_generation_retention
    ON dr_drive_website_root_generation (generation_status, retention_until);

CREATE TABLE IF NOT EXISTS dr_drive_space_website_profile (
    space_id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    project_key VARCHAR(128) NOT NULL,
    default_root_id VARCHAR(64) NOT NULL,
    case_collision_policy VARCHAR(32) NOT NULL DEFAULT 'reject',
    retained_generation_count INT NOT NULL DEFAULT 3,
    sync_policy VARCHAR(32) NOT NULL DEFAULT 'ordinary',
    profile_status VARCHAR(32) NOT NULL DEFAULT 'active',
    version BIGINT NOT NULL DEFAULT 1,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_space_website_profile_space_id
        FOREIGN KEY (space_id) REFERENCES dr_drive_space(id) ON DELETE CASCADE,
    CONSTRAINT fk_dr_drive_space_website_profile_default_root_id
        FOREIGN KEY (default_root_id) REFERENCES dr_drive_website_root(id) ON DELETE RESTRICT,
    CONSTRAINT ck_dr_drive_space_website_profile_case_policy
        CHECK (case_collision_policy IN ('reject')),
    CONSTRAINT ck_dr_drive_space_website_profile_retained_count
        CHECK (retained_generation_count BETWEEN 1 AND 100),
    CONSTRAINT ck_dr_drive_space_website_profile_sync_policy
        CHECK (sync_policy IN ('ordinary', 'atomic_recommended', 'atomic_required')),
    CONSTRAINT ck_dr_drive_space_website_profile_status
        CHECK (profile_status IN ('active', 'suspended', 'archived')),
    CONSTRAINT ck_dr_drive_space_website_profile_version
        CHECK (version >= 1)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_space_website_profile_project
    ON dr_drive_space_website_profile (tenant_id, project_key);

CREATE TABLE IF NOT EXISTS dr_drive_website_sync (
    id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    website_root_id VARCHAR(64) NOT NULL,
    space_id VARCHAR(64) NOT NULL,
    idempotency_key VARCHAR(128) NOT NULL,
    expected_root_version BIGINT NOT NULL,
    expected_generation BIGINT NOT NULL,
    staging_node_id VARCHAR(64) NOT NULL,
    manifest_sha256 VARCHAR(71) NOT NULL,
    manifest_file_count BIGINT NOT NULL,
    manifest_total_bytes BIGINT NOT NULL,
    uploaded_file_count BIGINT NOT NULL DEFAULT 0,
    uploaded_total_bytes BIGINT NOT NULL DEFAULT 0,
    sync_status VARCHAR(32) NOT NULL,
    lease_owner VARCHAR(128),
    lease_token VARCHAR(128),
    lease_expires_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL,
    validated_at TIMESTAMPTZ,
    activated_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error_code VARCHAR(64),
    error_summary VARCHAR(1024),
    version BIGINT NOT NULL DEFAULT 1,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_website_sync_root_id
        FOREIGN KEY (website_root_id) REFERENCES dr_drive_website_root(id) ON DELETE CASCADE,
    CONSTRAINT fk_dr_drive_website_sync_space_id
        FOREIGN KEY (space_id) REFERENCES dr_drive_space(id) ON DELETE CASCADE,
    CONSTRAINT fk_dr_drive_website_sync_staging_node_id
        FOREIGN KEY (staging_node_id) REFERENCES dr_drive_node(id) ON DELETE RESTRICT,
    CONSTRAINT ck_dr_drive_website_sync_expected
        CHECK (expected_root_version >= 1 AND expected_generation >= 1),
    CONSTRAINT ck_dr_drive_website_sync_manifest
        CHECK (manifest_sha256 ~ '^sha256:[0-9a-f]{64}$'),
    CONSTRAINT ck_dr_drive_website_sync_counts
        CHECK (
            manifest_file_count >= 0 AND manifest_total_bytes >= 0
            AND uploaded_file_count >= 0 AND uploaded_total_bytes >= 0
        ),
    CONSTRAINT ck_dr_drive_website_sync_status
        CHECK (sync_status IN ('created', 'uploading', 'ready', 'validating', 'active', 'completed', 'failed', 'aborted', 'expired')),
    CONSTRAINT ck_dr_drive_website_sync_lease
        CHECK ((lease_owner IS NULL AND lease_token IS NULL AND lease_expires_at IS NULL) OR (lease_owner IS NOT NULL AND lease_token IS NOT NULL AND lease_expires_at IS NOT NULL)),
    CONSTRAINT ck_dr_drive_website_sync_version
        CHECK (version >= 1)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_website_sync_idempotency
    ON dr_drive_website_sync (tenant_id, idempotency_key);
CREATE INDEX IF NOT EXISTS ix_dr_drive_website_sync_worker
    ON dr_drive_website_sync (sync_status, lease_expires_at, updated_at);
CREATE INDEX IF NOT EXISTS ix_dr_drive_website_sync_root_status
    ON dr_drive_website_sync (tenant_id, website_root_id, sync_status, created_at DESC);

CREATE TABLE IF NOT EXISTS dr_drive_root_scope_subscription (
    id VARCHAR(64) PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id VARCHAR(64) NOT NULL,
    space_id VARCHAR(64) NOT NULL,
    consumer_kind VARCHAR(32) NOT NULL,
    consumer_resource_id VARCHAR(128) NOT NULL,
    root_node_id VARCHAR(64) NOT NULL,
    scope_status VARCHAR(32) NOT NULL DEFAULT 'active',
    version BIGINT NOT NULL DEFAULT 1,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_root_scope_subscription_space_id
        FOREIGN KEY (space_id) REFERENCES dr_drive_space(id) ON DELETE CASCADE,
    CONSTRAINT fk_dr_drive_root_scope_subscription_node_id
        FOREIGN KEY (root_node_id) REFERENCES dr_drive_node(id) ON DELETE RESTRICT,
    CONSTRAINT ck_dr_drive_root_scope_subscription_kind
        CHECK (consumer_kind IN ('knowledgebase_raw')),
    CONSTRAINT ck_dr_drive_root_scope_subscription_status
        CHECK (scope_status IN ('active', 'suspended', 'revoked')),
    CONSTRAINT ck_dr_drive_root_scope_subscription_version
        CHECK (version >= 1)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_root_scope_subscription_uuid
    ON dr_drive_root_scope_subscription (uuid);
CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_root_scope_subscription_consumer
    ON dr_drive_root_scope_subscription (tenant_id, consumer_kind, consumer_resource_id)
    WHERE scope_status IN ('active', 'suspended');
CREATE INDEX IF NOT EXISTS ix_dr_drive_root_scope_subscription_root
    ON dr_drive_root_scope_subscription (tenant_id, space_id, root_node_id, scope_status);

CREATE TABLE IF NOT EXISTS dr_drive_node_permission (
    id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    node_id VARCHAR(64) NOT NULL,
    subject_type VARCHAR(32) NOT NULL,
    subject_id VARCHAR(128) NOT NULL,
    role VARCHAR(32) NOT NULL,
    inherited BOOLEAN NOT NULL DEFAULT FALSE,
    lifecycle_status VARCHAR(32) NOT NULL DEFAULT 'active',
    version BIGINT NOT NULL DEFAULT 1,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_node_permission_node_id
        FOREIGN KEY (node_id) REFERENCES dr_drive_node(id) ON DELETE CASCADE
        ,
    CONSTRAINT ck_dr_drive_node_permission_subject_type
        CHECK (subject_type IN ('user', 'group', 'domain', 'app')),
    CONSTRAINT ck_dr_drive_node_permission_role
        CHECK (role IN ('reader', 'commenter', 'writer', 'owner')),
    CONSTRAINT ck_dr_drive_node_permission_lifecycle_status
        CHECK (lifecycle_status IN ('active', 'deleted')),
    CONSTRAINT ck_dr_drive_node_permission_version
        CHECK (version >= 1)
);

CREATE INDEX IF NOT EXISTS ix_dr_drive_node_permission_resource
    ON dr_drive_node_permission (tenant_id, node_id, lifecycle_status);
CREATE INDEX IF NOT EXISTS ix_dr_drive_node_permission_subject
    ON dr_drive_node_permission (tenant_id, subject_type, subject_id, lifecycle_status);
CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_node_permission_node_subject_live
    ON dr_drive_node_permission (tenant_id, node_id, subject_type, subject_id)
    WHERE lifecycle_status = 'active';

CREATE TABLE IF NOT EXISTS dr_drive_node_share_link (
    id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    node_id VARCHAR(64) NOT NULL,
    token_hash VARCHAR(80) NOT NULL,
    access_code_hash VARCHAR(80),
    role VARCHAR(32) NOT NULL,
    expires_at_epoch_ms BIGINT,
    download_limit BIGINT,
    download_count BIGINT NOT NULL DEFAULT 0,
    lifecycle_status VARCHAR(32) NOT NULL DEFAULT 'active',
    version BIGINT NOT NULL DEFAULT 1,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_node_share_link_node_id
        FOREIGN KEY (node_id) REFERENCES dr_drive_node(id) ON DELETE CASCADE
        ,
    CONSTRAINT ck_dr_drive_node_share_link_role
        CHECK (role IN ('reader', 'commenter', 'writer')),
    CONSTRAINT ck_dr_drive_node_share_link_expires_at_epoch_ms
        CHECK (expires_at_epoch_ms IS NULL OR expires_at_epoch_ms > 0),
    CONSTRAINT ck_dr_drive_node_share_link_download_limit
        CHECK (download_limit IS NULL OR download_limit >= 0),
    CONSTRAINT ck_dr_drive_node_share_link_download_count
        CHECK (download_count >= 0),
    CONSTRAINT ck_dr_drive_node_share_link_lifecycle_status
        CHECK (lifecycle_status IN ('active', 'deleted')),
    CONSTRAINT ck_dr_drive_node_share_link_version
        CHECK (version >= 1)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_node_share_link_token_hash
    ON dr_drive_node_share_link (token_hash);
CREATE INDEX IF NOT EXISTS ix_dr_drive_node_share_link_resource
    ON dr_drive_node_share_link (tenant_id, node_id, lifecycle_status);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = current_schema()
          AND table_name = 'dr_drive_node_share_link'
          AND column_name = 'access_code_hash'
    ) THEN
        ALTER TABLE dr_drive_node_share_link ADD COLUMN access_code_hash VARCHAR(80);
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS dr_drive_node_comment (
    id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    node_id VARCHAR(64) NOT NULL,
    content TEXT NOT NULL,
    anchor TEXT,
    resolved BOOLEAN NOT NULL DEFAULT FALSE,
    lifecycle_status VARCHAR(32) NOT NULL DEFAULT 'active',
    version BIGINT NOT NULL DEFAULT 1,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_node_comment_node_id
        FOREIGN KEY (node_id) REFERENCES dr_drive_node(id) ON DELETE CASCADE
        ,
    CONSTRAINT ck_dr_drive_node_comment_lifecycle_status
        CHECK (lifecycle_status IN ('active', 'deleted')),
    CONSTRAINT ck_dr_drive_node_comment_version
        CHECK (version >= 1)
);

CREATE INDEX IF NOT EXISTS ix_dr_drive_node_comment_node
    ON dr_drive_node_comment (tenant_id, node_id, lifecycle_status, created_at DESC);
CREATE INDEX IF NOT EXISTS ix_dr_drive_node_comment_resolved
    ON dr_drive_node_comment (tenant_id, node_id, resolved, lifecycle_status, updated_at DESC);

CREATE TABLE IF NOT EXISTS dr_drive_node_comment_reply (
    id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    node_id VARCHAR(64) NOT NULL,
    comment_id VARCHAR(64) NOT NULL,
    content TEXT NOT NULL,
    lifecycle_status VARCHAR(32) NOT NULL DEFAULT 'active',
    version BIGINT NOT NULL DEFAULT 1,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_node_comment_reply_node_id
        FOREIGN KEY (node_id) REFERENCES dr_drive_node(id) ON DELETE CASCADE,
    CONSTRAINT fk_dr_drive_node_comment_reply_comment_id
        FOREIGN KEY (comment_id) REFERENCES dr_drive_node_comment(id) ON DELETE CASCADE
        ,
    CONSTRAINT ck_dr_drive_node_comment_reply_lifecycle_status
        CHECK (lifecycle_status IN ('active', 'deleted')),
    CONSTRAINT ck_dr_drive_node_comment_reply_version
        CHECK (version >= 1)
);

CREATE INDEX IF NOT EXISTS ix_dr_drive_node_comment_reply_comment
    ON dr_drive_node_comment_reply (tenant_id, comment_id, lifecycle_status, created_at ASC);
CREATE INDEX IF NOT EXISTS ix_dr_drive_node_comment_reply_node
    ON dr_drive_node_comment_reply (tenant_id, node_id, lifecycle_status, created_at ASC);

CREATE TABLE IF NOT EXISTS dr_drive_node_favorite (
    id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    node_id VARCHAR(64) NOT NULL,
    subject_type VARCHAR(32) NOT NULL,
    subject_id VARCHAR(128) NOT NULL,
    lifecycle_status VARCHAR(32) NOT NULL DEFAULT 'active',
    version BIGINT NOT NULL DEFAULT 1,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_node_favorite_node_id
        FOREIGN KEY (node_id) REFERENCES dr_drive_node(id) ON DELETE CASCADE
        ,
    CONSTRAINT ck_dr_drive_node_favorite_subject_type
        CHECK (subject_type IN ('user', 'group', 'domain', 'app')),
    CONSTRAINT ck_dr_drive_node_favorite_lifecycle_status
        CHECK (lifecycle_status IN ('active', 'deleted')),
    CONSTRAINT ck_dr_drive_node_favorite_version
        CHECK (version >= 1)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_node_favorite_subject_node
    ON dr_drive_node_favorite (tenant_id, subject_type, subject_id, node_id);
CREATE INDEX IF NOT EXISTS ix_dr_drive_node_favorite_subject
    ON dr_drive_node_favorite (tenant_id, subject_type, subject_id, lifecycle_status, updated_at DESC);

CREATE TABLE IF NOT EXISTS dr_drive_node_property (
    id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    node_id VARCHAR(64) NOT NULL,
    property_key VARCHAR(128) NOT NULL,
    property_value TEXT NOT NULL,
    visibility VARCHAR(32) NOT NULL,
    lifecycle_status VARCHAR(32) NOT NULL DEFAULT 'active',
    version BIGINT NOT NULL DEFAULT 1,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_node_property_node_id
        FOREIGN KEY (node_id) REFERENCES dr_drive_node(id) ON DELETE CASCADE
        ,
    CONSTRAINT ck_dr_drive_node_property_visibility
        CHECK (visibility IN ('private', 'app_public')),
    CONSTRAINT ck_dr_drive_node_property_lifecycle_status
        CHECK (lifecycle_status IN ('active', 'deleted')),
    CONSTRAINT ck_dr_drive_node_property_version
        CHECK (version >= 1)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_node_property_key
    ON dr_drive_node_property (tenant_id, node_id, property_key, visibility);
CREATE INDEX IF NOT EXISTS ix_dr_drive_node_property_node
    ON dr_drive_node_property (tenant_id, node_id, visibility, lifecycle_status, property_key ASC);

CREATE TABLE IF NOT EXISTS dr_drive_label (
    id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    label_key VARCHAR(128) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    color VARCHAR(32),
    description TEXT,
    lifecycle_status VARCHAR(32) NOT NULL DEFAULT 'active',
    version BIGINT NOT NULL DEFAULT 1,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT ck_dr_drive_label_lifecycle_status
        CHECK (lifecycle_status IN ('active', 'deleted')),
    CONSTRAINT ck_dr_drive_label_version
        CHECK (version >= 1)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_label_key
    ON dr_drive_label (tenant_id, label_key);
CREATE INDEX IF NOT EXISTS ix_dr_drive_label_tenant_status
    ON dr_drive_label (tenant_id, lifecycle_status, label_key ASC);

CREATE TABLE IF NOT EXISTS dr_drive_tenant_quota (
    tenant_id VARCHAR(64) PRIMARY KEY,
    max_bytes BIGINT,
    updated_by VARCHAR(128) NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version BIGINT NOT NULL DEFAULT 1,
    CONSTRAINT ck_dr_drive_tenant_quota_version
        CHECK (version >= 1),
    CONSTRAINT ck_dr_drive_tenant_quota_max_bytes
        CHECK (max_bytes IS NULL OR max_bytes > 0)
);

CREATE TABLE IF NOT EXISTS dr_drive_node_label (
    id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    node_id VARCHAR(64) NOT NULL,
    label_id VARCHAR(64) NOT NULL,
    lifecycle_status VARCHAR(32) NOT NULL DEFAULT 'active',
    version BIGINT NOT NULL DEFAULT 1,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_node_label_node_id
        FOREIGN KEY (node_id) REFERENCES dr_drive_node(id) ON DELETE CASCADE,
    CONSTRAINT fk_dr_drive_node_label_label_id
        FOREIGN KEY (label_id) REFERENCES dr_drive_label(id) ON DELETE CASCADE
        ,
    CONSTRAINT ck_dr_drive_node_label_lifecycle_status
        CHECK (lifecycle_status IN ('active', 'deleted')),
    CONSTRAINT ck_dr_drive_node_label_version
        CHECK (version >= 1)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_node_label_node_label
    ON dr_drive_node_label (tenant_id, node_id, label_id);
CREATE INDEX IF NOT EXISTS ix_dr_drive_node_label_node
    ON dr_drive_node_label (tenant_id, node_id, lifecycle_status, label_id ASC);
CREATE INDEX IF NOT EXISTS ix_dr_drive_node_label_label
    ON dr_drive_node_label (tenant_id, label_id, lifecycle_status, node_id ASC);

CREATE TABLE IF NOT EXISTS dr_drive_watch_channel (
    id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    space_id VARCHAR(64),
    node_id VARCHAR(64),
    resource_type VARCHAR(32) NOT NULL,
    resource_id VARCHAR(64),
    channel_type VARCHAR(32) NOT NULL DEFAULT 'web_hook',
    address TEXT NOT NULL,
    token_hash VARCHAR(64),
    expiration_epoch_ms BIGINT NOT NULL,
    lifecycle_status VARCHAR(32) NOT NULL DEFAULT 'active',
    version BIGINT NOT NULL DEFAULT 1,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_watch_channel_space_id
        FOREIGN KEY (space_id) REFERENCES dr_drive_space(id) ON DELETE CASCADE,
    CONSTRAINT fk_dr_drive_watch_channel_node_id
        FOREIGN KEY (node_id) REFERENCES dr_drive_node(id) ON DELETE CASCADE
        ,
    CONSTRAINT ck_dr_drive_watch_channel_resource_type
        CHECK (resource_type IN ('changes', 'node')),
    CONSTRAINT ck_dr_drive_watch_channel_channel_type
        CHECK (channel_type IN ('web_hook')),
    CONSTRAINT ck_dr_drive_watch_channel_expiration_epoch_ms
        CHECK (expiration_epoch_ms > 0),
    CONSTRAINT ck_dr_drive_watch_channel_lifecycle_status
        CHECK (lifecycle_status IN ('active', 'stopped', 'expired')),
    CONSTRAINT ck_dr_drive_watch_channel_version
        CHECK (version >= 1)
);

CREATE INDEX IF NOT EXISTS ix_dr_drive_watch_channel_tenant_status
    ON dr_drive_watch_channel (tenant_id, lifecycle_status, expiration_epoch_ms ASC);
CREATE INDEX IF NOT EXISTS ix_dr_drive_watch_channel_resource
    ON dr_drive_watch_channel (tenant_id, resource_type, resource_id, lifecycle_status);
CREATE INDEX IF NOT EXISTS ix_dr_drive_watch_channel_node
    ON dr_drive_watch_channel (tenant_id, node_id, lifecycle_status);
CREATE INDEX IF NOT EXISTS ix_dr_drive_watch_channel_expires
    ON dr_drive_watch_channel (tenant_id, lifecycle_status, expiration_epoch_ms ASC);

CREATE TABLE IF NOT EXISTS dr_drive_change_cursor (
    id VARCHAR(160) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    space_id VARCHAR(64) NOT NULL,
    last_sequence_no BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT ck_dr_drive_change_cursor_last_sequence_no
        CHECK (last_sequence_no >= 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_change_cursor_scope
    ON dr_drive_change_cursor (tenant_id, space_id);

CREATE TABLE IF NOT EXISTS dr_drive_change_log (
    id BIGINT NOT NULL PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    space_id VARCHAR(64) NOT NULL,
    node_id VARCHAR(64),
    sequence_no BIGINT NOT NULL,
    event_type VARCHAR(64) NOT NULL,
    actor_id VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT ck_dr_drive_change_log_sequence_no
        CHECK (sequence_no >= 1)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_change_log_space_sequence
    ON dr_drive_change_log (tenant_id, space_id, sequence_no);
CREATE INDEX IF NOT EXISTS ix_dr_drive_change_log_tenant_space_created
    ON dr_drive_change_log (tenant_id, space_id, created_at DESC);

CREATE TABLE IF NOT EXISTS dr_drive_upload_session (
    id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    space_id VARCHAR(64) NOT NULL,
    node_id VARCHAR(64) NOT NULL,
    bucket VARCHAR(255) NOT NULL,
    object_key TEXT NOT NULL,
    idempotency_key VARCHAR(128) NOT NULL,
    storage_provider_id VARCHAR(64) NOT NULL,
    storage_upload_id TEXT NOT NULL,
    state VARCHAR(32) NOT NULL,
    expires_at_epoch_ms BIGINT NOT NULL,
    version BIGINT NOT NULL DEFAULT 1,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_upload_session_space_id
        FOREIGN KEY (space_id) REFERENCES dr_drive_space(id) ON DELETE CASCADE,
    CONSTRAINT fk_dr_drive_upload_session_node_id
        FOREIGN KEY (node_id) REFERENCES dr_drive_node(id) ON DELETE CASCADE
        ,
    CONSTRAINT ck_dr_drive_upload_session_bucket
        CHECK (
            bucket = btrim(bucket)
            AND length(bucket) BETWEEN 1 AND 255
            AND bucket ~ '^[A-Za-z0-9._-]+$'
        ),
    CONSTRAINT ck_dr_drive_upload_session_object_key
        CHECK (
            object_key = btrim(object_key)
            AND octet_length(object_key) BETWEEN 1 AND 1024
            AND object_key !~ '(^/|/$|//|(^|/)[.](?:/|$)|(^|/)[.][.](?:/|$))'
        ),
    CONSTRAINT ck_dr_drive_upload_session_expires_at_epoch_ms
        CHECK (expires_at_epoch_ms > 0),
    CONSTRAINT ck_dr_drive_upload_session_state
        CHECK (state IN ('created', 'uploading', 'completing', 'completed', 'aborted', 'expired')),
    CONSTRAINT ck_dr_drive_upload_session_version
        CHECK (version >= 1)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_upload_session_idempotency
    ON dr_drive_upload_session (tenant_id, space_id, node_id, idempotency_key);
CREATE INDEX IF NOT EXISTS ix_dr_drive_upload_session_expires
    ON dr_drive_upload_session (tenant_id, state, expires_at_epoch_ms);

CREATE TABLE IF NOT EXISTS dr_drive_storage_provider (
    id VARCHAR(64) PRIMARY KEY,
    provider_kind VARCHAR(64) NOT NULL,
    name VARCHAR(128) NOT NULL,
    endpoint_url TEXT NOT NULL,
    region VARCHAR(128),
    bucket VARCHAR(255) NOT NULL,
    path_style BOOLEAN NOT NULL DEFAULT TRUE,
    strict_tls BOOLEAN NOT NULL DEFAULT TRUE,
    credential_ref VARCHAR(255),
    server_side_encryption_mode VARCHAR(64),
    default_storage_class VARCHAR(64),
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    version BIGINT NOT NULL DEFAULT 1,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT ck_dr_drive_storage_provider_endpoint_url
        CHECK (
            endpoint_url = btrim(endpoint_url)
            AND endpoint_url !~ '[[:space:]]'
            AND (
                (provider_kind = 'local_filesystem' AND endpoint_url ~* '^file://.+')
                OR (
                    provider_kind <> 'local_filesystem'
                    AND endpoint_url ~* '^https?://[^[:space:]/?#]+'
                )
            )
        ),
    CONSTRAINT ck_dr_drive_storage_provider_bucket
        CHECK (
            bucket = btrim(bucket)
            AND (
                (
                    provider_kind = 'local_filesystem'
                    AND length(bucket) BETWEEN 1 AND 255
                    AND bucket ~ '^[A-Za-z0-9._-]+$'
                )
                OR (
                    provider_kind <> 'local_filesystem'
                    AND length(bucket) BETWEEN 3 AND 63
                    AND bucket ~ '^[a-z0-9][a-z0-9.-]*[a-z0-9]$'
                    AND bucket !~ '[.][.]'
                    AND bucket !~ '[.][-]|[-][.]'
                    AND bucket !~ '^[0-9]+[.][0-9]+[.][0-9]+[.][0-9]+$'
                    AND bucket !~ '^(xn--|sthree-)'
                    AND bucket !~ '(-s3alias|--ol-s3|[.]mrap|--x-s3)$'
                )
            )
        ),
    CONSTRAINT ck_dr_drive_storage_provider_provider_kind
        CHECK (
            provider_kind IN (
                'local_filesystem',
                's3_compatible',
                'google_cloud_storage',
                'aliyun_oss',
                'tencent_cos',
                'huawei_obs',
                'volcengine_tos'
            )
            OR provider_kind ~ '^custom:[a-z0-9_-]{2,32}$'
        ),
    CONSTRAINT ck_dr_drive_storage_provider_strict_tls
        CHECK (
            strict_tls = FALSE
            OR endpoint_url ~* '^https://.+'
            OR endpoint_url ~* '^file://.+'
        ),
    CONSTRAINT ck_dr_drive_storage_provider_status
        CHECK (status IN ('active', 'disabled', 'deleted')),
    CONSTRAINT ck_dr_drive_storage_provider_version
        CHECK (version >= 1)
);

CREATE INDEX IF NOT EXISTS ix_dr_drive_storage_provider_status
    ON dr_drive_storage_provider (status, updated_at DESC);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'fk_dr_drive_upload_session_storage_provider_id'
    ) THEN
        ALTER TABLE dr_drive_upload_session
            ADD CONSTRAINT fk_dr_drive_upload_session_storage_provider_id
            FOREIGN KEY (storage_provider_id)
            REFERENCES dr_drive_storage_provider(id)
            ON DELETE RESTRICT;
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS dr_drive_storage_provider_binding (
    id VARCHAR(192) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    space_id VARCHAR(64),
    provider_id VARCHAR(64) NOT NULL,
    binding_scope VARCHAR(32) NOT NULL,
    purpose VARCHAR(32) NOT NULL DEFAULT 'primary',
    storage_root_prefix TEXT NOT NULL,
    lifecycle_status VARCHAR(32) NOT NULL DEFAULT 'active',
    version BIGINT NOT NULL DEFAULT 1,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_storage_provider_binding_space_id
        FOREIGN KEY (space_id) REFERENCES dr_drive_space(id) ON DELETE CASCADE,
    CONSTRAINT fk_dr_drive_storage_provider_binding_provider_id
        FOREIGN KEY (provider_id) REFERENCES dr_drive_storage_provider(id) ON DELETE CASCADE
        ,
    CONSTRAINT ck_dr_drive_storage_provider_binding_scope
        CHECK (binding_scope IN ('tenant', 'space', 'space_type')),
    CONSTRAINT ck_dr_drive_storage_provider_binding_purpose
        CHECK (
            (binding_scope IN ('tenant', 'space') AND purpose = 'primary')
            OR (
                binding_scope = 'space_type'
                AND purpose IN (
                    'personal', 'team', 'knowledge_base', 'ai_generated', 'git_repository',
                    'deployment', 'app_upload', 'im', 'rtc', 'notary'
                )
            )
        ),
    CONSTRAINT ck_dr_drive_storage_provider_binding_storage_root_prefix
        CHECK (
            storage_root_prefix = btrim(storage_root_prefix)
            AND octet_length(storage_root_prefix) BETWEEN 1 AND 512
            AND storage_root_prefix !~ '(^/|/$|//|(^|/)[.](?:/|$)|(^|/)[.][.](?:/|$))'
        ),
    CONSTRAINT ck_dr_drive_storage_provider_binding_lifecycle_status
        CHECK (lifecycle_status IN ('active', 'disabled', 'deleted')),
    CONSTRAINT ck_dr_drive_storage_provider_binding_version
        CHECK (version >= 1),
    CONSTRAINT ck_dr_drive_storage_provider_binding_scope_space
        CHECK (
            (binding_scope = 'tenant' AND space_id IS NULL)
            OR (binding_scope = 'space' AND space_id IS NOT NULL)
            OR (binding_scope = 'space_type' AND space_id IS NULL)
        )
);

CREATE INDEX IF NOT EXISTS ix_dr_drive_storage_provider_binding_lookup
    ON dr_drive_storage_provider_binding (tenant_id, space_id, purpose, lifecycle_status);
CREATE INDEX IF NOT EXISTS ix_dr_drive_storage_provider_binding_provider
    ON dr_drive_storage_provider_binding (provider_id, lifecycle_status);
CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_storage_provider_binding_tenant_primary_active
    ON dr_drive_storage_provider_binding (tenant_id, purpose)
    WHERE space_id IS NULL AND purpose = 'primary' AND lifecycle_status = 'active';
CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_storage_provider_binding_space_primary_active
    ON dr_drive_storage_provider_binding (tenant_id, space_id, purpose)
    WHERE space_id IS NOT NULL AND purpose = 'primary' AND lifecycle_status = 'active';
CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_storage_provider_binding_space_type_active
    ON dr_drive_storage_provider_binding (tenant_id, purpose)
    WHERE binding_scope = 'space_type' AND lifecycle_status = 'active';

CREATE TABLE IF NOT EXISTS dr_drive_storage_provider_kind (
    provider_kind VARCHAR(64) PRIMARY KEY,
    display_name VARCHAR(128) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    version BIGINT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT ck_dr_drive_storage_provider_kind_provider_kind
        CHECK (
            provider_kind IN (
                'local_filesystem',
                's3_compatible',
                'google_cloud_storage',
                'aliyun_oss',
                'tencent_cos',
                'huawei_obs',
                'volcengine_tos'
            )
        ),
    CONSTRAINT ck_dr_drive_storage_provider_kind_display_name
        CHECK (display_name = btrim(display_name) AND length(display_name) BETWEEN 1 AND 128),
    CONSTRAINT ck_dr_drive_storage_provider_kind_version
        CHECK (version >= 1)
);

INSERT INTO dr_drive_storage_provider_kind (provider_kind, display_name, enabled, sort_order)
VALUES
    ('local_filesystem', 'Local Filesystem', TRUE, 1),
    ('s3_compatible', 'Amazon S3 / S3 Compatible', TRUE, 2),
    ('google_cloud_storage', 'Google Cloud Storage', TRUE, 3),
    ('aliyun_oss', 'Alibaba Cloud OSS', TRUE, 4),
    ('tencent_cos', 'Tencent Cloud COS', TRUE, 5),
    ('huawei_obs', 'Huawei Cloud OBS', TRUE, 6),
    ('volcengine_tos', 'Volcengine TOS', TRUE, 7)
ON CONFLICT (provider_kind) DO NOTHING;

CREATE TABLE IF NOT EXISTS dr_drive_download_package (
    id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    package_name VARCHAR(255) NOT NULL,
    state VARCHAR(32) NOT NULL,
    storage_provider_id VARCHAR(64) NOT NULL,
    bucket VARCHAR(255) NOT NULL,
    archive_object_key TEXT NOT NULL,
    content_type VARCHAR(255) NOT NULL DEFAULT 'application/zip',
    file_count BIGINT NOT NULL,
    total_bytes BIGINT NOT NULL,
    archive_size_bytes BIGINT NOT NULL,
    requested_node_ids_json TEXT NOT NULL,
    item_manifest_json TEXT NOT NULL,
    expires_at_epoch_ms BIGINT NOT NULL,
    error_message TEXT,
    version BIGINT NOT NULL DEFAULT 1,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_download_package_storage_provider_id
        FOREIGN KEY (storage_provider_id) REFERENCES dr_drive_storage_provider(id) ON DELETE RESTRICT
        ,
    CONSTRAINT ck_dr_drive_download_package_bucket
        CHECK (
            bucket = btrim(bucket)
            AND length(bucket) BETWEEN 1 AND 255
            AND bucket ~ '^[A-Za-z0-9._-]+$'
        ),
    CONSTRAINT ck_dr_drive_download_package_archive_object_key
        CHECK (
            archive_object_key = btrim(archive_object_key)
            AND octet_length(archive_object_key) BETWEEN 1 AND 1024
            AND archive_object_key !~ '(^/|/$|//|(^|/)[.](?:/|$)|(^|/)[.][.](?:/|$))'
        ),
    CONSTRAINT ck_dr_drive_download_package_file_count
        CHECK (file_count >= 0),
    CONSTRAINT ck_dr_drive_download_package_total_bytes
        CHECK (total_bytes >= 0),
    CONSTRAINT ck_dr_drive_download_package_archive_size_bytes
        CHECK (archive_size_bytes >= 0),
    CONSTRAINT ck_dr_drive_download_package_expires_at_epoch_ms
        CHECK (expires_at_epoch_ms > 0),
    CONSTRAINT ck_dr_drive_download_package_state
        CHECK (state IN ('creating', 'ready', 'failed', 'expired')),
    CONSTRAINT ck_dr_drive_download_package_version
        CHECK (version >= 1)
);

CREATE INDEX IF NOT EXISTS ix_dr_drive_download_package_tenant_state_created
    ON dr_drive_download_package (tenant_id, state, created_at DESC);
CREATE INDEX IF NOT EXISTS ix_dr_drive_download_package_expires
    ON dr_drive_download_package (state, expires_at_epoch_ms);

CREATE TABLE IF NOT EXISTS dr_drive_audit_event (
    id BIGINT NOT NULL PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    action VARCHAR(128) NOT NULL,
    resource_type VARCHAR(64) NOT NULL,
    resource_id VARCHAR(128) NOT NULL,
    operator_id VARCHAR(128) NOT NULL,
    request_id VARCHAR(64),
    trace_id VARCHAR(128),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS ix_dr_drive_audit_event_tenant_created
    ON dr_drive_audit_event (tenant_id, created_at DESC);
CREATE INDEX IF NOT EXISTS ix_dr_drive_audit_event_resource
    ON dr_drive_audit_event (resource_type, resource_id, created_at DESC);
CREATE INDEX IF NOT EXISTS ix_dr_drive_audit_event_action_created
    ON dr_drive_audit_event (action, created_at DESC);
CREATE INDEX IF NOT EXISTS ix_dr_drive_audit_event_request_created
    ON dr_drive_audit_event (request_id, created_at DESC);
CREATE INDEX IF NOT EXISTS ix_dr_drive_audit_event_trace_created
  ON dr_drive_audit_event (trace_id, created_at DESC);

CREATE TABLE IF NOT EXISTS dr_drive_sandbox_volume (
  id VARCHAR(128) NOT NULL PRIMARY KEY,
  tenant_id VARCHAR(64) NOT NULL,
  organization_id VARCHAR(64) NOT NULL DEFAULT '0',
  display_name VARCHAR(255) NOT NULL,
  root_entry_id VARCHAR(128) NOT NULL,
  provider_kind VARCHAR(32) NOT NULL,
  provider_root_ref TEXT NOT NULL,
  lifecycle_status VARCHAR(32) NOT NULL DEFAULT 'active',
  default_access VARCHAR(32) NOT NULL DEFAULT 'full',
  version BIGINT NOT NULL DEFAULT 1,
  created_by VARCHAR(128) NOT NULL,
  updated_by VARCHAR(128) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT ck_dr_drive_sandbox_volume_runtime_provider CHECK (provider_kind = 'local_filesystem'),
  CONSTRAINT ck_dr_drive_sandbox_volume_status CHECK (lifecycle_status IN ('active', 'read_only', 'disabled')),
  CONSTRAINT ck_dr_drive_sandbox_volume_default_access CHECK (default_access IN ('full', 'read_only'))
);
DO $$
DECLARE
  unavailable_provider_count BIGINT;
BEGIN
  IF NOT EXISTS (
    SELECT 1
    FROM pg_constraint
    WHERE connamespace = current_schema()::regnamespace
      AND conrelid = 'dr_drive_sandbox_volume'::regclass
      AND conname = 'ck_dr_drive_sandbox_volume_runtime_provider'
  ) THEN
    SELECT COUNT(1)
    INTO unavailable_provider_count
    FROM dr_drive_sandbox_volume
    WHERE provider_kind != 'local_filesystem';

    IF unavailable_provider_count > 0 THEN
      RAISE EXCEPTION
        'sandbox provider migration blocked: % volume(s) use providers without runtime adapters',
        unavailable_provider_count;
    END IF;

    ALTER TABLE dr_drive_sandbox_volume
      ADD CONSTRAINT ck_dr_drive_sandbox_volume_runtime_provider
      CHECK (provider_kind = 'local_filesystem');
  END IF;
END $$;
CREATE INDEX IF NOT EXISTS ix_dr_drive_sandbox_volume_tenant_status
  ON dr_drive_sandbox_volume (tenant_id, lifecycle_status, display_name);
CREATE INDEX IF NOT EXISTS ix_dr_drive_sandbox_volume_tenant_organization_status
  ON dr_drive_sandbox_volume (
    tenant_id,
    organization_id,
    lifecycle_status,
    display_name,
    id
  );
CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_sandbox_volume_root_entry
  ON dr_drive_sandbox_volume (id, root_entry_id);

CREATE TABLE IF NOT EXISTS dr_drive_sandbox_grant (
  id VARCHAR(128) NOT NULL PRIMARY KEY,
  sandbox_id VARCHAR(128) NOT NULL REFERENCES dr_drive_sandbox_volume(id) ON DELETE CASCADE,
  subject_type VARCHAR(32) NOT NULL,
  subject_id VARCHAR(128) NOT NULL,
  access_level VARCHAR(32) NOT NULL DEFAULT 'full',
  granted_by VARCHAR(128) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT uk_dr_drive_sandbox_grant_subject UNIQUE (sandbox_id, subject_type, subject_id),
  CONSTRAINT ck_dr_drive_sandbox_grant_subject_type CHECK (subject_type IN ('user', 'organization', 'workspace', 'role')),
  CONSTRAINT ck_dr_drive_sandbox_grant_access CHECK (access_level IN ('full', 'read_only'))
);
CREATE INDEX IF NOT EXISTS ix_dr_drive_sandbox_grant_subject
  ON dr_drive_sandbox_grant (subject_type, subject_id, sandbox_id);

CREATE TABLE IF NOT EXISTS dr_drive_sandbox_mutation_operation (
  id BIGINT NOT NULL PRIMARY KEY,
  tenant_id VARCHAR(64) NOT NULL,
  sandbox_id VARCHAR(128) NOT NULL REFERENCES dr_drive_sandbox_volume(id) ON DELETE CASCADE,
  actor_id VARCHAR(128) NOT NULL,
  idempotency_key_hash VARCHAR(64) NOT NULL,
  request_fingerprint VARCHAR(64) NOT NULL,
  mutation_kind VARCHAR(32) NOT NULL,
  parent_logical_path TEXT NOT NULL,
  entry_name VARCHAR(255) NOT NULL,
  operation_status VARCHAR(32) NOT NULL DEFAULT 'pending',
  lease_token VARCHAR(64) NOT NULL,
  lease_expires_at_ms BIGINT NOT NULL,
  result_entry_id VARCHAR(128),
  result_parent_id VARCHAR(128),
  result_entry_kind VARCHAR(32),
  result_logical_path TEXT,
  result_revision VARCHAR(128),
  result_deleted BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT uk_dr_drive_sandbox_mutation_operation_key
    UNIQUE (tenant_id, sandbox_id, actor_id, idempotency_key_hash),
  CONSTRAINT ck_dr_drive_sandbox_mutation_operation_status
    CHECK (operation_status IN ('pending', 'completed', 'failed_conflict')),
  CONSTRAINT ck_dr_drive_sandbox_mutation_operation_result_kind
    CHECK (result_entry_kind IS NULL OR result_entry_kind IN ('directory', 'file')),
  CONSTRAINT ck_dr_drive_sandbox_mutation_operation_kind
    CHECK (mutation_kind IN ('create_directory', 'create_file', 'update_file', 'move_entry', 'delete_entry'))
);
CREATE INDEX IF NOT EXISTS ix_dr_drive_sandbox_mutation_operation_pending
  ON dr_drive_sandbox_mutation_operation (operation_status, lease_expires_at_ms);
CREATE INDEX IF NOT EXISTS ix_dr_drive_sandbox_mutation_operation_sandbox_created
  ON dr_drive_sandbox_mutation_operation (tenant_id, sandbox_id, created_at DESC);

CREATE TABLE IF NOT EXISTS dr_drive_maintenance_job (
    id BIGINT NOT NULL PRIMARY KEY,
    job_type VARCHAR(64) NOT NULL,
    status VARCHAR(32) NOT NULL,
    dry_run BOOLEAN NOT NULL,
    scanned_count BIGINT NOT NULL,
    affected_count BIGINT NOT NULL,
    operator_id VARCHAR(128) NOT NULL,
    request_id VARCHAR(64),
    trace_id VARCHAR(128),
    error_message TEXT,
    started_at TIMESTAMPTZ NOT NULL,
    finished_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT ck_dr_drive_maintenance_job_type
        CHECK (job_type IN (
            'object_sweep',
            'upload_session_sweep',
            'expired_upload_content_sweep',
            'abandoned_upload_task_sweep'
        )),
    CONSTRAINT ck_dr_drive_maintenance_job_status
        CHECK (status IN ('completed', 'failed')),
    CONSTRAINT ck_dr_drive_maintenance_job_scanned_count
        CHECK (scanned_count >= 0),
    CONSTRAINT ck_dr_drive_maintenance_job_affected_count
        CHECK (affected_count >= 0)
);

CREATE INDEX IF NOT EXISTS ix_dr_drive_maintenance_job_type_created
    ON dr_drive_maintenance_job (job_type, created_at DESC);
CREATE INDEX IF NOT EXISTS ix_dr_drive_maintenance_job_status_created
    ON dr_drive_maintenance_job (status, created_at DESC);
CREATE INDEX IF NOT EXISTS ix_dr_drive_maintenance_job_operator_created
    ON dr_drive_maintenance_job (operator_id, created_at DESC);

CREATE TABLE IF NOT EXISTS dr_drive_storage_object (
    id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    node_id VARCHAR(64) NOT NULL,
    version_no BIGINT NOT NULL,
    storage_provider_id VARCHAR(64) NOT NULL,
    bucket VARCHAR(255) NOT NULL,
    object_key TEXT NOT NULL,
    scene VARCHAR(128),
    source VARCHAR(128),
    content_type VARCHAR(255) NOT NULL,
    content_length BIGINT NOT NULL,
    checksum_sha256_hex VARCHAR(255) NOT NULL,
    lifecycle_status VARCHAR(32) NOT NULL DEFAULT 'active',
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_storage_object_node_id
        FOREIGN KEY (node_id) REFERENCES dr_drive_node(id) ON DELETE CASCADE
        ,
    CONSTRAINT fk_dr_drive_storage_object_storage_provider_id
        FOREIGN KEY (storage_provider_id) REFERENCES dr_drive_storage_provider(id) ON DELETE RESTRICT,
    CONSTRAINT ck_dr_drive_storage_object_bucket
        CHECK (
            bucket = btrim(bucket)
            AND length(bucket) BETWEEN 1 AND 255
            AND bucket ~ '^[A-Za-z0-9._-]+$'
        ),
    CONSTRAINT ck_dr_drive_storage_object_object_key
        CHECK (
            object_key = btrim(object_key)
            AND octet_length(object_key) BETWEEN 1 AND 1024
            AND object_key !~ '(^/|/$|//|(^|/)[.](?:/|$)|(^|/)[.][.](?:/|$))'
        ),
    CONSTRAINT ck_dr_drive_storage_object_scene
        CHECK (scene IS NULL OR (
            scene = btrim(scene)
            AND length(scene) BETWEEN 1 AND 128
            AND scene ~ '^[A-Za-z0-9._:@-]+$'
        )),
    CONSTRAINT ck_dr_drive_storage_object_source
        CHECK (source IS NULL OR (
            source = btrim(source)
            AND length(source) BETWEEN 1 AND 128
            AND source ~ '^[A-Za-z0-9._:@-]+$'
        )),
    CONSTRAINT ck_dr_drive_storage_object_version_no
        CHECK (version_no >= 1),
    CONSTRAINT ck_dr_drive_storage_object_content_type
        CHECK (
            content_type = btrim(content_type)
            AND length(content_type) BETWEEN 3 AND 255
            AND content_type ~ '^[^[:space:]/]+/[^[:space:]/]+$'
        ),
    CONSTRAINT ck_dr_drive_storage_object_content_length
        CHECK (content_length >= 0),
    CONSTRAINT ck_dr_drive_storage_object_checksum_sha256_hex
        CHECK (checksum_sha256_hex ~ '^sha256:[0-9a-f]{64}$'),
    CONSTRAINT ck_dr_drive_storage_object_lifecycle_status
        CHECK (lifecycle_status IN ('active', 'deleted'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_storage_object_node_version
    ON dr_drive_storage_object (tenant_id, node_id, version_no);
CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_storage_object_active_locator
    ON dr_drive_storage_object (tenant_id, node_id, storage_provider_id, bucket, object_key)
    WHERE lifecycle_status = 'active';
CREATE INDEX IF NOT EXISTS ix_dr_drive_storage_object_node_latest
    ON dr_drive_storage_object (tenant_id, node_id, lifecycle_status, version_no DESC);

CREATE TABLE IF NOT EXISTS dr_drive_node_version (
    id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    space_id VARCHAR(64) NOT NULL,
    node_id VARCHAR(64) NOT NULL,
    version_no BIGINT NOT NULL,
    storage_object_id VARCHAR(64),
    content_type VARCHAR(255) NOT NULL,
    content_length BIGINT NOT NULL,
    checksum_sha256_hex VARCHAR(255) NOT NULL,
    version_kind VARCHAR(32) NOT NULL DEFAULT 'auto',
    version_label VARCHAR(128),
    change_source VARCHAR(32) NOT NULL DEFAULT 'app_api',
    change_summary TEXT,
    restored_from_version_id VARCHAR(64),
    app_id VARCHAR(128),
    app_resource_type VARCHAR(64),
    app_resource_id VARCHAR(128),
    scene VARCHAR(128),
    source VARCHAR(128),
    lifecycle_status VARCHAR(32) NOT NULL DEFAULT 'active',
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_node_version_space_id
        FOREIGN KEY (space_id) REFERENCES dr_drive_space(id) ON DELETE CASCADE,
    CONSTRAINT fk_dr_drive_node_version_node_id
        FOREIGN KEY (node_id) REFERENCES dr_drive_node(id) ON DELETE CASCADE,
    CONSTRAINT fk_dr_drive_node_version_storage_object_id
        FOREIGN KEY (storage_object_id) REFERENCES dr_drive_storage_object(id) ON DELETE SET NULL,
    CONSTRAINT fk_dr_drive_node_version_restored_from_version_id
        FOREIGN KEY (restored_from_version_id) REFERENCES dr_drive_node_version(id) ON DELETE SET NULL,
    CONSTRAINT ck_dr_drive_node_version_version_no
        CHECK (version_no >= 1),
    CONSTRAINT ck_dr_drive_node_version_content_type
        CHECK (
            content_type = btrim(content_type)
            AND length(content_type) BETWEEN 3 AND 255
            AND content_type ~ '^[^[:space:]/]+/[^[:space:]/]+$'
        ),
    CONSTRAINT ck_dr_drive_node_version_content_length
        CHECK (content_length >= 0),
    CONSTRAINT ck_dr_drive_node_version_checksum_sha256_hex
        CHECK (checksum_sha256_hex ~ '^sha256:[0-9a-f]{64}$'),
    CONSTRAINT ck_dr_drive_node_version_kind
        CHECK (version_kind IN ('auto', 'manual', 'restore', 'import', 'ai_generated', 'system')),
    CONSTRAINT ck_dr_drive_node_version_label
        CHECK (version_label IS NULL OR (version_label = btrim(version_label) AND length(version_label) BETWEEN 1 AND 128)),
    CONSTRAINT ck_dr_drive_node_version_change_source
        CHECK (change_source IN ('app_api', 'backend_api', 'uploader', 'sync', 'ai', 'import', 'restore', 'system')),
    CONSTRAINT ck_dr_drive_node_version_change_summary
        CHECK (change_summary IS NULL OR (change_summary = btrim(change_summary) AND length(change_summary) BETWEEN 1 AND 1024)),
    CONSTRAINT ck_dr_drive_node_version_app_id
        CHECK (app_id IS NULL OR (app_id = btrim(app_id) AND length(app_id) BETWEEN 1 AND 128 AND app_id ~ '^[A-Za-z0-9._:@-]+$')),
    CONSTRAINT ck_dr_drive_node_version_app_resource_type
        CHECK (app_resource_type IS NULL OR (app_resource_type = btrim(app_resource_type) AND length(app_resource_type) BETWEEN 1 AND 64 AND app_resource_type ~ '^[A-Za-z0-9._:@-]+$')),
    CONSTRAINT ck_dr_drive_node_version_app_resource_id
        CHECK (app_resource_id IS NULL OR (app_resource_id = btrim(app_resource_id) AND length(app_resource_id) BETWEEN 1 AND 128 AND app_resource_id ~ '^[A-Za-z0-9._:@-]+$')),
    CONSTRAINT ck_dr_drive_node_version_scene
        CHECK (scene IS NULL OR (
            scene = btrim(scene)
            AND length(scene) BETWEEN 1 AND 128
            AND scene ~ '^[A-Za-z0-9._:@-]+$'
        )),
    CONSTRAINT ck_dr_drive_node_version_source
        CHECK (source IS NULL OR (
            source = btrim(source)
            AND length(source) BETWEEN 1 AND 128
            AND source ~ '^[A-Za-z0-9._:@-]+$'
        )),
    CONSTRAINT ck_dr_drive_node_version_lifecycle_status
        CHECK (lifecycle_status IN ('active', 'deleted'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_node_version_node_version
    ON dr_drive_node_version (tenant_id, node_id, version_no);
CREATE INDEX IF NOT EXISTS ix_dr_drive_node_version_node_latest
    ON dr_drive_node_version (tenant_id, node_id, lifecycle_status, version_no DESC);
CREATE INDEX IF NOT EXISTS ix_dr_drive_node_version_storage_object
    ON dr_drive_node_version (tenant_id, storage_object_id)
    WHERE storage_object_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS ix_dr_drive_node_version_app_resource
    ON dr_drive_node_version (tenant_id, app_id, app_resource_type, app_resource_id, created_at DESC)
    WHERE app_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS dr_drive_space_version_policy (
    id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    space_id VARCHAR(64) NOT NULL,
    versioning_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    default_version_kind VARCHAR(32) NOT NULL DEFAULT 'auto',
    retention_mode VARCHAR(32) NOT NULL DEFAULT 'unlimited',
    max_versions BIGINT,
    retention_days BIGINT,
    keep_deleted_versions BOOLEAN NOT NULL DEFAULT TRUE,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_space_version_policy_space_id
        FOREIGN KEY (space_id) REFERENCES dr_drive_space(id) ON DELETE CASCADE,
    CONSTRAINT ck_dr_drive_space_version_policy_kind
        CHECK (default_version_kind IN ('auto', 'manual', 'restore', 'import', 'ai_generated', 'system')),
    CONSTRAINT ck_dr_drive_space_version_policy_retention_mode
        CHECK (retention_mode IN ('unlimited', 'max_versions', 'time_window')),
    CONSTRAINT ck_dr_drive_space_version_policy_max_versions
        CHECK (max_versions IS NULL OR max_versions >= 1),
    CONSTRAINT ck_dr_drive_space_version_policy_retention_days
        CHECK (retention_days IS NULL OR retention_days >= 1)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_space_version_policy_space
    ON dr_drive_space_version_policy (tenant_id, space_id);

CREATE TABLE IF NOT EXISTS dr_drive_node_version_policy (
    id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    space_id VARCHAR(64) NOT NULL,
    node_id VARCHAR(64) NOT NULL,
    versioning_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    default_version_kind VARCHAR(32) NOT NULL DEFAULT 'auto',
    retention_mode VARCHAR(32) NOT NULL DEFAULT 'unlimited',
    max_versions BIGINT,
    retention_days BIGINT,
    keep_deleted_versions BOOLEAN NOT NULL DEFAULT TRUE,
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_node_version_policy_space_id
        FOREIGN KEY (space_id) REFERENCES dr_drive_space(id) ON DELETE CASCADE,
    CONSTRAINT fk_dr_drive_node_version_policy_node_id
        FOREIGN KEY (node_id) REFERENCES dr_drive_node(id) ON DELETE CASCADE,
    CONSTRAINT ck_dr_drive_node_version_policy_kind
        CHECK (default_version_kind IN ('auto', 'manual', 'restore', 'import', 'ai_generated', 'system')),
    CONSTRAINT ck_dr_drive_node_version_policy_retention_mode
        CHECK (retention_mode IN ('unlimited', 'max_versions', 'time_window')),
    CONSTRAINT ck_dr_drive_node_version_policy_max_versions
        CHECK (max_versions IS NULL OR max_versions >= 1),
    CONSTRAINT ck_dr_drive_node_version_policy_retention_days
        CHECK (retention_days IS NULL OR retention_days >= 1)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_node_version_policy_node
    ON dr_drive_node_version_policy (tenant_id, node_id);

CREATE TABLE IF NOT EXISTS dr_drive_upload_item (
    id VARCHAR(64) PRIMARY KEY,
    task_id VARCHAR(128) NOT NULL,
    tenant_id VARCHAR(64) NOT NULL,
    organization_id VARCHAR(128) NOT NULL DEFAULT '0',
    user_id VARCHAR(128),
    actor_type VARCHAR(32) NOT NULL,
    actor_id VARCHAR(128) NOT NULL,
    app_id VARCHAR(128) NOT NULL,
    app_resource_type VARCHAR(64) NOT NULL,
    app_resource_id VARCHAR(128) NOT NULL,
    scene VARCHAR(128),
    source VARCHAR(128),
    upload_profile_code VARCHAR(64) NOT NULL,
    file_fingerprint VARCHAR(255) NOT NULL,
    space_id VARCHAR(64) NOT NULL,
    node_id VARCHAR(64) NOT NULL,
    upload_session_id VARCHAR(64),
    storage_provider_id VARCHAR(64),
    storage_upload_id TEXT,
    original_file_name VARCHAR(255) NOT NULL,
    file_extension VARCHAR(64),
    content_type VARCHAR(255) NOT NULL,
    content_type_group VARCHAR(64) NOT NULL,
    detected_content_type VARCHAR(255),
    content_length BIGINT NOT NULL,
    checksum_sha256_hex VARCHAR(255),
    chunk_size_bytes BIGINT NOT NULL,
    total_parts INTEGER NOT NULL,
    uploaded_parts_count INTEGER NOT NULL DEFAULT 0,
    uploaded_bytes BIGINT NOT NULL DEFAULT 0,
    status VARCHAR(32) NOT NULL,
    retention_mode VARCHAR(32) NOT NULL,
    retention_expires_at_epoch_ms BIGINT,
    cleanup_action VARCHAR(32),
    hard_delete_after_epoch_ms BIGINT,
    cleanup_status VARCHAR(32) NOT NULL DEFAULT 'active',
    post_process_status VARCHAR(32) NOT NULL DEFAULT 'not_required',
    created_by VARCHAR(128) NOT NULL,
    updated_by VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_upload_item_space_id
        FOREIGN KEY (space_id) REFERENCES dr_drive_space(id) ON DELETE CASCADE,
    CONSTRAINT fk_dr_drive_upload_item_node_id
        FOREIGN KEY (node_id) REFERENCES dr_drive_node(id) ON DELETE CASCADE,
    CONSTRAINT fk_dr_drive_upload_item_upload_session_id
        FOREIGN KEY (upload_session_id) REFERENCES dr_drive_upload_session(id) ON DELETE SET NULL,
    CONSTRAINT fk_dr_drive_upload_item_storage_provider_id
        FOREIGN KEY (storage_provider_id) REFERENCES dr_drive_storage_provider(id) ON DELETE SET NULL,
    CONSTRAINT ck_dr_drive_upload_item_actor_type
        CHECK (actor_type IN ('anonymous', 'user', 'system')),
    CONSTRAINT ck_dr_drive_upload_item_scene
        CHECK (scene IS NULL OR (
            scene = btrim(scene)
            AND length(scene) BETWEEN 1 AND 128
            AND scene ~ '^[A-Za-z0-9._:@-]+$'
        )),
    CONSTRAINT ck_dr_drive_upload_item_source
        CHECK (source IS NULL OR (
            source = btrim(source)
            AND length(source) BETWEEN 1 AND 128
            AND source ~ '^[A-Za-z0-9._:@-]+$'
        )),
    CONSTRAINT ck_dr_drive_upload_item_profile
        CHECK (upload_profile_code IN (
            'generic', 'video', 'image', 'audio', 'document', 'archive',
            'text', 'dataset', 'attachment', 'avatar', 'thumbnail'
        )),
    CONSTRAINT ck_dr_drive_upload_item_content_type
        CHECK (
            content_type = btrim(content_type)
            AND length(content_type) BETWEEN 3 AND 255
            AND content_type ~ '^[^[:space:]/]+/[^[:space:]/]+$'
        ),
    CONSTRAINT ck_dr_drive_upload_item_content_length
        CHECK (content_length >= 0),
    CONSTRAINT ck_dr_drive_upload_item_chunk_size
        CHECK (chunk_size_bytes > 0),
    CONSTRAINT ck_dr_drive_upload_item_total_parts
        CHECK (total_parts >= 1),
    CONSTRAINT ck_dr_drive_upload_item_uploaded_parts
        CHECK (uploaded_parts_count >= 0),
    CONSTRAINT ck_dr_drive_upload_item_uploaded_bytes
        CHECK (uploaded_bytes >= 0),
    CONSTRAINT ck_dr_drive_upload_item_status
        CHECK (status IN ('prepared', 'uploading', 'paused', 'completing', 'completed', 'failed', 'cancelled', 'expired')),
    CONSTRAINT ck_dr_drive_upload_item_retention
        CHECK (
            (retention_mode = 'long_term' AND retention_expires_at_epoch_ms IS NULL AND cleanup_action IS NULL)
            OR
            (retention_mode = 'temporary' AND retention_expires_at_epoch_ms IS NOT NULL AND cleanup_action IN ('soft_delete', 'hard_delete'))
        ),
    CONSTRAINT ck_dr_drive_upload_item_cleanup_status
        CHECK (cleanup_status IN ('active', 'expired', 'soft_deleted', 'hard_deleted', 'failed')),
    CONSTRAINT ck_dr_drive_upload_item_post_process_status
        CHECK (post_process_status IN ('not_required', 'pending', 'processing', 'completed', 'failed')),
    CONSTRAINT ck_dr_drive_upload_item_checksum_sha256_hex
        CHECK (checksum_sha256_hex IS NULL OR checksum_sha256_hex ~ '^sha256:[0-9a-f]{64}$')
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_upload_item_task
    ON dr_drive_upload_item (tenant_id, task_id);
CREATE INDEX IF NOT EXISTS ix_dr_drive_upload_item_fingerprint
    ON dr_drive_upload_item (tenant_id, file_fingerprint, status);
CREATE INDEX IF NOT EXISTS ix_dr_drive_upload_item_retention
    ON dr_drive_upload_item (tenant_id, cleanup_status, retention_expires_at_epoch_ms);
CREATE INDEX IF NOT EXISTS ix_dr_drive_upload_item_usage_scope
    ON dr_drive_upload_item (
        tenant_id, organization_id, user_id, app_id,
        scene, source, upload_profile_code, content_type_group, cleanup_status
    );

CREATE TABLE IF NOT EXISTS dr_drive_upload_part (
    id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    upload_item_id VARCHAR(64) NOT NULL,
    upload_session_id VARCHAR(64) NOT NULL,
    part_no INTEGER NOT NULL,
    offset_bytes BIGINT NOT NULL,
    size_bytes BIGINT NOT NULL,
    etag TEXT NOT NULL,
    checksum_sha256_hex VARCHAR(255),
    status VARCHAR(32) NOT NULL,
    retry_count INTEGER NOT NULL DEFAULT 0,
    uploaded_at_epoch_ms BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_upload_part_upload_item_id
        FOREIGN KEY (upload_item_id) REFERENCES dr_drive_upload_item(id) ON DELETE CASCADE,
    CONSTRAINT fk_dr_drive_upload_part_upload_session_id
        FOREIGN KEY (upload_session_id) REFERENCES dr_drive_upload_session(id) ON DELETE CASCADE,
    CONSTRAINT ck_dr_drive_upload_part_no
        CHECK (part_no BETWEEN 1 AND 10000),
    CONSTRAINT ck_dr_drive_upload_part_offset
        CHECK (offset_bytes >= 0),
    CONSTRAINT ck_dr_drive_upload_part_size
        CHECK (size_bytes > 0),
    CONSTRAINT ck_dr_drive_upload_part_status
        CHECK (status IN ('pending', 'uploading', 'uploaded', 'failed')),
    CONSTRAINT ck_dr_drive_upload_part_retry_count
        CHECK (retry_count >= 0),
    CONSTRAINT ck_dr_drive_upload_part_checksum_sha256_hex
        CHECK (checksum_sha256_hex IS NULL OR checksum_sha256_hex ~ '^sha256:[0-9a-f]{64}$')
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_dr_drive_upload_part_item_part
    ON dr_drive_upload_part (tenant_id, upload_item_id, part_no);
CREATE INDEX IF NOT EXISTS ix_dr_drive_upload_part_session
    ON dr_drive_upload_part (tenant_id, upload_session_id, status, part_no);

CREATE TABLE IF NOT EXISTS dr_drive_file_sensitive_operation (
    id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    organization_id VARCHAR(128) NOT NULL DEFAULT '0',
    user_id VARCHAR(128),
    space_id VARCHAR(64) NOT NULL,
    node_id VARCHAR(64) NOT NULL,
    storage_object_id VARCHAR(64),
    upload_item_id VARCHAR(64),
    operation_type VARCHAR(64) NOT NULL,
    operation_reason VARCHAR(64) NOT NULL,
    content_type VARCHAR(255) NOT NULL,
    content_type_group VARCHAR(64) NOT NULL,
    content_length BIGINT NOT NULL,
    checksum_sha256_hex VARCHAR(255),
    object_bucket VARCHAR(255),
    object_key TEXT,
    before_lifecycle_status VARCHAR(32),
    after_lifecycle_status VARCHAR(32),
    operator_id VARCHAR(128) NOT NULL,
    maintenance_job_id BIGINT,
    request_id VARCHAR(64),
    trace_id VARCHAR(128),
    object_delete_status VARCHAR(32) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_dr_drive_file_sensitive_operation_space_id
        FOREIGN KEY (space_id) REFERENCES dr_drive_space(id) ON DELETE CASCADE,
    CONSTRAINT fk_dr_drive_file_sensitive_operation_node_id
        FOREIGN KEY (node_id) REFERENCES dr_drive_node(id) ON DELETE CASCADE,
    CONSTRAINT fk_dr_drive_file_sensitive_operation_storage_object_id
        FOREIGN KEY (storage_object_id) REFERENCES dr_drive_storage_object(id) ON DELETE SET NULL,
    CONSTRAINT fk_dr_drive_file_sensitive_operation_upload_item_id
        FOREIGN KEY (upload_item_id) REFERENCES dr_drive_upload_item(id) ON DELETE SET NULL,
    CONSTRAINT fk_dr_drive_file_sensitive_operation_maintenance_job_id
        FOREIGN KEY (maintenance_job_id) REFERENCES dr_drive_maintenance_job(id) ON DELETE SET NULL,
    CONSTRAINT ck_dr_drive_file_sensitive_operation_type
        CHECK (operation_type IN (
            'upload_completed',
            'soft_delete',
            'hard_delete',
            'restore',
            'share_created',
            'share_revoked',
            'permission_changed',
            'download_granted',
            'retention_expired'
        )),
    CONSTRAINT ck_dr_drive_file_sensitive_operation_reason
        CHECK (operation_reason IN (
            'user_request',
            'retention_expired',
            'manual',
            'admin',
            'quota_policy',
            'system'
        )),
    CONSTRAINT ck_dr_drive_file_sensitive_operation_content_length
        CHECK (content_length >= 0),
    CONSTRAINT ck_dr_drive_file_sensitive_operation_object_status
        CHECK (object_delete_status IN ('not_required', 'deleted', 'missing', 'failed')),
    CONSTRAINT ck_dr_drive_file_sensitive_operation_checksum_sha256_hex
        CHECK (checksum_sha256_hex IS NULL OR checksum_sha256_hex ~ '^sha256:[0-9a-f]{64}$')
);

CREATE INDEX IF NOT EXISTS ix_dr_drive_file_sensitive_operation_upload_item
    ON dr_drive_file_sensitive_operation (tenant_id, upload_item_id, created_at DESC);
CREATE INDEX IF NOT EXISTS ix_dr_drive_file_sensitive_operation_tenant_created
    ON dr_drive_file_sensitive_operation (tenant_id, created_at DESC);

CREATE TABLE IF NOT EXISTS dr_drive_domain_outbox (
    id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    space_id VARCHAR(64) NOT NULL,
    node_id VARCHAR(64),
    event_type VARCHAR(128) NOT NULL,
    actor_id VARCHAR(128) NOT NULL,
    sequence_no BIGINT NOT NULL,
    payload_json TEXT NOT NULL,
    delivery_status VARCHAR(32) NOT NULL DEFAULT 'pending',
    attempt_count INT NOT NULL DEFAULT 0,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    delivered_at TIMESTAMPTZ,
    CONSTRAINT ck_dr_drive_domain_outbox_delivery_status
        CHECK (delivery_status IN ('pending', 'delivered', 'failed'))
);

CREATE INDEX IF NOT EXISTS ix_dr_drive_domain_outbox_pending
    ON dr_drive_domain_outbox (delivery_status, created_at);
CREATE INDEX IF NOT EXISTS ix_dr_drive_domain_outbox_pending_dispatch
    ON dr_drive_domain_outbox (attempt_count, created_at)
    WHERE delivery_status = 'pending';

CREATE TABLE IF NOT EXISTS dr_drive_domain_outbox_channel_delivery (
    outbox_id VARCHAR(64) NOT NULL,
    channel_id VARCHAR(64) NOT NULL,
    delivered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (outbox_id, channel_id),
    CONSTRAINT fk_dr_drive_domain_outbox_channel_delivery_outbox_id
        FOREIGN KEY (outbox_id) REFERENCES dr_drive_domain_outbox(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS ix_dr_drive_domain_outbox_channel_delivery_channel
    ON dr_drive_domain_outbox_channel_delivery (channel_id, delivered_at DESC);

CREATE TABLE IF NOT EXISTS dr_drive_maintenance_leader (
    lock_key VARCHAR(64) PRIMARY KEY,
    holder_id VARCHAR(128) NOT NULL,
    acquired_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
