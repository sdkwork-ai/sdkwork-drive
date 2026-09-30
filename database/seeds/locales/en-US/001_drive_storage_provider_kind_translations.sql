-- Locale seed: en-US display names for the built-in storage provider-kind
-- catalog (I18N_SPEC.md §11, DATABASE_SPEC.md §6.4.1). Idempotent upsert: the
-- version bumps only when a display name actually changes, so re-running the
-- seed never rewrites unchanged rows. Base rows come from the baseline DDL /
-- migration 0008, which this file's foreign key requires. The en-US labels
-- mirror the locale-neutral base display names on purpose, so an operator
-- editing the base table keeps the two in step.

INSERT INTO dr_drive_storage_provider_kind_translation (provider_kind, locale, display_name) VALUES
    ('local_filesystem', 'en-US', 'Local Filesystem'),
    ('s3_compatible', 'en-US', 'Amazon S3 / S3 Compatible'),
    ('aliyun_oss', 'en-US', 'Alibaba Cloud OSS'),
    ('tencent_cos', 'en-US', 'Tencent Cloud COS'),
    ('huawei_obs', 'en-US', 'Huawei Cloud OBS'),
    ('volcengine_tos', 'en-US', 'Volcengine TOS'),
    ('baidu_bos', 'en-US', 'Baidu Cloud BOS'),
    ('kingsoft_ks3', 'en-US', 'Kingsoft Cloud KS3'),
    ('qiniu_kodo', 'en-US', 'Qiniu Kodo'),
    ('china_mobile_ecloud', 'en-US', 'China Mobile Ecloud'),
    ('china_telecom_eos', 'en-US', 'China Telecom EOS'),
    ('china_unicom_wo', 'en-US', 'China Unicom Wo Cloud'),
    ('minio', 'en-US', 'MinIO'),
    ('cloudflare_r2', 'en-US', 'Cloudflare R2'),
    ('backblaze_b2', 'en-US', 'Backblaze B2'),
    ('wasabi', 'en-US', 'Wasabi'),
    ('digitalocean_spaces', 'en-US', 'DigitalOcean Spaces'),
    ('linode_object_storage', 'en-US', 'Akamai / Linode Object Storage'),
    ('vultr_object_storage', 'en-US', 'Vultr Object Storage'),
    ('scaleway_object_storage', 'en-US', 'Scaleway Object Storage'),
    ('oracle_cloud_storage', 'en-US', 'Oracle Cloud Infrastructure Object Storage'),
    ('ibm_cos', 'en-US', 'IBM Cloud Object Storage'),
    ('alibaba_cloud_international', 'en-US', 'Alibaba Cloud OSS (International)'),
    ('tencent_cloud_international', 'en-US', 'Tencent Cloud COS (International)'),
    ('google_cloud_storage', 'en-US', 'Google Cloud Storage')
ON CONFLICT (provider_kind, locale) DO UPDATE
SET display_name = EXCLUDED.display_name,
    version = dr_drive_storage_provider_kind_translation.version + 1,
    updated_at = CURRENT_TIMESTAMP
WHERE dr_drive_storage_provider_kind_translation.display_name
    IS DISTINCT FROM EXCLUDED.display_name;
