import type { StorageProviderHealthStatus, StorageProviderKind } from '../types/storageProviderAdminTypes';

export interface ProviderRegion {
  value: string;
  label: string;
}

export interface ProviderCredentialFieldMeta {
  accessKeyLabel: string;
  secretKeyLabel: string;
  accessKeyPlaceholder: string;
  secretKeyPlaceholder: string;
  defaultEnvAccessKey: string;
  defaultEnvSecretKey: string;
  consoleUrl?: string;
  bucketHint?: string;
}

export interface ProviderKindMeta {
  value: StorageProviderKind;
  label: string;
  shortLabel: string;
  icon: string;
  color: string;
  bgClass: string;
  textClass: string;
  endpointHint: string;
  regionHint: string;
  regions: ProviderRegion[];
  credentialHint: string;
  credentialLabel: string;
  credentialFields?: ProviderCredentialFieldMeta;
  sseModes: string[];
  storageClasses: string[];
  features: {
    hasRegion: boolean;
    hasPathStyle: boolean;
    hasSse: boolean;
    hasStorageClass: boolean;
    isLocal: boolean;
    structuredCredentials: boolean;
  };
}

const PROVIDER_KIND_META: ProviderKindMeta[] = [
  {
    value: 's3_compatible',
    label: 'Amazon S3 / S3 Compatible',
    shortLabel: 'S3',
    icon: 'S3',
    color: '#FF9900',
    bgClass: 'bg-orange-50 dark:bg-orange-950/30',
    textClass: 'text-orange-700 dark:text-orange-300',
    endpointHint: 'https://s3.amazonaws.com',
    regionHint: 'us-east-1',
    regions: [
      { value: 'us-east-1', label: 'US East (N. Virginia) us-east-1' },
      { value: 'us-east-2', label: 'US East (Ohio) us-east-2' },
      { value: 'us-west-1', label: 'US West (N. California) us-west-1' },
      { value: 'us-west-2', label: 'US West (Oregon) us-west-2' },
      { value: 'af-south-1', label: 'Africa (Cape Town) af-south-1' },
      { value: 'ap-east-1', label: 'Asia Pacific (Hong Kong) ap-east-1' },
      { value: 'ap-south-1', label: 'Asia Pacific (Mumbai) ap-south-1' },
      { value: 'ap-south-2', label: 'Asia Pacific (Hyderabad) ap-south-2' },
      { value: 'ap-southeast-1', label: 'Asia Pacific (Singapore) ap-southeast-1' },
      { value: 'ap-southeast-2', label: 'Asia Pacific (Sydney) ap-southeast-2' },
      { value: 'ap-southeast-3', label: 'Asia Pacific (Jakarta) ap-southeast-3' },
      { value: 'ap-northeast-1', label: 'Asia Pacific (Tokyo) ap-northeast-1' },
      { value: 'ap-northeast-2', label: 'Asia Pacific (Seoul) ap-northeast-2' },
      { value: 'ap-northeast-3', label: 'Asia Pacific (Osaka) ap-northeast-3' },
      { value: 'ca-central-1', label: 'Canada (Central) ca-central-1' },
      { value: 'eu-central-1', label: 'Europe (Frankfurt) eu-central-1' },
      { value: 'eu-central-2', label: 'Europe (Zurich) eu-central-2' },
      { value: 'eu-west-1', label: 'Europe (Ireland) eu-west-1' },
      { value: 'eu-west-2', label: 'Europe (London) eu-west-2' },
      { value: 'eu-west-3', label: 'Europe (Paris) eu-west-3' },
      { value: 'eu-south-1', label: 'Europe (Milan) eu-south-1' },
      { value: 'eu-south-2', label: 'Europe (Spain) eu-south-2' },
      { value: 'eu-north-1', label: 'Europe (Stockholm) eu-north-1' },
      { value: 'me-south-1', label: 'Middle East (Bahrain) me-south-1' },
      { value: 'me-central-1', label: 'Middle East (UAE) me-central-1' },
      { value: 'sa-east-1', label: 'South America (São Paulo) sa-east-1' },
    ],
    credentialHint: 'env:AWS_ACCESS_KEY_ID:AWS_SECRET_ACCESS_KEY or plain:access_key:secret_key',
    credentialLabel: 'AWS Access Key',
    credentialFields: {
      accessKeyLabel: 'Access Key ID',
      secretKeyLabel: 'Secret Access Key',
      accessKeyPlaceholder: 'AKIAxxxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'AWS_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'AWS_SECRET_ACCESS_KEY',
      consoleUrl: 'https://console.aws.amazon.com/iam/home#/security_credentials',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
    },
    sseModes: ['AES256', 'aws:kms', 'aws:kms:dsse'],
    storageClasses: ['STANDARD', 'STANDARD_IA', 'ONEZONE_IA', 'INTELLIGENT_TIERING', 'GLACIER', 'DEEP_ARCHIVE'],
    features: {
      hasRegion: true,
      hasPathStyle: true,
      hasSse: true,
      hasStorageClass: true,
      isLocal: false,
      structuredCredentials: true,
    },
  },
  {
    value: 'aliyun_oss',
    label: 'Alibaba Cloud OSS',
    shortLabel: 'OSS',
    icon: 'OSS',
    color: '#FF6A00',
    bgClass: 'bg-orange-50 dark:bg-orange-950/30',
    textClass: 'text-orange-700 dark:text-orange-300',
    endpointHint: 'https://oss-cn-hangzhou.aliyuncs.com',
    regionHint: 'cn-hangzhou',
    regions: [
      { value: 'cn-hangzhou', label: '华东1（杭州）cn-hangzhou' },
      { value: 'cn-shanghai', label: '华东2（上海）cn-shanghai' },
      { value: 'cn-nanjing-1', label: '华东5（南京）cn-nanjing-1' },
      { value: 'cn-fuzhou', label: '华东6（福州）cn-fuzhou' },
      { value: 'cn-beijing', label: '华北2（北京）cn-beijing' },
      { value: 'cn-zhangjiakou', label: '华北3（张家口）cn-zhangjiakou' },
      { value: 'cn-huhehaote', label: '华北5（呼和浩特）cn-huhehaote' },
      { value: 'cn-wulanchabu', label: '华北6（乌兰察布）cn-wulanchabu' },
      { value: 'cn-shenzhen', label: '华南1（深圳）cn-shenzhen' },
      { value: 'cn-heyuan', label: '华南2（河源）cn-heyuan' },
      { value: 'cn-guangzhou', label: '华南3（广州）cn-guangzhou' },
      { value: 'cn-chengdu', label: '西南1（成都）cn-chengdu' },
      { value: 'cn-hongkong', label: '中国香港 cn-hongkong' },
      { value: 'ap-southeast-1', label: '新加坡 ap-southeast-1' },
      { value: 'ap-southeast-2', label: '悉尼 ap-southeast-2' },
      { value: 'ap-southeast-3', label: '吉隆坡 ap-southeast-3' },
      { value: 'ap-southeast-5', label: '雅加达 ap-southeast-5' },
      { value: 'ap-southeast-6', label: '马尼拉 ap-southeast-6' },
      { value: 'ap-southeast-7', label: '曼谷 ap-southeast-7' },
      { value: 'ap-northeast-1', label: '东京 ap-northeast-1' },
      { value: 'ap-northeast-2', label: '首尔 ap-northeast-2' },
      { value: 'ap-south-1', label: '孟买 ap-south-1' },
      { value: 'eu-central-1', label: '法兰克福 eu-central-1' },
      { value: 'eu-west-1', label: '伦敦 eu-west-1' },
      { value: 'me-east-1', label: '迪拜 me-east-1' },
      { value: 'us-east-1', label: '弗吉尼亚 us-east-1' },
      { value: 'us-west-1', label: '硅谷 us-west-1' },
    ],
    credentialHint: 'env:OSS_ACCESS_KEY_ID:OSS_ACCESS_KEY_SECRET or plain:access_key:secret_key',
    credentialLabel: 'AccessKey',
    credentialFields: {
      accessKeyLabel: 'AccessKey ID',
      secretKeyLabel: 'AccessKey Secret',
      accessKeyPlaceholder: 'LTAIxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'OSS_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'OSS_ACCESS_KEY_SECRET',
      consoleUrl: 'https://ram.console.aliyun.com/manage/ak',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
    },
    sseModes: ['KMS', 'AES256'],
    storageClasses: ['Standard', 'IA', 'Archive', 'ColdArchive', 'DeepColdArchive'],
    features: {
      hasRegion: true,
      hasPathStyle: false,
      hasSse: true,
      hasStorageClass: true,
      isLocal: false,
      structuredCredentials: true,
    },
  },
  {
    value: 'tencent_cos',
    label: 'Tencent Cloud COS',
    shortLabel: 'COS',
    icon: 'COS',
    color: '#006EFF',
    bgClass: 'bg-blue-50 dark:bg-blue-950/30',
    textClass: 'text-blue-700 dark:text-blue-300',
    endpointHint: 'https://cos.ap-guangzhou.myqcloud.com',
    regionHint: 'ap-guangzhou',
    regions: [
      { value: 'ap-beijing', label: '华北（北京）ap-beijing' },
      { value: 'ap-beijing-1', label: '华北（北京）ap-beijing-1' },
      { value: 'ap-nanjing', label: '华东（南京）ap-nanjing' },
      { value: 'ap-shanghai', label: '华东（上海）ap-shanghai' },
      { value: 'ap-guangzhou', label: '华南（广州）ap-guangzhou' },
      { value: 'ap-chengdu', label: '西南（成都）ap-chengdu' },
      { value: 'ap-chongqing', label: '西南（重庆）ap-chongqing' },
      { value: 'ap-hongkong', label: '中国香港 ap-hongkong' },
      { value: 'ap-singapore', label: '新加坡 ap-singapore' },
      { value: 'ap-mumbai', label: '孟买 ap-mumbai' },
      { value: 'ap-jakarta', label: '雅加达 ap-jakarta' },
      { value: 'ap-seoul', label: '首尔 ap-seoul' },
      { value: 'ap-tokyo', label: '东京 ap-tokyo' },
      { value: 'na-siliconvalley', label: '硅谷 na-siliconvalley' },
      { value: 'na-ashburn', label: '弗吉尼亚 na-ashburn' },
      { value: 'sa-saopaulo', label: '圣保罗 sa-saopaulo' },
      { value: 'eu-frankfurt', label: '法兰克福 eu-frankfurt' },
      { value: 'eu-moscow', label: '莫斯科 eu-moscow' },
    ],
    credentialHint: 'env:COS_SECRET_ID:COS_SECRET_KEY or plain:secret_id:secret_key',
    credentialLabel: 'API 密钥',
    credentialFields: {
      accessKeyLabel: 'SecretId',
      secretKeyLabel: 'SecretKey',
      accessKeyPlaceholder: 'AKIDxxxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'COS_SECRET_ID',
      defaultEnvSecretKey: 'COS_SECRET_KEY',
      consoleUrl: 'https://console.cloud.tencent.com/cam/capi',
      bucketHint: 'bucketName-appId',
    },
    sseModes: ['AES256', 'KMS'],
    storageClasses: ['STANDARD', 'STANDARD_IA', 'ARCHIVE', 'DEEP_ARCHIVE'],
    features: {
      hasRegion: true,
      hasPathStyle: false,
      hasSse: true,
      hasStorageClass: true,
      isLocal: false,
      structuredCredentials: true,
    },
  },
  {
    value: 'huawei_obs',
    label: 'Huawei Cloud OBS',
    shortLabel: 'OBS',
    icon: 'OBS',
    color: '#CF0A2C',
    bgClass: 'bg-red-50 dark:bg-red-950/30',
    textClass: 'text-red-700 dark:text-red-300',
    endpointHint: 'https://obs.cn-north-1.myhuaweicloud.com',
    regionHint: 'cn-north-1',
    regions: [
      { value: 'cn-north-1', label: '华北-北京一 cn-north-1' },
      { value: 'cn-north-4', label: '华北-北京四 cn-north-4' },
      { value: 'cn-north-2', label: '华北-乌兰察布一 cn-north-2' },
      { value: 'cn-east-2', label: '华东-上海二 cn-east-2' },
      { value: 'cn-east-3', label: '华东-上海一 cn-east-3' },
      { value: 'cn-south-1', label: '华南-广州 cn-south-1' },
      { value: 'cn-south-2', label: '华南-深圳 cn-south-2' },
      { value: 'cn-southwest-2', label: '西南-贵阳一 cn-southwest-2' },
      { value: 'ap-southeast-1', label: '中国香港 ap-southeast-1' },
      { value: 'ap-southeast-2', label: '曼谷 ap-southeast-2' },
      { value: 'ap-southeast-3', label: '新加坡 ap-southeast-3' },
      { value: 'af-south-1', label: '约翰内斯堡 af-south-1' },
      { value: 'na-mexico-1', label: '墨西哥城一 na-mexico-1' },
      { value: 'la-south-2', label: '圣地亚哥 la-south-2' },
      { value: 'sa-brazil-1', label: '圣保罗一 sa-brazil-1' },
      { value: 'tr-west-1', label: '伊斯坦布尔 tr-west-1' },
      { value: 'ae-ad-1', label: '阿布扎比一 ae-ad-1' },
      { value: 'ap-southeast-4', label: '雅加达 ap-southeast-4' },
      { value: 'me-east-1', label: '利雅得 me-east-1' },
    ],
    credentialHint: 'env:OBS_ACCESS_KEY_ID:OBS_SECRET_ACCESS_KEY or plain:access_key:secret_key',
    credentialLabel: 'AK/SK',
    credentialFields: {
      accessKeyLabel: 'Access Key ID',
      secretKeyLabel: 'Secret Access Key',
      accessKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'OBS_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'OBS_SECRET_ACCESS_KEY',
      consoleUrl: 'https://console.huaweicloud.com/iam/#/mine/accessKey',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
    },
    sseModes: ['kms', 'AES256'],
    storageClasses: ['STANDARD', 'WARM', 'COLD'],
    features: {
      hasRegion: true,
      hasPathStyle: true,
      hasSse: true,
      hasStorageClass: true,
      isLocal: false,
      structuredCredentials: true,
    },
  },
  {
    value: 'volcengine_tos',
    label: 'Volcengine TOS',
    shortLabel: 'TOS',
    icon: 'TOS',
    color: '#0052D9',
    bgClass: 'bg-blue-50 dark:bg-blue-950/30',
    textClass: 'text-blue-700 dark:text-blue-300',
    endpointHint: 'https://tos-cn-beijing.volces.com',
    regionHint: 'cn-beijing',
    regions: [
      { value: 'cn-beijing', label: '华北（北京）cn-beijing' },
      { value: 'cn-shanghai', label: '华东（上海）cn-shanghai' },
      { value: 'cn-guangzhou', label: '华南（广州）cn-guangzhou' },
      { value: 'cn-chengdu', label: '西南（成都）cn-chengdu' },
      { value: 'cn-nanjing', label: '华东（南京）cn-nanjing' },
      { value: 'cn-hongkong', label: '中国香港 cn-hongkong' },
      { value: 'ap-singapore', label: '新加坡 ap-singapore' },
      { value: 'ap-tokyo', label: '东京 ap-tokyo' },
      { value: 'ap-seoul', label: '首尔 ap-seoul' },
      { value: 'ap-mumbai', label: '孟买 ap-mumbai' },
      { value: 'ap-bangkok', label: '曼谷 ap-bangkok' },
      { value: 'eu-amsterdam', label: '阿姆斯特丹 eu-amsterdam' },
      { value: 'us-east-1', label: '美东（弗吉尼亚）us-east-1' },
      { value: 'us-west-1', label: '美西（硅谷）us-west-1' },
    ],
    credentialHint: 'env:TOS_ACCESS_KEY_ID:TOS_SECRET_ACCESS_KEY or plain:access_key:secret_key',
    credentialLabel: 'Access Key',
    credentialFields: {
      accessKeyLabel: 'Access Key ID',
      secretKeyLabel: 'Secret Access Key',
      accessKeyPlaceholder: 'AKxxxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'TOS_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'TOS_SECRET_ACCESS_KEY',
      consoleUrl: 'https://console.volcengine.com/iam/keymanage/',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
    },
    sseModes: ['AES256', 'KMS'],
    storageClasses: ['STANDARD', 'IA', 'ARCHIVE', 'DEEP_ARCHIVE'],
    features: {
      hasRegion: true,
      hasPathStyle: false,
      hasSse: true,
      hasStorageClass: true,
      isLocal: false,
      structuredCredentials: true,
    },
  },
  {
    value: 'google_cloud_storage',
    label: 'Google Cloud Storage',
    shortLabel: 'GCS',
    icon: 'GCS',
    color: '#4285F4',
    bgClass: 'bg-blue-50 dark:bg-blue-950/30',
    textClass: 'text-blue-700 dark:text-blue-300',
    endpointHint: 'https://storage.googleapis.com',
    regionHint: 'us-central1',
    regions: [
      { value: 'us-central1', label: 'Iowa us-central1' },
      { value: 'us-east1', label: 'South Carolina us-east1' },
      { value: 'us-east4', label: 'Northern Virginia us-east4' },
      { value: 'us-east5', label: 'Columbus us-east5' },
      { value: 'us-south1', label: 'Dallas us-south1' },
      { value: 'us-west1', label: 'Oregon us-west1' },
      { value: 'us-west2', label: 'Los Angeles us-west2' },
      { value: 'us-west3', label: 'Salt Lake City us-west3' },
      { value: 'us-west4', label: 'Las Vegas us-west4' },
      { value: 'northamerica-northeast1', label: 'Montreal northamerica-northeast1' },
      { value: 'northamerica-northeast2', label: 'Toronto northamerica-northeast2' },
      { value: 'southamerica-east1', label: 'São Paulo southamerica-east1' },
      { value: 'southamerica-west1', label: 'Santiago southamerica-west1' },
      { value: 'europe-central2', label: 'Warsaw europe-central2' },
      { value: 'europe-north1', label: 'Finland europe-north1' },
      { value: 'europe-southwest1', label: 'Madrid europe-southwest1' },
      { value: 'europe-west1', label: 'Belgium europe-west1' },
      { value: 'europe-west2', label: 'London europe-west2' },
      { value: 'europe-west3', label: 'Frankfurt europe-west3' },
      { value: 'europe-west4', label: 'Netherlands europe-west4' },
      { value: 'europe-west6', label: 'Zurich europe-west6' },
      { value: 'europe-west8', label: 'Milan europe-west8' },
      { value: 'europe-west9', label: 'Paris europe-west9' },
      { value: 'asia-east1', label: 'Taiwan asia-east1' },
      { value: 'asia-east2', label: 'Hong Kong asia-east2' },
      { value: 'asia-northeast1', label: 'Tokyo asia-northeast1' },
      { value: 'asia-northeast2', label: 'Osaka asia-northeast2' },
      { value: 'asia-northeast3', label: 'Seoul asia-northeast3' },
      { value: 'asia-south1', label: 'Mumbai asia-south1' },
      { value: 'asia-south2', label: 'Delhi asia-south2' },
      { value: 'asia-southeast1', label: 'Singapore asia-southeast1' },
      { value: 'asia-southeast2', label: 'Jakarta asia-southeast2' },
      { value: 'australia-southeast1', label: 'Sydney australia-southeast1' },
      { value: 'australia-southeast2', label: 'Melbourne australia-southeast2' },
      { value: 'me-west1', label: 'Tel Aviv me-west1' },
      { value: 'me-central1', label: 'Doha me-central1' },
      { value: 'me-central2', label: 'Dammam me-central2' },
      { value: 'africa-south1', label: 'Johannesburg africa-south1' },
    ],
    credentialHint: 'env:GOOGLE_ACCESS_KEY_ID:GOOGLE_SECRET_ACCESS_KEY or secret:gcs-hmac',
    credentialLabel: 'HMAC Key',
    credentialFields: {
      accessKeyLabel: 'HMAC Access ID',
      secretKeyLabel: 'HMAC Secret',
      accessKeyPlaceholder: 'GOOGxxxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'GOOGLE_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'GOOGLE_SECRET_ACCESS_KEY',
      consoleUrl: 'https://console.cloud.google.com/storage/settings;tab=interoperability',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens, underscores',
    },
    sseModes: ['AES256', 'GOOGLE_DEFAULT_ENCRYPTION'],
    storageClasses: ['STANDARD', 'NEARLINE', 'COLDLINE', 'ARCHIVE'],
    features: {
      hasRegion: true,
      hasPathStyle: false,
      hasSse: true,
      hasStorageClass: true,
      isLocal: false,
      structuredCredentials: true,
    },
  },
  {
    value: 'local_filesystem',
    label: 'Local Filesystem',
    shortLabel: 'Local',
    icon: 'LOCAL',
    color: '#6B7280',
    bgClass: 'bg-neutral-100 dark:bg-neutral-800/50',
    textClass: 'text-neutral-700 dark:text-neutral-300',
    endpointHint: '/var/data/drive-storage',
    regionHint: '',
    regions: [],
    credentialHint: '',
    credentialLabel: '',
    sseModes: [],
    storageClasses: [],
    features: {
      hasRegion: false,
      hasPathStyle: false,
      hasSse: false,
      hasStorageClass: false,
      isLocal: true,
      structuredCredentials: false,
    },
  },
];

const CUSTOM_PROVIDER_META: ProviderKindMeta = {
  value: 'custom',
  label: 'Custom S3-compatible',
  shortLabel: 'Custom',
  icon: 'S3',
  color: '#8B5CF6',
  bgClass: 'bg-purple-50 dark:bg-purple-950/30',
  textClass: 'text-purple-700 dark:text-purple-300',
  endpointHint: 'https://custom-endpoint.example.com',
  regionHint: 'us-east-1',
  regions: [],
  credentialHint: 'env:CUSTOM_ACCESS_KEY_ID:CUSTOM_SECRET_ACCESS_KEY or plain:key:secret',
  credentialLabel: 'Credential',
  credentialFields: {
    accessKeyLabel: 'Access Key ID',
    secretKeyLabel: 'Secret Access Key',
    accessKeyPlaceholder: 'access-key-id',
    secretKeyPlaceholder: 'secret-access-key',
    defaultEnvAccessKey: 'CUSTOM_ACCESS_KEY_ID',
    defaultEnvSecretKey: 'CUSTOM_SECRET_ACCESS_KEY',
  },
  sseModes: ['AES256'],
  storageClasses: ['STANDARD'],
  features: {
    hasRegion: true,
    hasPathStyle: true,
    hasSse: true,
    hasStorageClass: true,
    isLocal: false,
    structuredCredentials: true,
  },
};

const REGION_ENDPOINT_BUILDERS: Record<string, (region: string) => string> = {
  s3_compatible: (region) =>
    region === 'us-east-1' ? 'https://s3.amazonaws.com' : `https://s3.${region}.amazonaws.com`,
  aliyun_oss: (region) => `https://oss-${region}.aliyuncs.com`,
  tencent_cos: (region) => `https://cos.${region}.myqcloud.com`,
  huawei_obs: (region) => `https://obs.${region}.myhuaweicloud.com`,
  volcengine_tos: (region) => `https://tos-${region}.volces.com`,
};

export function buildProviderEndpointUrl(kind: string, region: string): string | undefined {
  const normalizedKind = kind.startsWith('custom:') ? 'custom' : kind;
  const builder = REGION_ENDPOINT_BUILDERS[normalizedKind];
  if (!builder || !region.trim()) {
    return undefined;
  }
  return builder(region.trim());
}

export function getProviderKindMeta(kind: string): ProviderKindMeta {
  if (kind?.startsWith('custom:')) {
    return { ...CUSTOM_PROVIDER_META, label: `Custom: ${kind.substring(7)}`, shortLabel: kind.substring(7) };
  }
  return PROVIDER_KIND_META.find((m) => m.value === kind) ?? CUSTOM_PROVIDER_META;
}

export function getAllProviderKindMeta(): ProviderKindMeta[] {
  return [...PROVIDER_KIND_META, CUSTOM_PROVIDER_META];
}

export function resolveProviderKindMeta(kind: StorageProviderKind): ProviderKindMeta {
  return getProviderKindMeta(kind);
}

export const HEALTH_STATUS_CONFIG: Record<StorageProviderHealthStatus, {
  label: string;
  icon: string;
  dotClass: string;
  bgClass: string;
  textClass: string;
}> = {
  unknown: {
    label: 'Unknown',
    icon: '?',
    dotClass: 'bg-neutral-400',
    bgClass: 'bg-neutral-100 dark:bg-neutral-800',
    textClass: 'text-neutral-600 dark:text-neutral-400',
  },
  healthy: {
    label: 'Healthy',
    icon: '\u2713',
    dotClass: 'bg-emerald-500',
    bgClass: 'bg-emerald-50 dark:bg-emerald-950/30',
    textClass: 'text-emerald-700 dark:text-emerald-300',
  },
  degraded: {
    label: 'Degraded',
    icon: '!',
    dotClass: 'bg-amber-500',
    bgClass: 'bg-amber-50 dark:bg-amber-950/30',
    textClass: 'text-amber-700 dark:text-amber-300',
  },
  unreachable: {
    label: 'Unreachable',
    icon: '\u2717',
    dotClass: 'bg-red-500',
    bgClass: 'bg-red-50 dark:bg-red-950/30',
    textClass: 'text-red-700 dark:text-red-300',
  },
};

export function formatRelativeTime(epochMs?: number): string {
  if (!epochMs) return 'never';
  const seconds = Math.floor((Date.now() - epochMs) / 1000);
  if (seconds < 60) return 'just now';
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
  return `${Math.floor(seconds / 86400)}d ago`;
}
