-- sdkwork:migration
-- id: 0004_drive_storage_provider_kind_catalog
-- engine: postgres
-- module: sdkwork-drive
-- purpose: Reverse 0004_drive_storage_provider_kind_catalog.up.sql by restoring
--   the original seven-kind whitelist.
--   Not reversible in the data sense: once a row uses one of the added kinds,
--   narrowing the constraint makes that row fail validation, so the down path
--   deletes the added catalog rows first and then narrows. A provider
--   configuration still on an added kind would make the ADD CONSTRAINT fail,
--   which is the intended outcome — it surfaces the dependency instead of
--   silently discarding it.
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

DELETE FROM dr_drive_storage_provider_kind
WHERE provider_kind IN (
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
);

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
                'volcengine_tos'
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
                'volcengine_tos'
            )
        );

COMMIT;
