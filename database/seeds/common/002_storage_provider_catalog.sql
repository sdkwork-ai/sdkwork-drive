-- Storage provider initialization data for the platform bootstrap tenant.
--
-- Why this exists
-- ---------------
-- `/admin/storage/providers` (Drive-owned surface, composed into the CloudRouter
-- console) reads `dr_drive_storage_provider`. Before this seed the table was
-- empty on a fresh install and on upgrade: the rows were only created by the
-- application's startup bootstrap (`bootstrap_storage_provider_defaults`), which
-- is fail-open and therefore cannot be relied on for a database that must be
-- complete right after `install`/`upgrade`. The AWS-style account-center
-- bootstrap also needs a writable IAM key ring, so a SQL seed is the only
-- database-side way to ship the catalog deterministically.
--
-- What it seeds
-- -------------
-- 1. The built-in provider-kind catalog (25 kinds) — the same set the baseline
--    DDL installs. Repeated here so an upgraded database that predates a kind
--    converges without the application running; `ON CONFLICT DO NOTHING` keeps
--    an operator's enable/disable decision intact.
-- 2. One provider configuration per mainstream cloud vendor plus the
--    credential-free `local_filesystem` fallback, for tenant `100001`
--    (`PLATFORM_TENANT_ID`, the tenant the console administrative user belongs
--    to). Ids match the application bootstrap's derivation
--    (`builtin-storage-provider-<tenant>-<kind>`), so a later runtime bootstrap
--    recognizes these rows instead of creating a second set.
-- 3. The tenant's default binding, pointing at Tencent Cloud COS. SDKWork's
--    primary cloud is Tencent COS, so a fresh plane stores real objects there
--    once the operator supplies the credential — no hand-created provider or
--    binding is required.
--
-- How an operator enables a vendor
-- --------------------------------
-- `credential_ref` is seeded as an `env:` reference, so deploying only needs the
-- vendor's key pair in the environment:
--
-- * Tencent COS: `SDKWORK_DRIVE_TENCENT_COS_SECRET_ID` /
--   `SDKWORK_DRIVE_TENCENT_COS_SECRET_KEY` (the same pair the deployment
--   bootstrap reads). Bucket/region/endpoint can also be overridden from the
--   provider row in the console, or with `SDKWORK_DRIVE_TENCENT_COS_BUCKET`,
--   `SDKWORK_DRIVE_TENCENT_COS_REGION`, `SDKWORK_DRIVE_TENCENT_COS_ENDPOINT`.
-- * Every other vendor: the variable names the console shows for that vendor
--   (`AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY`, `COS_SECRET_ID` /
--   `COS_SECRET_KEY`, `MINIO_ROOT_USER` / `MINIO_ROOT_PASSWORD`, ...). Path-style
--   addressing, TLS, the default SSE mode and storage class are pre-filled from
--   the provider-kind contract, so editing the endpoint/region/bucket/keys is
--   the only step left.
--
-- Endpoints, regions and bucket placeholders mirror the application's built-in
-- vendor table (`crates/sdkwork-routes-storage-backend-api/src/provider_account_defaults.rs`);
-- a contract test pins the two together so they cannot drift.
--
-- Idempotency and upgrade safety
-- ------------------------------
-- Every insert is `ON CONFLICT DO NOTHING`, and the binding is inserted only
-- when the tenant has no active tenant-scope `primary` binding. Re-running the
-- seed therefore never rewrites an operator edit (a renamed bucket, a re-pointed
-- credential, a different default vendor) and an upgrade only adds vendors that
-- are still missing. The tenant id is the platform bootstrap tenant; a
-- deployment whose console tenant differs keeps using
-- `SDKWORK_DRIVE_BOOTSTRAP_TENANT_ID` for the runtime bootstrap.

INSERT INTO dr_drive_storage_provider_kind (provider_kind, display_name, enabled, sort_order)
VALUES
    ('local_filesystem', 'Local Filesystem', TRUE, 1),
    ('s3_compatible', 'Amazon S3 / S3 Compatible', TRUE, 2),
    ('aliyun_oss', 'Alibaba Cloud OSS', TRUE, 10),
    ('tencent_cos', 'Tencent Cloud COS', TRUE, 11),
    ('huawei_obs', 'Huawei Cloud OBS', TRUE, 12),
    ('volcengine_tos', 'Volcengine TOS', TRUE, 13),
    ('baidu_bos', 'Baidu Cloud BOS', TRUE, 14),
    ('kingsoft_ks3', 'Kingsoft Cloud KS3', TRUE, 15),
    ('qiniu_kodo', 'Qiniu Kodo', TRUE, 16),
    ('china_mobile_ecloud', 'China Mobile Ecloud', TRUE, 17),
    ('china_telecom_eos', 'China Telecom EOS', TRUE, 18),
    ('china_unicom_wo', 'China Unicom Wo Cloud', TRUE, 19),
    ('minio', 'MinIO', TRUE, 30),
    ('cloudflare_r2', 'Cloudflare R2', TRUE, 31),
    ('backblaze_b2', 'Backblaze B2', TRUE, 32),
    ('wasabi', 'Wasabi', TRUE, 33),
    ('digitalocean_spaces', 'DigitalOcean Spaces', TRUE, 34),
    ('linode_object_storage', 'Akamai / Linode Object Storage', TRUE, 35),
    ('vultr_object_storage', 'Vultr Object Storage', TRUE, 36),
    ('scaleway_object_storage', 'Scaleway Object Storage', TRUE, 37),
    ('oracle_cloud_storage', 'Oracle Cloud Infrastructure Object Storage', TRUE, 38),
    ('ibm_cos', 'IBM Cloud Object Storage', TRUE, 39),
    ('alibaba_cloud_international', 'Alibaba Cloud OSS (International)', TRUE, 40),
    ('tencent_cloud_international', 'Tencent Cloud COS (International)', TRUE, 41),
    ('google_cloud_storage', 'Google Cloud Storage', TRUE, 50)
ON CONFLICT (provider_kind) DO NOTHING;

INSERT INTO dr_drive_storage_provider (
    id, tenant_id, provider_kind, name, endpoint_url, region, bucket,
    path_style, strict_tls, credential_ref, provider_account_id,
    server_side_encryption_mode, default_storage_class, status, version,
    created_by, updated_by
)
VALUES
    ('builtin-storage-provider-100001-local-filesystem', '100001', 'local_filesystem',
     'Built-in Local Filesystem', 'file:///var/lib/sdkwork-drive', NULL, 'sdkwork-drive',
     TRUE, TRUE, NULL, NULL, NULL, NULL, 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-s3-compatible', '100001', 's3_compatible',
     'Built-in Amazon S3', 'https://s3.us-east-1.amazonaws.com', 'us-east-1',
     'sdkwork-drive-s3-compatible', TRUE, TRUE,
     'env:AWS_ACCESS_KEY_ID:AWS_SECRET_ACCESS_KEY', NULL,
     'AES256', 'STANDARD', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-aliyun-oss', '100001', 'aliyun_oss',
     'Built-in Alibaba Cloud OSS', 'https://oss-cn-hangzhou.aliyuncs.com', 'cn-hangzhou',
     'sdkwork-drive-aliyun-oss', FALSE, TRUE,
     'env:ALIBABA_CLOUD_ACCESS_KEY_ID:ALIBABA_CLOUD_ACCESS_KEY_SECRET', NULL,
     'KMS', 'Standard', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-tencent-cos', '100001', 'tencent_cos',
     'Built-in Tencent Cloud COS', 'https://cos.ap-guangzhou.myqcloud.com', 'ap-guangzhou',
     'sdkwork-drive-cos-1250000000', FALSE, TRUE,
     'env:SDKWORK_DRIVE_TENCENT_COS_SECRET_ID:SDKWORK_DRIVE_TENCENT_COS_SECRET_KEY', NULL,
     'AES256', 'STANDARD', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-huawei-obs', '100001', 'huawei_obs',
     'Built-in Huawei Cloud OBS', 'https://obs.cn-north-1.myhuaweicloud.com', 'cn-north-1',
     'sdkwork-drive-huawei-obs', FALSE, TRUE,
     'env:HUAWEI_ACCESS_KEY_ID:HUAWEI_SECRET_ACCESS_KEY', NULL,
     'kms', 'STANDARD', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-volcengine-tos', '100001', 'volcengine_tos',
     'Built-in Volcengine TOS', 'https://tos-cn-beijing.volces.com', 'cn-beijing',
     'sdkwork-drive-volcengine-tos', FALSE, TRUE,
     'env:VOLC_ACCESSKEY:VOLC_SECRETKEY', NULL,
     'AES256', 'STANDARD', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-google-cloud-storage', '100001', 'google_cloud_storage',
     'Built-in Google Cloud Storage', 'https://storage.googleapis.com', 'us-central1',
     'sdkwork-drive-gcs', FALSE, TRUE,
     'env:GOOGLE_ACCESS_KEY_ID:GOOGLE_SECRET_ACCESS_KEY', NULL,
     'AES256', 'STANDARD', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-baidu-bos', '100001', 'baidu_bos',
     'Built-in Baidu Cloud BOS', 'https://s3.bj.bcebos.com', 'bj',
     'sdkwork-drive-baidu-bos', FALSE, TRUE,
     'env:BCE_ACCESS_KEY_ID:BCE_SECRET_ACCESS_KEY', NULL,
     'AES256', 'STANDARD', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-kingsoft-ks3', '100001', 'kingsoft_ks3',
     'Built-in Kingsoft Cloud KS3', 'https://ks3-cn-beijing.ksyuncs.com', 'BEIJING',
     'sdkwork-drive-ks3', FALSE, TRUE,
     'env:KS3_ACCESS_KEY_ID:KS3_SECRET_ACCESS_KEY', NULL,
     'AES256', 'STANDARD', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-qiniu-kodo', '100001', 'qiniu_kodo',
     'Built-in Qiniu Kodo', 'https://s3-cn-east-1.qiniucs.com', 'cn-east-1',
     'sdkwork-drive-qiniu-kodo', FALSE, TRUE,
     'env:QINIU_ACCESS_KEY:QINIU_SECRET_KEY', NULL,
     'AES256', 'STANDARD', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-china-mobile-ecloud', '100001', 'china_mobile_ecloud',
     'Built-in China Mobile Ecloud', 'https://eos-wuxi-1.cmecloud.cn', 'wuxi-1',
     'sdkwork-drive-ecloud', TRUE, TRUE,
     'env:ECLOUD_ACCESS_KEY:ECLOUD_SECRET_KEY', NULL,
     'AES256', 'STANDARD', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-china-telecom-eos', '100001', 'china_telecom_eos',
     'Built-in China Telecom EOS', 'https://ooscn-shanghai.ctyunapi.cn', 'shanghai',
     'sdkwork-drive-ctyun-eos', TRUE, TRUE,
     'env:CTYUN_ACCESS_KEY:CTYUN_SECRET_KEY', NULL,
     'AES256', 'STANDARD', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-china-unicom-wo', '100001', 'china_unicom_wo',
     'Built-in China Unicom Wo Cloud', 'https://oss-cn-hangzhou.wocloud.com', 'hangzhou',
     'sdkwork-drive-unicom-wo', TRUE, TRUE,
     'env:WO_ACCESS_KEY:WO_SECRET_KEY', NULL,
     'AES256', 'STANDARD', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-minio', '100001', 'minio',
     'Built-in MinIO', 'https://minio.internal.sdkwork.local', 'us-east-1',
     'sdkwork-drive-minio', TRUE, TRUE,
     'env:MINIO_ROOT_USER:MINIO_ROOT_PASSWORD', NULL,
     'AES256', 'STANDARD', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-cloudflare-r2', '100001', 'cloudflare_r2',
     'Built-in Cloudflare R2', 'https://placeholder.r2.cloudflarestorage.com', 'auto',
     'sdkwork-drive-r2', TRUE, TRUE,
     'env:R2_ACCESS_KEY_ID:R2_SECRET_ACCESS_KEY', NULL,
     'AES256', 'STANDARD', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-backblaze-b2', '100001', 'backblaze_b2',
     'Built-in Backblaze B2', 'https://s3.us-west-004.backblazeb2.com', 'us-west-004',
     'sdkwork-drive-b2', FALSE, TRUE,
     'env:B2_APPLICATION_KEY_ID:B2_APPLICATION_KEY', NULL,
     'AES256', 'STANDARD', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-wasabi', '100001', 'wasabi',
     'Built-in Wasabi', 'https://s3.us-east-1.wasabisys.com', 'us-east-1',
     'sdkwork-drive-wasabi', FALSE, TRUE,
     'env:WASABI_ACCESS_KEY:WASABI_SECRET_KEY', NULL,
     'AES256', 'STANDARD', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-digitalocean-spaces', '100001', 'digitalocean_spaces',
     'Built-in DigitalOcean Spaces', 'https://nyc3.digitaloceanspaces.com', 'nyc3',
     'sdkwork-drive-do-spaces', FALSE, TRUE,
     'env:SPACES_ACCESS_KEY_ID:SPACES_SECRET_ACCESS_KEY', NULL,
     'AES256', 'STANDARD', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-linode-object-storage', '100001', 'linode_object_storage',
     'Built-in Akamai / Linode Object Storage', 'https://us-east-1.linodeobjects.com', 'us-east-1',
     'sdkwork-drive-linode', TRUE, TRUE,
     'env:LINODE_ACCESS_KEY:LINODE_SECRET_KEY', NULL,
     'AES256', 'STANDARD', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-vultr-object-storage', '100001', 'vultr_object_storage',
     'Built-in Vultr Object Storage', 'https://ewr1.vultrobjects.com', 'ewr1',
     'sdkwork-drive-vultr', TRUE, TRUE,
     'env:VULTR_ACCESS_KEY:VULTR_SECRET_KEY', NULL,
     'AES256', 'STANDARD', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-scaleway-object-storage', '100001', 'scaleway_object_storage',
     'Built-in Scaleway Object Storage', 'https://s3.fr-par.scw.cloud', 'fr-par',
     'sdkwork-drive-scaleway', FALSE, TRUE,
     'env:SCW_ACCESS_KEY:SCW_SECRET_KEY', NULL,
     'AES256', 'STANDARD', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-oracle-cloud-storage', '100001', 'oracle_cloud_storage',
     'Built-in Oracle Cloud Object Storage', 'https://placeholder.oraclecloud.com', 'us-ashburn-1',
     'sdkwork-drive-oci', FALSE, TRUE,
     'env:OCI_ACCESS_KEY:OCI_SECRET_KEY', NULL,
     'AES256', 'Standard', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-ibm-cos', '100001', 'ibm_cos',
     'Built-in IBM Cloud Object Storage', 'https://s3.us-east.cloud-object-storage.appdomain.cloud',
     'us-east', 'sdkwork-drive-ibm-cos', TRUE, TRUE,
     'env:IBM_COS_ACCESS_KEY_ID:IBM_COS_SECRET_ACCESS_KEY', NULL,
     'AES256', 'STANDARD', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-alibaba-cloud-international', '100001',
     'alibaba_cloud_international', 'Built-in Alibaba Cloud OSS (International)',
     'https://oss-ap-southeast-1.aliyuncs.com', 'ap-southeast-1',
     'sdkwork-drive-aliyun-intl', FALSE, TRUE,
     'env:ALIBABA_CLOUD_ACCESS_KEY_ID:ALIBABA_CLOUD_ACCESS_KEY_SECRET', NULL,
     'KMS', 'Standard', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed'),
    ('builtin-storage-provider-100001-tencent-cloud-international', '100001',
     'tencent_cloud_international', 'Built-in Tencent Cloud COS (International)',
     'https://cos.ap-singapore.myqcloud.com', 'ap-singapore',
     'sdkwork-drive-cos-intl-1250000000', FALSE, TRUE,
     'env:COS_SECRET_ID:COS_SECRET_KEY', NULL,
     'AES256', 'STANDARD', 'active', 1, 'sdkwork-storage-seed', 'sdkwork-storage-seed')
ON CONFLICT (id) DO NOTHING;

INSERT INTO dr_drive_storage_provider_binding (
    id, tenant_id, space_id, provider_id, binding_scope, purpose,
    storage_root_prefix, lifecycle_status, version, created_by, updated_by
)
SELECT
    'default:tenant:100001',
    '100001',
    NULL,
    'builtin-storage-provider-100001-tencent-cos',
    'tenant',
    'primary',
    'sdkwork-drive/v1/tenants/100001',
    'active',
    1,
    'sdkwork-storage-seed',
    'sdkwork-storage-seed'
WHERE NOT EXISTS (
    SELECT 1
    FROM dr_drive_storage_provider_binding
    WHERE tenant_id = '100001'
      AND space_id IS NULL
      AND purpose = 'primary'
      AND lifecycle_status = 'active'
)
ON CONFLICT DO NOTHING;
