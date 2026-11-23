-- Default logical storage plane for the platform bootstrap tenant.
--
-- Why this exists
-- ---------------
-- A fresh install must be usable after the operator edits *one* thing: the
-- provider account credentials. The physical plane (provider configurations,
-- buckets, reusable accounts) is seeded by `002_storage_provider_catalog.sql`;
-- this seed adds the logical plane on top of it:
--
--   business scope -> logical bucket -> provider configuration -> physical bucket
--
-- Space-type rows never name a provider. The built-in Tencent COS provider is
-- the default provider for every logical bucket, so switching providers later is
-- one edit per logical bucket instead of eleven edits per business route.
--
-- Idempotency: every insert is ON CONFLICT DO NOTHING. An operator edit (a
-- renamed bucket, a different provider, a custom prefix) is never overwritten by
-- a re-run or upgrade.
--
-- Runtime bootstrap mirrors this seed for `SDKWORK_DRIVE_BOOTSTRAP_TENANT_ID`
-- so a deployment whose tenant is not `100001` still starts fully routed.

INSERT INTO dr_drive_logical_bucket (
    id, tenant_id, bucket_key, display_name, object_prefix,
    lifecycle_status, version, created_by, updated_by
)
VALUES
    ('logical-bucket:100001:business', '100001', 'business', 'Business Content',
     'logical/business', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('logical-bucket:100001:system', '100001', 'system', 'System Content',
     'logical/system', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('logical-bucket:100001:archive', '100001', 'archive', 'Archive',
     'logical/archive', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('logical-bucket:100001:temp', '100001', 'temp', 'Temporary',
     'logical/temp', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed')
ON CONFLICT (id) DO NOTHING;

INSERT INTO dr_drive_logical_bucket_provider_binding (
    id, tenant_id, logical_bucket_id, provider_id, storage_root_prefix,
    lifecycle_status, version, created_by, updated_by
)
VALUES
    ('logical-bucket-provider:100001:business', '100001',
     'logical-bucket:100001:business', 'builtin-storage-provider-100001-tencent-cos',
     'sdkwork-drive/v1/tenants/100001', 'active', 1,
     'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('logical-bucket-provider:100001:system', '100001',
     'logical-bucket:100001:system', 'builtin-storage-provider-100001-tencent-cos',
     'sdkwork-drive/v1/tenants/100001', 'active', 1,
     'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('logical-bucket-provider:100001:archive', '100001',
     'logical-bucket:100001:archive', 'builtin-storage-provider-100001-tencent-cos',
     'sdkwork-drive/v1/tenants/100001', 'active', 1,
     'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('logical-bucket-provider:100001:temp', '100001',
     'logical-bucket:100001:temp', 'builtin-storage-provider-100001-tencent-cos',
     'sdkwork-drive/v1/tenants/100001', 'active', 1,
     'sdkwork-storage-seed', 'sdkwork-storage-seed')
ON CONFLICT (id) DO NOTHING;

INSERT INTO dr_drive_business_storage_binding (
    id, tenant_id, scope_type, scope_id, logical_bucket_id, write_prefix,
    lifecycle_status, version, created_by, updated_by
)
VALUES
    ('business-storage:100001:space_type:personal', '100001', 'space_type', 'personal',
     'logical-bucket:100001:business', 'space-types/personal', 'active', 1,
     'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('business-storage:100001:space_type:team', '100001', 'space_type', 'team',
     'logical-bucket:100001:business', 'space-types/team', 'active', 1,
     'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('business-storage:100001:space_type:knowledge_base', '100001', 'space_type', 'knowledge_base',
     'logical-bucket:100001:business', 'space-types/knowledge_base', 'active', 1,
     'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('business-storage:100001:space_type:ai_generated', '100001', 'space_type', 'ai_generated',
     'logical-bucket:100001:business', 'space-types/ai_generated', 'active', 1,
     'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('business-storage:100001:space_type:im', '100001', 'space_type', 'im',
     'logical-bucket:100001:business', 'space-types/im', 'active', 1,
     'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('business-storage:100001:space_type:rtc', '100001', 'space_type', 'rtc',
     'logical-bucket:100001:business', 'space-types/rtc', 'active', 1,
     'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('business-storage:100001:space_type:git_repository', '100001', 'space_type', 'git_repository',
     'logical-bucket:100001:system', 'space-types/git_repository', 'active', 1,
     'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('business-storage:100001:space_type:deployment', '100001', 'space_type', 'deployment',
     'logical-bucket:100001:system', 'space-types/deployment', 'active', 1,
     'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('business-storage:100001:space_type:app_upload', '100001', 'space_type', 'app_upload',
     'logical-bucket:100001:system', 'space-types/app_upload', 'active', 1,
     'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('business-storage:100001:space_type:notary', '100001', 'space_type', 'notary',
     'logical-bucket:100001:system', 'space-types/notary', 'active', 1,
     'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('business-storage:100001:space_type:website', '100001', 'space_type', 'website',
     'logical-bucket:100001:system', 'space-types/website', 'active', 1,
     'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    -- Document bodies are product content written by the documents service, so
    -- they route to the same system bucket as the other platform-managed types.
    ('business-storage:100001:space_type:documents', '100001', 'space_type', 'documents',
     'logical-bucket:100001:system', 'space-types/documents', 'active', 1,
     'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    -- Certificate bundles are platform-managed artifacts, not user content, so
    -- they route to the system bucket like deployment and website assets.
    ('business-storage:100001:space_type:certificate', '100001', 'space_type', 'certificate',
     'logical-bucket:100001:system', 'space-types/certificate', 'active', 1,
     'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('business-storage:100001:tenant:100001', '100001', 'tenant', '100001',
     'logical-bucket:100001:business', 'tenant', 'active', 1,
     'sdkwork-storage-seed', 'sdkwork-storage-seed')
ON CONFLICT (id) DO NOTHING;
