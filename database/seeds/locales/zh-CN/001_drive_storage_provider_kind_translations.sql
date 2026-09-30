-- Locale seed: zh-CN display names for the built-in storage provider-kind
-- catalog (I18N_SPEC.md §11, DATABASE_SPEC.md §6.4.1). Idempotent upsert: the
-- version bumps only when a display name actually changes, so re-running the
-- seed never rewrites unchanged rows. Base rows come from the baseline DDL /
-- migration 0008, which this file's foreign key requires.

INSERT INTO dr_drive_storage_provider_kind_translation (provider_kind, locale, display_name) VALUES
    ('local_filesystem', 'zh-CN', '本地文件系统'),
    ('s3_compatible', 'zh-CN', 'Amazon S3 / S3 兼容'),
    ('aliyun_oss', 'zh-CN', '阿里云 OSS'),
    ('tencent_cos', 'zh-CN', '腾讯云 COS'),
    ('huawei_obs', 'zh-CN', '华为云 OBS'),
    ('volcengine_tos', 'zh-CN', '火山引擎 TOS'),
    ('baidu_bos', 'zh-CN', '百度智能云 BOS'),
    ('kingsoft_ks3', 'zh-CN', '金山云 KS3'),
    ('qiniu_kodo', 'zh-CN', '七牛云 Kodo'),
    ('china_mobile_ecloud', 'zh-CN', '移动云 EOS'),
    ('china_telecom_eos', 'zh-CN', '天翼云 EOS'),
    ('china_unicom_wo', 'zh-CN', '联通云沃存储'),
    ('minio', 'zh-CN', 'MinIO'),
    ('cloudflare_r2', 'zh-CN', 'Cloudflare R2'),
    ('backblaze_b2', 'zh-CN', 'Backblaze B2'),
    ('wasabi', 'zh-CN', 'Wasabi'),
    ('digitalocean_spaces', 'zh-CN', 'DigitalOcean Spaces'),
    ('linode_object_storage', 'zh-CN', 'Akamai / Linode 对象存储'),
    ('vultr_object_storage', 'zh-CN', 'Vultr 对象存储'),
    ('scaleway_object_storage', 'zh-CN', 'Scaleway 对象存储'),
    ('oracle_cloud_storage', 'zh-CN', 'Oracle 云基础设施对象存储'),
    ('ibm_cos', 'zh-CN', 'IBM Cloud 对象存储'),
    ('alibaba_cloud_international', 'zh-CN', '阿里云 OSS（国际版）'),
    ('tencent_cloud_international', 'zh-CN', '腾讯云 COS（国际版）'),
    ('google_cloud_storage', 'zh-CN', 'Google Cloud Storage')
ON CONFLICT (provider_kind, locale) DO UPDATE
SET display_name = EXCLUDED.display_name,
    version = dr_drive_storage_provider_kind_translation.version + 1,
    updated_at = CURRENT_TIMESTAMP
WHERE dr_drive_storage_provider_kind_translation.display_name
    IS DISTINCT FROM EXCLUDED.display_name;
