import type {
  StorageProviderHealthStatus,
  StorageProviderKind,
  StorageProviderView,
} from '../types/storageProviderAdminTypes';

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
  /**
   * i18n key of the bucket-naming hint, preferred over the raw `bucketHint`
   * string when the storage dictionary carries a translation.
   */
  bucketHintKey?: string;
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
      bucketHintKey: 'bucketHint.dns',
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
    credentialHint: 'env:ALIBABA_CLOUD_ACCESS_KEY_ID:ALIBABA_CLOUD_ACCESS_KEY_SECRET or plain:access_key:secret_key',
    credentialLabel: 'AccessKey',
    credentialFields: {
      accessKeyLabel: 'AccessKey ID',
      secretKeyLabel: 'AccessKey Secret',
      accessKeyPlaceholder: 'LTAIxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'ALIBABA_CLOUD_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'ALIBABA_CLOUD_ACCESS_KEY_SECRET',
      consoleUrl: 'https://ram.console.aliyun.com/manage/ak',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
      bucketHintKey: 'bucketHint.dns',
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
      bucketHintKey: 'bucketHint.appId',
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
    credentialHint: 'env:HUAWEI_ACCESS_KEY_ID:HUAWEI_SECRET_ACCESS_KEY or plain:access_key:secret_key',
    credentialLabel: 'AK/SK',
    credentialFields: {
      accessKeyLabel: 'Access Key ID (AK)',
      secretKeyLabel: 'Secret Access Key (SK)',
      accessKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'HUAWEI_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'HUAWEI_SECRET_ACCESS_KEY',
      consoleUrl: 'https://console.huaweicloud.com/iam/#/myCredential',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
      bucketHintKey: 'bucketHint.dns',
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
    credentialHint: 'env:VOLC_ACCESSKEY:VOLC_SECRETKEY or plain:access_key:secret_key',
    credentialLabel: 'Access Key',
    credentialFields: {
      accessKeyLabel: 'Access Key ID (AK)',
      secretKeyLabel: 'Secret Access Key (SK)',
      accessKeyPlaceholder: 'AKxxxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'VOLC_ACCESSKEY',
      defaultEnvSecretKey: 'VOLC_SECRETKEY',
      consoleUrl: 'https://console.volcengine.com/iam/keymanage/',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
      bucketHintKey: 'bucketHint.dns',
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
      bucketHintKey: 'bucketHint.dnsUnderscores',
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
  // --- Mainland China vendors -------------------------------------------
  {
    value: 'baidu_bos',
    label: 'Baidu Cloud BOS',
    shortLabel: 'BOS',
    icon: 'BOS',
    color: '#2932E1',
    bgClass: 'bg-blue-50 dark:bg-blue-950/30',
    textClass: 'text-blue-700 dark:text-blue-300',
    endpointHint: 'https://s3.bj.bcebos.com',
    regionHint: 'bj',
    regions: [
      { value: 'bj', label: '华北-北京 bj' },
      { value: 'gz', label: '华南-广州 gz' },
      { value: 'su', label: '华东-苏州 su' },
      { value: 'hkg', label: '中国香港 hkg' },
      { value: 'fwh', label: '华中-武汉 fwh' },
      { value: 'fsh', label: '华东-上海 fsh' },
      { value: 'sin', label: '新加坡 sin' },
    ],
    credentialHint: 'env:BCE_ACCESS_KEY_ID:BCE_SECRET_ACCESS_KEY or plain:access_key:secret_key',
    credentialLabel: 'AK/SK',
    credentialFields: {
      accessKeyLabel: 'Access Key ID (AK)',
      secretKeyLabel: 'Secret Access Key (SK)',
      accessKeyPlaceholder: 'ALTAKxxxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'BCE_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'BCE_SECRET_ACCESS_KEY',
      consoleUrl: 'https://console.bce.baidu.com/iam/#/iam/accesslist',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
      bucketHintKey: 'bucketHint.dns',
    },
    sseModes: ['AES256'],
    storageClasses: ['STANDARD', 'STANDARD_IA', 'COLD', 'ARCHIVE'],
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
    value: 'kingsoft_ks3',
    label: 'Kingsoft Cloud KS3',
    shortLabel: 'KS3',
    icon: 'KS3',
    color: '#1E6FFF',
    bgClass: 'bg-blue-50 dark:bg-blue-950/30',
    textClass: 'text-blue-700 dark:text-blue-300',
    endpointHint: 'https://ks3-cn-beijing.ksyuncs.com',
    regionHint: 'BEIJING',
    regions: [
      { value: 'BEIJING', label: '华北1（北京）BEIJING' },
      { value: 'SHANGHAI', label: '华东1（上海）SHANGHAI' },
      { value: 'GUANGZHOU', label: '华南1（广州）GUANGZHOU' },
      { value: 'HANGZHOU', label: '华东2（杭州）HANGZHOU' },
      { value: 'Tianjin', label: '华北2（天津）Tianjin' },
      { value: 'HONGKONG', label: '中国香港 HONGKONG' },
      { value: 'SINGAPORE', label: '新加坡 SINGAPORE' },
      { value: 'RUSSIA', label: '俄罗斯 RUSSIA' },
    ],
    credentialHint: 'env:KS3_ACCESS_KEY_ID:KS3_SECRET_ACCESS_KEY or plain:access_key:secret_key',
    credentialLabel: 'AK/SK',
    credentialFields: {
      accessKeyLabel: 'Access Key ID',
      secretKeyLabel: 'Secret Access Key',
      accessKeyPlaceholder: 'AKLTxxxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'KS3_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'KS3_SECRET_ACCESS_KEY',
      consoleUrl: 'https://console.ksyun.com/iam/accesskey',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
      bucketHintKey: 'bucketHint.dns',
    },
    sseModes: ['AES256'],
    storageClasses: ['STANDARD', 'STANDARD_IA', 'ARCHIVE'],
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
    value: 'qiniu_kodo',
    label: 'Qiniu Kodo',
    shortLabel: 'Kodo',
    icon: 'KODO',
    color: '#00AAE7',
    bgClass: 'bg-cyan-50 dark:bg-cyan-950/30',
    textClass: 'text-cyan-700 dark:text-cyan-300',
    endpointHint: 'https://s3-cn-east-1.qiniucs.com',
    regionHint: 'cn-east-1',
    regions: [
      { value: 'cn-east-1', label: '华东-浙江 cn-east-1' },
      { value: 'cn-east-2', label: '华东-浙江2 cn-east-2' },
      { value: 'cn-north-1', label: '华北-河北 cn-north-1' },
      { value: 'cn-south-1', label: '华南-广东 cn-south-1' },
      { value: 'cn-southwest-1', label: '西南-贵州 cn-southwest-1' },
      { value: 'cn-northwest-1', label: '西北-陕西 cn-northwest-1' },
      { value: 'ap-southeast-1', label: '东南亚-新加坡 ap-southeast-1' },
      { value: 'us-north-1', label: '北美-美国 us-north-1' },
    ],
    credentialHint: 'env:QINIU_ACCESS_KEY:QINIU_SECRET_KEY or plain:access_key:secret_key',
    credentialLabel: 'AK/SK',
    credentialFields: {
      accessKeyLabel: 'AccessKey',
      secretKeyLabel: 'SecretKey',
      accessKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'QINIU_ACCESS_KEY',
      defaultEnvSecretKey: 'QINIU_SECRET_KEY',
      consoleUrl: 'https://portal.qiniu.com/user/key',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
      bucketHintKey: 'bucketHint.dns',
    },
    sseModes: ['AES256'],
    storageClasses: ['STANDARD', 'LINE', 'ARCHIVE', 'ARCHIVE_IA'],
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
    value: 'china_mobile_ecloud',
    label: 'China Mobile Ecloud',
    shortLabel: 'Ecloud',
    icon: 'EOS',
    color: '#0086D1',
    bgClass: 'bg-sky-50 dark:bg-sky-950/30',
    textClass: 'text-sky-700 dark:text-sky-300',
    endpointHint: 'https://eos-wuxi-1.cmecloud.cn',
    regionHint: 'wuxi-1',
    regions: [
      { value: 'wuxi-1', label: '华东-无锡 wuxi-1' },
      { value: 'wuxi-2', label: '华东-无锡2 wuxi-2' },
      { value: 'langfang-1', label: '华北-廊坊 langfang-1' },
      { value: 'huanan-1', label: '华南-广州 huanan-1' },
      { value: 'chengdu-1', label: '西南-成都 chengdu-1' },
      { value: 'beijing-1', label: '华北-北京 beijing-1' },
    ],
    credentialHint: 'env:ECLOUD_ACCESS_KEY:ECLOUD_SECRET_KEY or plain:access_key:secret_key',
    credentialLabel: 'AK/SK',
    credentialFields: {
      accessKeyLabel: 'Access Key (AK)',
      secretKeyLabel: 'Secret Key (SK)',
      accessKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'ECLOUD_ACCESS_KEY',
      defaultEnvSecretKey: 'ECLOUD_SECRET_KEY',
      consoleUrl: 'https://ecloud.10086.cn/',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
      bucketHintKey: 'bucketHint.dns',
    },
    sseModes: ['AES256'],
    storageClasses: ['STANDARD', 'STANDARD_IA', 'ARCHIVE'],
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
    value: 'china_telecom_eos',
    label: 'China Telecom EOS',
    shortLabel: 'EOS',
    icon: 'CTEOS',
    color: '#0068B7',
    bgClass: 'bg-indigo-50 dark:bg-indigo-950/30',
    textClass: 'text-indigo-700 dark:text-indigo-300',
    endpointHint: 'https://ooscn-shanghai.ctyunapi.cn',
    regionHint: 'shanghai',
    regions: [
      { value: 'shanghai', label: '华东-上海 shanghai' },
      { value: 'hangzhou', label: '华东-杭州 hangzhou' },
      { value: 'beijing', label: '华北-北京 beijing' },
      { value: 'guangzhou', label: '华南-广州 guangzhou' },
      { value: 'chengdu', label: '西南-成都 chengdu' },
      { value: 'wuhan', label: '华中-武汉 wuhan' },
    ],
    credentialHint: 'env:CTYUN_ACCESS_KEY:CTYUN_SECRET_KEY or plain:access_key:secret_key',
    credentialLabel: 'AK/SK',
    credentialFields: {
      accessKeyLabel: 'Access Key (AK)',
      secretKeyLabel: 'Secret Key (SK)',
      accessKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'CTYUN_ACCESS_KEY',
      defaultEnvSecretKey: 'CTYUN_SECRET_KEY',
      consoleUrl: 'https://www.ctyun.cn/console/user/aksk',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
      bucketHintKey: 'bucketHint.dns',
    },
    sseModes: ['AES256'],
    storageClasses: ['STANDARD', 'STANDARD_IA', 'ARCHIVE'],
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
    value: 'china_unicom_wo',
    label: 'China Unicom Wo Cloud',
    shortLabel: 'Wo Cloud',
    icon: 'WO',
    color: '#E60012',
    bgClass: 'bg-red-50 dark:bg-red-950/30',
    textClass: 'text-red-700 dark:text-red-300',
    endpointHint: 'https://oss-cn-hangzhou.wocloud.com',
    regionHint: 'hangzhou',
    regions: [
      { value: 'hangzhou', label: '华东-杭州 hangzhou' },
      { value: 'beijing', label: '华北-北京 beijing' },
      { value: 'shanghai', label: '华东-上海 shanghai' },
      { value: 'guangzhou', label: '华南-广州 guangzhou' },
      { value: 'shenzhen', label: '华南-深圳 shenzhen' },
    ],
    credentialHint: 'env:WO_ACCESS_KEY:WO_SECRET_KEY or plain:access_key:secret_key',
    credentialLabel: 'AK/SK',
    credentialFields: {
      accessKeyLabel: 'Access Key (AK)',
      secretKeyLabel: 'Secret Key (SK)',
      accessKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'WO_ACCESS_KEY',
      defaultEnvSecretKey: 'WO_SECRET_KEY',
      consoleUrl: 'https://www.wocloud.com.cn/',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
      bucketHintKey: 'bucketHint.dns',
    },
    sseModes: ['AES256'],
    storageClasses: ['STANDARD', 'STANDARD_IA', 'ARCHIVE'],
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
    value: 'minio',
    label: 'MinIO',
    shortLabel: 'MinIO',
    icon: 'MINIO',
    color: '#C72C48',
    bgClass: 'bg-rose-50 dark:bg-rose-950/30',
    textClass: 'text-rose-700 dark:text-rose-300',
    endpointHint: 'https://minio.example.com:9000',
    regionHint: 'us-east-1',
    regions: [],
    credentialHint: 'env:MINIO_ROOT_USER:MINIO_ROOT_PASSWORD or plain:access_key:secret_key',
    credentialLabel: 'Access Key',
    credentialFields: {
      accessKeyLabel: 'Access Key',
      secretKeyLabel: 'Secret Key',
      accessKeyPlaceholder: 'minioadmin',
      secretKeyPlaceholder: 'minioadmin',
      defaultEnvAccessKey: 'MINIO_ROOT_USER',
      defaultEnvSecretKey: 'MINIO_ROOT_PASSWORD',
      consoleUrl: 'https://min.io/docs/minio/linux/administration/identity-access-management/minio-user-management.html',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
      bucketHintKey: 'bucketHint.dns',
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
  },
  {
    value: 'cloudflare_r2',
    label: 'Cloudflare R2',
    shortLabel: 'R2',
    icon: 'R2',
    color: '#F38020',
    bgClass: 'bg-orange-50 dark:bg-orange-950/30',
    textClass: 'text-orange-700 dark:text-orange-300',
    endpointHint: 'https://<account-id>.r2.cloudflarestorage.com',
    regionHint: 'auto',
    // R2 accepts only the literal `auto`, and its host carries the Cloudflare
    // account id, so there is no region selector to offer and no endpoint to
    // derive. The operator supplies the endpoint; the region field stays `auto`.
    regions: [],
    credentialHint: 'env:R2_ACCESS_KEY_ID:R2_SECRET_ACCESS_KEY or plain:access_key:secret_key',
    credentialLabel: 'R2 API Token',
    credentialFields: {
      accessKeyLabel: 'Access Key ID',
      secretKeyLabel: 'Secret Access Key',
      accessKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'R2_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'R2_SECRET_ACCESS_KEY',
      consoleUrl: 'https://dash.cloudflare.com/?to=/:account/r2/api-tokens',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
      bucketHintKey: 'bucketHint.dns',
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
  },
  {
    value: 'backblaze_b2',
    label: 'Backblaze B2',
    shortLabel: 'B2',
    icon: 'B2',
    color: '#E21E29',
    bgClass: 'bg-red-50 dark:bg-red-950/30',
    textClass: 'text-red-700 dark:text-red-300',
    endpointHint: 'https://s3.us-west-004.backblazeb2.com',
    regionHint: 'us-west-004',
    regions: [
      { value: 'us-west-000', label: 'US West (California) us-west-000' },
      { value: 'us-west-001', label: 'US West (Arizona) us-west-001' },
      { value: 'us-west-002', label: 'US West (Nevada) us-west-002' },
      { value: 'us-west-004', label: 'US West (California) us-west-004' },
      { value: 'us-east-005', label: 'US East (N. Virginia) us-east-005' },
      { value: 'eu-central-003', label: 'EU Central (Amsterdam) eu-central-003' },
      { value: 'ca-central-001', label: 'Canada (Toronto) ca-central-001' },
      { value: 'ap-northeast-001', label: 'Asia Pacific (Sydney) ap-northeast-001' },
    ],
    credentialHint: 'env:B2_APPLICATION_KEY_ID:B2_APPLICATION_KEY or plain:key_id:application_key',
    credentialLabel: 'Application Key',
    credentialFields: {
      accessKeyLabel: 'Application Key ID',
      secretKeyLabel: 'Application Key',
      accessKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'K00xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'B2_APPLICATION_KEY_ID',
      defaultEnvSecretKey: 'B2_APPLICATION_KEY',
      consoleUrl: 'https://secure.backblaze.com/app_keys.htm',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
      bucketHintKey: 'bucketHint.dns',
    },
    sseModes: ['AES256'],
    storageClasses: ['STANDARD'],
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
    value: 'wasabi',
    label: 'Wasabi',
    shortLabel: 'Wasabi',
    icon: 'WASABI',
    color: '#01CD3E',
    bgClass: 'bg-emerald-50 dark:bg-emerald-950/30',
    textClass: 'text-emerald-700 dark:text-emerald-300',
    endpointHint: 'https://s3.us-east-1.wasabisys.com',
    regionHint: 'us-east-1',
    regions: [
      { value: 'us-east-1', label: 'US East 1 (Virginia) us-east-1' },
      { value: 'us-east-2', label: 'US East 2 (Ohio) us-east-2' },
      { value: 'us-central-1', label: 'US Central 1 (Texas) us-central-1' },
      { value: 'us-west-1', label: 'US West 1 (California) us-west-1' },
      { value: 'ca-central-1', label: 'Canada Central 1 ca-central-1' },
      { value: 'eu-central-1', label: 'EU Central 1 (Amsterdam) eu-central-1' },
      { value: 'eu-central-2', label: 'EU Central 2 (Frankfurt) eu-central-2' },
      { value: 'eu-west-1', label: 'EU West 1 (London) eu-west-1' },
      { value: 'eu-west-2', label: 'EU West 2 (Paris) eu-west-2' },
      { value: 'ap-northeast-1', label: 'AP Northeast 1 (Tokyo) ap-northeast-1' },
      { value: 'ap-northeast-2', label: 'AP Northeast 2 (Osaka) ap-northeast-2' },
      { value: 'ap-southeast-1', label: 'AP Southeast 1 (Singapore) ap-southeast-1' },
      { value: 'ap-southeast-2', label: 'AP Southeast 2 (Sydney) ap-southeast-2' },
    ],
    credentialHint: 'env:WASABI_ACCESS_KEY:WASABI_SECRET_KEY or plain:access_key:secret_key',
    credentialLabel: 'Access Key',
    credentialFields: {
      accessKeyLabel: 'Access Key',
      secretKeyLabel: 'Secret Key',
      accessKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'WASABI_ACCESS_KEY',
      defaultEnvSecretKey: 'WASABI_SECRET_KEY',
      consoleUrl: 'https://console.wasabisys.com/',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
      bucketHintKey: 'bucketHint.dns',
    },
    sseModes: ['AES256'],
    storageClasses: ['STANDARD'],
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
    value: 'digitalocean_spaces',
    label: 'DigitalOcean Spaces',
    shortLabel: 'Spaces',
    icon: 'DO',
    color: '#0080FF',
    bgClass: 'bg-blue-50 dark:bg-blue-950/30',
    textClass: 'text-blue-700 dark:text-blue-300',
    endpointHint: 'https://nyc3.digitaloceanspaces.com',
    regionHint: 'nyc3',
    regions: [
      { value: 'nyc3', label: 'New York 3 nyc3' },
      { value: 'sfo2', label: 'San Francisco 2 sfo2' },
      { value: 'sfo3', label: 'San Francisco 3 sfo3' },
      { value: 'ams3', label: 'Amsterdam 3 ams3' },
      { value: 'sgp1', label: 'Singapore 1 sgp1' },
      { value: 'blr1', label: 'Bangalore 1 blr1' },
      { value: 'fra1', label: 'Frankfurt 1 fra1' },
      { value: 'syd1', label: 'Sydney 1 syd1' },
    ],
    credentialHint: 'env:SPACES_ACCESS_KEY_ID:SPACES_SECRET_ACCESS_KEY or plain:access_key:secret_key',
    credentialLabel: 'Spaces Key',
    credentialFields: {
      accessKeyLabel: 'Access Key',
      secretKeyLabel: 'Secret Key',
      accessKeyPlaceholder: 'DO00xxxxxxxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'SPACES_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'SPACES_SECRET_ACCESS_KEY',
      consoleUrl: 'https://cloud.digitalocean.com/account/api/tokens',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
      bucketHintKey: 'bucketHint.dns',
    },
    sseModes: ['AES256'],
    storageClasses: ['STANDARD'],
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
    value: 'linode_object_storage',
    label: 'Akamai / Linode Object Storage',
    shortLabel: 'Linode',
    icon: 'LINODE',
    color: '#00A95C',
    bgClass: 'bg-emerald-50 dark:bg-emerald-950/30',
    textClass: 'text-emerald-700 dark:text-emerald-300',
    endpointHint: 'https://us-east-1.linodeobjects.com',
    regionHint: 'us-east-1',
    regions: [
      { value: 'us-east-1', label: 'Newark, NJ us-east-1' },
      { value: 'us-southeast-1', label: 'Atlanta, GA us-southeast-1' },
      { value: 'us-ord', label: 'Chicago, IL us-ord' },
      { value: 'us-sea', label: 'Seattle, WA us-sea' },
      { value: 'us-lax', label: 'Los Angeles, CA us-lax' },
      { value: 'us-mia', label: 'Miami, FL us-mia' },
      { value: 'eu-central-1', label: 'Frankfurt, DE eu-central-1' },
      { value: 'eu-west-1', label: 'London, UK eu-west-1' },
      { value: 'nl-ams-1', label: 'Amsterdam, NL nl-ams-1' },
      { value: 'fr-par-1', label: 'Paris, FR fr-par-1' },
      { value: 'gb-lon-1', label: 'London, UK gb-lon-1' },
      { value: 'in-maa-1', label: 'Chennai, IN in-maa-1' },
      { value: 'jp-osa-1', label: 'Osaka, JP jp-osa-1' },
      { value: 'au-mel-1', label: 'Melbourne, AU au-mel-1' },
      { value: 'sg-sin-1', label: 'Singapore, SG sg-sin-1' },
      { value: 'br-gru-1', label: 'São Paulo, BR br-gru-1' },
    ],
    credentialHint: 'env:LINODE_ACCESS_KEY:LINODE_SECRET_KEY or plain:access_key:secret_key',
    credentialLabel: 'Access Key',
    credentialFields: {
      accessKeyLabel: 'Access Key',
      secretKeyLabel: 'Secret Key',
      accessKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'LINODE_ACCESS_KEY',
      defaultEnvSecretKey: 'LINODE_SECRET_KEY',
      consoleUrl: 'https://cloud.linode.com/object-storage/access-keys',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
      bucketHintKey: 'bucketHint.dns',
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
  },
  {
    value: 'vultr_object_storage',
    label: 'Vultr Object Storage',
    shortLabel: 'Vultr',
    icon: 'VULTR',
    color: '#007BFC',
    bgClass: 'bg-blue-50 dark:bg-blue-950/30',
    textClass: 'text-blue-700 dark:text-blue-300',
    endpointHint: 'https://ewr1.vultrobjects.com',
    regionHint: 'ewr1',
    regions: [
      { value: 'ewr1', label: 'New Jersey ewr1' },
      { value: 'del1', label: 'Delhi del1' },
      { value: 'blr1', label: 'Bangalore blr1' },
      { value: 'sjc1', label: 'Silicon Valley sjc1' },
      { value: 'atl1', label: 'Atlanta atl1' },
      { value: 'hnl1', label: 'Honolulu hnl1' },
      { value: 'lax1', label: 'Los Angeles lax1' },
      { value: 'ord1', label: 'Chicago ord1' },
      { value: 'sea1', label: 'Seattle sea1' },
      { value: 'ams1', label: 'Amsterdam ams1' },
      { value: 'cdg1', label: 'Paris cdg1' },
      { value: 'fra1', label: 'Frankfurt fra1' },
      { value: 'lhr1', label: 'London lhr1' },
      { value: 'mad1', label: 'Madrid mad1' },
      { value: 'waw1', label: 'Warsaw waw1' },
      { value: 'syd1', label: 'Sydney syd1' },
      { value: 'nrt1', label: 'Tokyo nrt1' },
      { value: 'icn1', label: 'Seoul icn1' },
      { value: 'sgp1', label: 'Singapore sgp1' },
      { value: 'bom1', label: 'Mumbai bom1' },
      { value: 'jnb1', label: 'Johannesburg jnb1' },
      { value: 'sao1', label: 'São Paulo sao1' },
    ],
    credentialHint: 'env:VULTR_ACCESS_KEY:VULTR_SECRET_KEY or plain:access_key:secret_key',
    credentialLabel: 'Object Storage Key',
    credentialFields: {
      accessKeyLabel: 'Access Key',
      secretKeyLabel: 'Secret Key',
      accessKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'VULTR_ACCESS_KEY',
      defaultEnvSecretKey: 'VULTR_SECRET_KEY',
      consoleUrl: 'https://my.vultr.com/objectstorage/',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
      bucketHintKey: 'bucketHint.dns',
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
  },
  {
    value: 'scaleway_object_storage',
    label: 'Scaleway Object Storage',
    shortLabel: 'Scaleway',
    icon: 'SCW',
    color: '#5100FF',
    bgClass: 'bg-violet-50 dark:bg-violet-950/30',
    textClass: 'text-violet-700 dark:text-violet-300',
    endpointHint: 'https://s3.fr-par.scw.cloud',
    regionHint: 'fr-par',
    regions: [
      { value: 'fr-par', label: 'Paris fr-par' },
      { value: 'nl-ams', label: 'Amsterdam nl-ams' },
      { value: 'pl-waw', label: 'Warsaw pl-waw' },
    ],
    credentialHint: 'env:SCW_ACCESS_KEY:SCW_SECRET_KEY or plain:access_key:secret_key',
    credentialLabel: 'API Key',
    credentialFields: {
      accessKeyLabel: 'Access Key',
      secretKeyLabel: 'Secret Key',
      accessKeyPlaceholder: 'SCWxxxxxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx',
      defaultEnvAccessKey: 'SCW_ACCESS_KEY',
      defaultEnvSecretKey: 'SCW_SECRET_KEY',
      consoleUrl: 'https://console.scaleway.com/iam/api-keys',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
      bucketHintKey: 'bucketHint.dns',
    },
    sseModes: ['AES256'],
    storageClasses: ['STANDARD', 'ONEZONE_IA', 'GLACIER'],
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
    value: 'oracle_cloud_storage',
    label: 'Oracle Cloud Infrastructure Object Storage',
    shortLabel: 'OCI',
    icon: 'OCI',
    color: '#C74634',
    bgClass: 'bg-red-50 dark:bg-red-950/30',
    textClass: 'text-red-700 dark:text-red-300',
    endpointHint: 'https://<namespace>.compat.objectstorage.us-ashburn-1.oraclecloud.com',
    regionHint: 'us-ashburn-1',
    regions: [],
    // The host embeds an identifier the region cannot supply (tenancy namespace), so the
    // endpoint is operator-supplied rather than region-derived.

    credentialHint: 'env:OCI_ACCESS_KEY:OCI_SECRET_KEY or plain:access_key:secret_key',
    credentialLabel: 'Customer Secret Key',
    credentialFields: {
      accessKeyLabel: 'Access Key',
      secretKeyLabel: 'Secret Key',
      accessKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'OCI_ACCESS_KEY',
      defaultEnvSecretKey: 'OCI_SECRET_KEY',
      consoleUrl: 'https://cloud.oracle.com/identity/domains/my-profile/api-keys',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
      bucketHintKey: 'bucketHint.dns',
    },
    sseModes: ['AES256'],
    storageClasses: ['Standard', 'InfrequentAccess', 'Archive'],
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
    value: 'ibm_cos',
    label: 'IBM Cloud Object Storage',
    shortLabel: 'IBM COS',
    icon: 'IBM',
    color: '#0F62FE',
    bgClass: 'bg-blue-50 dark:bg-blue-950/30',
    textClass: 'text-blue-700 dark:text-blue-300',
    endpointHint: 'https://s3.us-east.cloud-object-storage.appdomain.cloud',
    regionHint: 'us-east',
    regions: [
      { value: 'us-east', label: 'US East (Virginia) us-east' },
      { value: 'us-south', label: 'US South (Dallas) us-south' },
      { value: 'ca-tor', label: 'Canada (Toronto) ca-tor' },
      { value: 'eu-gb', label: 'UK (London) eu-gb' },
      { value: 'eu-de', label: 'Germany (Frankfurt) eu-de' },
      { value: 'eu-es', label: 'Spain (Madrid) eu-es' },
      { value: 'jp-tok', label: 'Japan (Tokyo) jp-tok' },
      { value: 'jp-osa', label: 'Japan (Osaka) jp-osa' },
      { value: 'au-syd', label: 'Australia (Sydney) au-syd' },
      { value: 'br-sao', label: 'Brazil (São Paulo) br-sao' },
      { value: 'in-che', label: 'India (Chennai) in-che' },
    ],
    credentialHint: 'env:IBM_COS_ACCESS_KEY_ID:IBM_COS_SECRET_ACCESS_KEY or plain:access_key:secret_key',
    credentialLabel: 'HMAC Key',
    credentialFields: {
      accessKeyLabel: 'Access Key ID',
      secretKeyLabel: 'Secret Access Key',
      accessKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'IBM_COS_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'IBM_COS_SECRET_ACCESS_KEY',
      consoleUrl: 'https://cloud.ibm.com/resources',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
      bucketHintKey: 'bucketHint.dns',
    },
    sseModes: ['AES256'],
    storageClasses: ['STANDARD', 'VAULT', 'COLD', 'SMART'],
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
    value: 'alibaba_cloud_international',
    label: 'Alibaba Cloud OSS (International)',
    shortLabel: 'OSS Intl',
    icon: 'OSS',
    color: '#FF6A00',
    bgClass: 'bg-orange-50 dark:bg-orange-950/30',
    textClass: 'text-orange-700 dark:text-orange-300',
    endpointHint: 'https://oss-ap-southeast-1.aliyuncs.com',
    regionHint: 'ap-southeast-1',
    regions: [
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
    credentialHint: 'env:ALIBABA_CLOUD_ACCESS_KEY_ID:ALIBABA_CLOUD_ACCESS_KEY_SECRET or plain:access_key:secret_key',
    credentialLabel: 'AccessKey',
    credentialFields: {
      accessKeyLabel: 'AccessKey ID',
      secretKeyLabel: 'AccessKey Secret',
      accessKeyPlaceholder: 'LTAIxxxxxxxxxxxxxx',
      secretKeyPlaceholder: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      defaultEnvAccessKey: 'ALIBABA_CLOUD_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'ALIBABA_CLOUD_ACCESS_KEY_SECRET',
      consoleUrl: 'https://ram.console.aliyun.com/manage/ak',
      bucketHint: '3-63 chars, lowercase letters, numbers, hyphens',
      bucketHintKey: 'bucketHint.dns',
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
    value: 'tencent_cloud_international',
    label: 'Tencent Cloud COS (International)',
    shortLabel: 'COS Intl',
    icon: 'COS',
    color: '#006EFF',
    bgClass: 'bg-blue-50 dark:bg-blue-950/30',
    textClass: 'text-blue-700 dark:text-blue-300',
    endpointHint: 'https://cos.ap-singapore.myqcloud.com',
    regionHint: 'ap-singapore',
    regions: [
      { value: 'ap-singapore', label: '新加坡 ap-singapore' },
      { value: 'ap-mumbai', label: '孟买 ap-mumbai' },
      { value: 'ap-jakarta', label: '雅加达 ap-jakarta' },
      { value: 'ap-seoul', label: '首尔 ap-seoul' },
      { value: 'ap-tokyo', label: '东京 ap-tokyo' },
      { value: 'ap-bangkok', label: '曼谷 ap-bangkok' },
      { value: 'na-siliconvalley', label: '硅谷 na-siliconvalley' },
      { value: 'na-ashburn', label: '弗吉尼亚 na-ashburn' },
      { value: 'sa-saopaulo', label: '圣保罗 sa-saopaulo' },
      { value: 'eu-frankfurt', label: '法兰克福 eu-frankfurt' },
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
      bucketHintKey: 'bucketHint.appId',
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

/**
 * Provider id prefix every bootstrapped configuration shares.
 *
 * Mirrors `BUILTIN_PROVIDER_ID_PREFIX` in the server's
 * `provider_account_defaults.rs`, where the id is derived from the kind rather
 * than generated, so a re-run recognizes its own row. That same property is
 * what makes the prefix the honest way to recognize a built-in row here: the
 * display name cannot, because an operator may rename a built-in row and a
 * hand-made row may carry any name at all.
 */
export const BUILTIN_PROVIDER_ID_PREFIX = 'builtin-storage-provider-';

/**
 * Display name the server's bootstrap writes for each catalogued kind.
 *
 * This is the *canonical English* name as stored in the row
 * (`BuiltinCloudProvider::provider_name`), not a per-locale label: it is what
 * a row still carrying its bootstrap name compares equal to, which is how
 * {@link providerDisplayName} tells an untouched built-in row from one an
 * operator renamed. The operator-facing per-locale copy lives in the
 * `builtInProviderName.<kind>` dictionary entries.
 *
 * Keep it in step with the server table; `providerKindCatalog.test.ts` mirrors
 * the Rust side and fails when the two drift.
 */
export const BUILTIN_PROVIDER_DISPLAY_NAMES: Readonly<Record<string, string>> = {
  local_filesystem: 'Built-in Local Filesystem',
  s3_compatible: 'Built-in Amazon S3',
  aliyun_oss: 'Built-in Alibaba Cloud OSS',
  tencent_cos: 'Built-in Tencent Cloud COS',
  huawei_obs: 'Built-in Huawei Cloud OBS',
  volcengine_tos: 'Built-in Volcengine TOS',
  google_cloud_storage: 'Built-in Google Cloud Storage',
  baidu_bos: 'Built-in Baidu Cloud BOS',
  kingsoft_ks3: 'Built-in Kingsoft Cloud KS3',
  qiniu_kodo: 'Built-in Qiniu Kodo',
  china_mobile_ecloud: 'Built-in China Mobile Ecloud',
  china_telecom_eos: 'Built-in China Telecom EOS',
  china_unicom_wo: 'Built-in China Unicom Wo Cloud',
  minio: 'Built-in MinIO',
  cloudflare_r2: 'Built-in Cloudflare R2',
  backblaze_b2: 'Built-in Backblaze B2',
  wasabi: 'Built-in Wasabi',
  digitalocean_spaces: 'Built-in DigitalOcean Spaces',
  linode_object_storage: 'Built-in Akamai / Linode Object Storage',
  vultr_object_storage: 'Built-in Vultr Object Storage',
  scaleway_object_storage: 'Built-in Scaleway Object Storage',
  oracle_cloud_storage: 'Built-in Oracle Cloud Object Storage',
  ibm_cos: 'Built-in IBM Cloud Object Storage',
  alibaba_cloud_international: 'Built-in Alibaba Cloud OSS (International)',
  tencent_cloud_international: 'Built-in Tencent Cloud COS (International)',
};

const REGION_ENDPOINT_BUILDERS: Record<string, (region: string) => string> = {
  s3_compatible: (region) =>
    region === 'us-east-1' ? 'https://s3.amazonaws.com' : `https://s3.${region}.amazonaws.com`,
  aliyun_oss: (region) => `https://oss-${region}.aliyuncs.com`,
  tencent_cos: (region) => `https://cos.${region}.myqcloud.com`,
  huawei_obs: (region) => `https://obs.${region}.myhuaweicloud.com`,
  volcengine_tos: (region) => `https://tos-${region}.volces.com`,
  // Google Cloud Storage exposes one global S3-interoperable endpoint for every
  // region, so the region never participates in the host name. Without this
  // entry the region-driven auto-fill produced an empty endpoint for a kind the
  // editor offers in its picker.
  google_cloud_storage: () => 'https://storage.googleapis.com',
  // --- Mainland China ---------------------------------------------------
  baidu_bos: (region) => `https://s3.${region}.bcebos.com`,
  kingsoft_ks3: (region) => `https://ks3-${region.toLowerCase()}.ksyuncs.com`,
  qiniu_kodo: (region) => `https://s3-${region}.qiniucs.com`,
  china_mobile_ecloud: (region) => `https://eos-${region}.cmecloud.cn`,
  china_telecom_eos: (region) => `https://ooscn-${region}.ctyunapi.cn`,
  china_unicom_wo: (region) => `https://oss-cn-${region}.wocloud.com`,
  alibaba_cloud_international: (region) => `https://oss-${region}.aliyuncs.com`,
  tencent_cloud_international: (region) => `https://cos.${region}.myqcloud.com`,
  // --- Rest of world ----------------------------------------------------
  // Deliberately absent, because their endpoint host embeds an identifier the
  // region alone cannot supply, so a region-driven auto-fill could only emit a
  // placeholder that fails at request time. The editor leaves the field to the
  // operator instead:
  //   minio                     self-hosted, host is deployment-specific
  //   cloudflare_r2             host embeds the Cloudflare account id
  //   oracle_cloud_storage      host embeds the tenancy namespace
  backblaze_b2: (region) => `https://s3.${region}.backblazeb2.com`,
  wasabi: (region) => `https://s3.${region}.wasabisys.com`,
  digitalocean_spaces: (region) => `https://${region}.digitaloceanspaces.com`,
  linode_object_storage: (region) => `https://${region}.linodeobjects.com`,
  vultr_object_storage: (region) => `https://${region}.vultrobjects.com`,
  scaleway_object_storage: (region) => `https://s3.${region}.scw.cloud`,
  ibm_cos: (region) => `https://s3.${region}.cloud-object-storage.appdomain.cloud`,
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

/**
 * Default platform account-center vendor for a storage provider kind. The
 * account center accepts any `^[a-z][a-z0-9_]{1,31}$` vendor code, so this is
 * only the pre-selection for the "new account" form, not a restriction.
 */
export function providerVendorCodeForKind(kind: string): string {
  switch (kind) {
    case 's3_compatible':
      return 'aws';
    case 'aliyun_oss':
    case 'alibaba_cloud_international':
      return 'aliyun';
    case 'tencent_cos':
    case 'tencent_cloud_international':
      return 'tencent';
    case 'huawei_obs':
      return 'huawei';
    case 'volcengine_tos':
      return 'volcengine';
    case 'google_cloud_storage':
      return 'google';
    case 'baidu_bos':
      return 'baidu';
    case 'kingsoft_ks3':
      return 'kingsoft';
    case 'qiniu_kodo':
      return 'qiniu';
    case 'china_mobile_ecloud':
      return 'china_mobile';
    case 'china_telecom_eos':
      return 'china_telecom';
    case 'china_unicom_wo':
      return 'china_unicom';
    case 'minio':
      return 'minio';
    case 'cloudflare_r2':
      return 'cloudflare';
    case 'backblaze_b2':
      return 'backblaze';
    case 'wasabi':
      return 'wasabi';
    case 'digitalocean_spaces':
      return 'digitalocean';
    case 'linode_object_storage':
      return 'linode';
    case 'vultr_object_storage':
      return 'vultr';
    case 'scaleway_object_storage':
      return 'scaleway';
    case 'oracle_cloud_storage':
      return 'oracle';
    case 'ibm_cos':
      return 'ibm';
    default:
      return 'custom';
  }
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

/**
 * Translate contract shared with the package `useTranslation` hook: dotted
 * keys under the `storageProviders` namespace with `{param}` interpolation.
 */
export type ProviderMetaTranslate = (
  key: string,
  params?: Record<string, string | number>,
) => string;

/**
 * Resolve a catalog display string through the storage dictionary. The drive
 * dictionary returns the full prefixed key when a lookup misses in every
 * locale, so the registry's own string doubles as the final fallback — the
 * registry data stays locale-neutral reference data while the catalogs own
 * the per-locale copy (`I18N_SPEC.md` §6 catalog ownership).
 */
function translateOr(
  translate: ProviderMetaTranslate,
  key: string,
  fallback: string,
): string {
  const value = translate(key);
  /*
   * Two miss shapes, and both have to fall back: the dictionary provider
   * answers with the fully qualified key, while a component rendered without
   * one — a unit test, or a host that forgot to mount `LanguageProvider` —
   * receives the bare key it passed in. Treating only the first as a miss
   * would paint raw keys into the UI of exactly the host that is already
   * misconfigured.
   */
  return value === `storageProviders.${key}` || value === key ? fallback : value;
}

/**
 * Operator-facing label for a provider kind, localized from the
 * `kindLabel.<value>` dictionary entries with the registry label as
 * fallback. `custom:<name>` kinds keep the operator-defined name suffix.
 */
export function providerKindLabel(
  translate: ProviderMetaTranslate,
  meta: Pick<ProviderKindMeta, 'value' | 'label'>,
): string {
  const value = String(meta.value);
  if (value.startsWith('custom:')) {
    const translated = translate('kindCustomLabel', { name: value.slice('custom:'.length) });
    return translated === 'storageProviders.kindCustomLabel' ? meta.label : translated;
  }
  return translateOr(translate, `kindLabel.${value}`, meta.label);
}

/**
 * Operator-facing label for a region option: the localized region name from
 * `regionLabel.<kind>.<code>` followed by the region code, matching the
 * registry's `"<name> <code>"` display shape.
 */
export function providerRegionLabel(
  translate: ProviderMetaTranslate,
  kind: string,
  region: ProviderRegion,
): string {
  const name = translateOr(translate, `regionLabel.${kind}.${region.value}`, region.label);
  return name === region.label ? region.label : `${name} ${region.value}`;
}

/**
 * Bucket-naming hint for the credential section, localized through the
 * `bucketHint.*` dictionary entries when the registry declares a hint key.
 */
export function providerBucketHint(
  translate: ProviderMetaTranslate,
  fields: ProviderCredentialFieldMeta,
): string | undefined {
  if (fields.bucketHintKey) {
    return translateOr(translate, fields.bucketHintKey, fields.bucketHint ?? '');
  }
  return fields.bucketHint;
}

/**
 * Vendor credential vocabulary label for the manual credential-reference
 * field, localized from `credentialLabel.<value>` with the registry string as
 * fallback; an empty label lets the caller render its generic default.
 */
export function providerCredentialLabel(
  translate: ProviderMetaTranslate,
  meta: Pick<ProviderKindMeta, 'value' | 'credentialLabel'>,
): string {
  return translateOr(translate, `credentialLabel.${String(meta.value)}`, meta.credentialLabel);
}

/**
 * Operator-facing name of a provider configuration.
 *
 * A built-in row that still carries the name the bootstrap wrote is shown in
 * the reader's language (`builtInProviderName.<kind>`), so a Chinese console
 * does not greet its operator with a wall of `Built-in …` rows. Both halves of
 * the test are required and neither is decoration:
 *
 * * the id prefix says the row came from the bootstrap, so a hand-made
 *   configuration that happens to be named like a built-in one is left alone;
 * * the name equality says nobody has renamed it, so an operator's own name —
 *   their data, in whatever language they chose — is never replaced by a
 *   translated default they did not pick.
 *
 * A miss in both catalogs keeps the stored name, which is what a provider
 * whose kind the console does not catalogue yet must show.
 */
export function providerDisplayName(
  translate: ProviderMetaTranslate,
  provider: Pick<StorageProviderView, 'id' | 'providerKind' | 'displayName'>,
): string {
  const kind = String(getProviderKindMeta(provider.providerKind).value);
  const builtInName = BUILTIN_PROVIDER_DISPLAY_NAMES[kind];
  if (
    !builtInName
    || provider.displayName !== builtInName
    || !provider.id.startsWith(BUILTIN_PROVIDER_ID_PREFIX)
  ) {
    return provider.displayName;
  }
  return translateOr(translate, `builtInProviderName.${kind}`, provider.displayName);
}
