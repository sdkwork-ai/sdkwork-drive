-- sdkwork:migration
-- id: 0004_drive_storage_provider_kind_catalog
-- engine: postgres
-- module: sdkwork-drive
-- purpose: Widen the storage provider-kind whitelist from the original seven
--   kinds to the full S3-compatible vendor catalog. Both
--   dr_drive_storage_provider.provider_kind and
--   dr_drive_storage_provider_kind.provider_kind carried a hard-coded IN list,
--   so a named vendor (MinIO, Cloudflare R2, Wasabi, Baidu BOS, ...) could only
--   be configured through the free-form `custom:<vendor>` escape hatch, losing
--   the per-vendor defaults the console shows. The `custom:` branch is kept
--   unchanged so existing rows and keys stay valid.
--   The catalog seed rows themselves are inserted by the application
--   (`initialize_storage_provider_kinds`), which is idempotent; this migration
--   only relaxes the two CHECK constraints.
-- reversible: true
-- rollback: down-migration
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

ALTER TABLE dr_drive_storage_provider
    DROP CONSTRAINT IF EXISTS ck_dr_drive_storage_provider_provider_kind;

ALTER TABLE dr_drive_storage_provider
    ADD CONSTRAINT ck_dr_drive_storage_provider_provider_kind
        CHECK (
            provider_kind IN (
                'local_filesystem',
                's3_compatible',
                'google_cloud_storage',
                'aliyun_oss',
                'tencent_cos',
                'huawei_obs',
                'volcengine_tos',
                'baidu_bos',
                'kingsoft_ks3',
                'qiniu_kodo',
                'china_mobile_ecloud',
                'china_telecom_eos',
                'china_unicom_wo',
                'minio',
                'cloudflare_r2',
                'backblaze_b2',
                'wasabi',
                'digitalocean_spaces',
                'linode_object_storage',
                'vultr_object_storage',
                'scaleway_object_storage',
                'oracle_cloud_storage',
                'ibm_cos',
                'alibaba_cloud_international',
                'tencent_cloud_international'
            )
            OR provider_kind ~ '^custom:[a-z0-9_-]{2,32}$'
        );

ALTER TABLE dr_drive_storage_provider_kind
    DROP CONSTRAINT IF EXISTS ck_dr_drive_storage_provider_kind_provider_kind;

ALTER TABLE dr_drive_storage_provider_kind
    ADD CONSTRAINT ck_dr_drive_storage_provider_kind_provider_kind
        CHECK (
            provider_kind IN (
                'local_filesystem',
                's3_compatible',
                'google_cloud_storage',
                'aliyun_oss',
                'tencent_cos',
                'huawei_obs',
                'volcengine_tos',
                'baidu_bos',
                'kingsoft_ks3',
                'qiniu_kodo',
                'china_mobile_ecloud',
                'china_telecom_eos',
                'china_unicom_wo',
                'minio',
                'cloudflare_r2',
                'backblaze_b2',
                'wasabi',
                'digitalocean_spaces',
                'linode_object_storage',
                'vultr_object_storage',
                'scaleway_object_storage',
                'oracle_cloud_storage',
                'ibm_cos',
                'alibaba_cloud_international',
                'tencent_cloud_international'
            )
        );

COMMIT;
