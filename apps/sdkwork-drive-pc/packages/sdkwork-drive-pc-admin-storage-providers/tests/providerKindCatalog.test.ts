import { describe, expect, it } from 'vitest';
import {
  buildProviderEndpointUrl,
  getAllProviderKindMeta,
  getProviderKindMeta,
  providerVendorCodeForKind,
} from '../src/utils/providerKindConfig';

/**
 * The backend's `DriveStorageProviderKind::BUILTIN` list, mirrored here on
 * purpose.
 *
 * The console and the server each own a copy of the catalog — the server also
 * whitelists these keys in Postgres — and a key present in one but not the
 * other is a defect that only shows up as a 422 at submit time or as a vendor
 * that cannot be picked at all. Duplicating the list in a test is what turns
 * that drift into a red test instead.
 *
 * When adding a vendor, add it here too; the assertions below then require the
 * console metadata to catch up.
 */
const BACKEND_BUILTIN_PROVIDER_KINDS = [
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
  'tencent_cloud_international',
] as const;

describe('provider kind catalog', () => {
  it('offers every backend built-in kind in the picker', () => {
    const offered = new Set(getAllProviderKindMeta().map((meta) => meta.value));
    for (const kind of BACKEND_BUILTIN_PROVIDER_KINDS) {
      expect(offered, `picker is missing backend kind ${kind}`).toContain(kind);
    }
  });

  it('does not offer a kind the backend would reject', () => {
    // `custom` is the console-side marker for the free-form escape hatch; every
    // other offered kind must exist in the backend whitelist.
    for (const meta of getAllProviderKindMeta()) {
      if (meta.value === 'custom') {
        continue;
      }
      expect(
        BACKEND_BUILTIN_PROVIDER_KINDS as readonly string[],
        `picker offers ${meta.value}, which the backend does not accept`,
      ).toContain(meta.value);
    }
  });

  it('gives every non-local kind a resolvable vendor code', () => {
    for (const meta of getAllProviderKindMeta()) {
      if (meta.value === 'custom' || meta.features.isLocal) {
        continue;
      }
      const vendor = providerVendorCodeForKind(meta.value);
      expect(vendor, `${meta.value} has no vendor code`).toMatch(/^[a-z][a-z0-9_]{1,31}$/);
      expect(vendor, `${meta.value} fell back to the generic vendor code`).not.toBe('custom');
    }
  });

  it('keeps mainland and rest-of-world vendors in one catalog', () => {
    // Guards the user-visible requirement: both markets are fully covered, so a
    // tenant in either completes the picker without resorting to `custom:`.
    const values = getAllProviderKindMeta().map((meta) => meta.value);
    const mainland = [
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
    ];
    const restOfWorld = [
      's3_compatible',
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
    ];
    for (const kind of [...mainland, ...restOfWorld]) {
      expect(values, `${kind} is not offered`).toContain(kind);
    }
  });
});

describe('provider endpoint templates for the added vendors', () => {
  it('derives mainland endpoints from the region', () => {
    expect(buildProviderEndpointUrl('baidu_bos', 'bj')).toBe('https://s3.bj.bcebos.com');
    expect(buildProviderEndpointUrl('qiniu_kodo', 'cn-east-1')).toBe(
      'https://s3-cn-east-1.qiniucs.com',
    );
    expect(buildProviderEndpointUrl('china_mobile_ecloud', 'wuxi-1')).toBe(
      'https://eos-wuxi-1.cmecloud.cn',
    );
    expect(buildProviderEndpointUrl('china_telecom_eos', 'shanghai')).toBe(
      'https://ooscn-shanghai.ctyunapi.cn',
    );
    expect(buildProviderEndpointUrl('china_unicom_wo', 'hangzhou')).toBe(
      'https://oss-cn-hangzhou.wocloud.com',
    );
    expect(buildProviderEndpointUrl('kingsoft_ks3', 'BEIJING')).toBe(
      'https://ks3-beijing.ksyuncs.com',
    );
    expect(buildProviderEndpointUrl('alibaba_cloud_international', 'ap-southeast-1')).toBe(
      'https://oss-ap-southeast-1.aliyuncs.com',
    );
    expect(buildProviderEndpointUrl('tencent_cloud_international', 'ap-singapore')).toBe(
      'https://cos.ap-singapore.myqcloud.com',
    );
  });

  it('derives rest-of-world endpoints from the region', () => {
    expect(buildProviderEndpointUrl('backblaze_b2', 'us-west-004')).toBe(
      'https://s3.us-west-004.backblazeb2.com',
    );
    expect(buildProviderEndpointUrl('wasabi', 'us-east-1')).toBe(
      'https://s3.us-east-1.wasabisys.com',
    );
    expect(buildProviderEndpointUrl('digitalocean_spaces', 'nyc3')).toBe(
      'https://nyc3.digitaloceanspaces.com',
    );
    expect(buildProviderEndpointUrl('linode_object_storage', 'us-east-1')).toBe(
      'https://us-east-1.linodeobjects.com',
    );
    expect(buildProviderEndpointUrl('vultr_object_storage', 'ewr1')).toBe(
      'https://ewr1.vultrobjects.com',
    );
    expect(buildProviderEndpointUrl('scaleway_object_storage', 'fr-par')).toBe(
      'https://s3.fr-par.scw.cloud',
    );
    expect(buildProviderEndpointUrl('ibm_cos', 'us-east')).toBe(
      'https://s3.us-east.cloud-object-storage.appdomain.cloud',
    );
  });

  it('leaves identifier-embedding hosts to the operator', () => {
    // These hosts carry an account id / namespace / deployment host that the
    // region cannot supply, so the editor must not fabricate a placeholder.
    expect(buildProviderEndpointUrl('minio', 'us-east-1')).toBeUndefined();
    expect(buildProviderEndpointUrl('cloudflare_r2', 'auto')).toBeUndefined();
    expect(buildProviderEndpointUrl('oracle_cloud_storage', 'us-ashburn-1')).toBeUndefined();
  });
});

describe('provider kind metadata for the added vendors', () => {
  it('labels each vendor and keeps its hint aligned with a real region', () => {
    for (const kind of BACKEND_BUILTIN_PROVIDER_KINDS) {
      const meta = getProviderKindMeta(kind);
      expect(meta.label, `${kind} has no label`).not.toBe('');
      expect(meta.shortLabel, `${kind} has no short label`).not.toBe('');
      if (meta.value === 'custom' || meta.features.isLocal) {
        continue;
      }
      // A region-driven kind must hint at a region it actually offers, or the
      // auto-fill picks a value the select can never show as selected.
      if (meta.regions.length > 0) {
        expect(
          meta.regions.map((region) => region.value),
          `${kind} region hint is not in its own region list`,
        ).toContain(meta.regionHint);
      }
    }
  });

  it('describes credential fields for every S3 vendor', () => {
    for (const meta of getAllProviderKindMeta()) {
      if (meta.value === 'custom' || meta.features.isLocal) {
        continue;
      }
      expect(meta.credentialFields, `${meta.value} exposes no credential fields`).toBeDefined();
      expect(meta.credentialFields?.defaultEnvAccessKey).toMatch(/^[A-Z][A-Z0-9_]*$/);
      expect(meta.credentialFields?.defaultEnvSecretKey).toMatch(/^[A-Z][A-Z0-9_]*$/);
      expect(meta.sseModes.length, `${meta.value} offers no SSE mode`).toBeGreaterThan(0);
      expect(meta.storageClasses.length, `${meta.value} offers no storage class`).toBeGreaterThan(0);
    }
  });

  /**
   * The console labels and env-name defaults are mirrored a second time in the
   * server's bootstrap table (`provider_account_defaults.rs`), which writes the
   * placeholder key pair and returns its own `credentialFields` copy with the
   * `initializeProviderAccountDefaults` response.
   *
   * Two copies that disagree is a defect an operator only meets at the worst
   * moment: the editor pre-fills `AWS_ACCESS_KEY_ID` while the bootstrapped
   * account centre was minted under a different name, so the key the operator
   * pasted looks like it belongs to a vendor it does not. The server end of the
   * pair is the one that actually ships the placeholder, so the console end is
   * the one that has to track it — and this table is what makes a lapse red.
   */
  const BACKEND_VENDOR_CREDENTIAL_FIELDS: Record<
    string,
    { accessKeyLabel: string; secretKeyLabel: string; defaultEnvAccessKey: string; defaultEnvSecretKey: string }
  > = {
    s3_compatible: {
      accessKeyLabel: 'Access Key ID',
      secretKeyLabel: 'Secret Access Key',
      defaultEnvAccessKey: 'AWS_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'AWS_SECRET_ACCESS_KEY',
    },
    aliyun_oss: {
      accessKeyLabel: 'AccessKey ID',
      secretKeyLabel: 'AccessKey Secret',
      defaultEnvAccessKey: 'ALIBABA_CLOUD_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'ALIBABA_CLOUD_ACCESS_KEY_SECRET',
    },
    tencent_cos: {
      accessKeyLabel: 'SecretId',
      secretKeyLabel: 'SecretKey',
      defaultEnvAccessKey: 'COS_SECRET_ID',
      defaultEnvSecretKey: 'COS_SECRET_KEY',
    },
    huawei_obs: {
      accessKeyLabel: 'Access Key ID (AK)',
      secretKeyLabel: 'Secret Access Key (SK)',
      defaultEnvAccessKey: 'HUAWEI_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'HUAWEI_SECRET_ACCESS_KEY',
    },
    volcengine_tos: {
      accessKeyLabel: 'Access Key ID (AK)',
      secretKeyLabel: 'Secret Access Key (SK)',
      defaultEnvAccessKey: 'VOLC_ACCESSKEY',
      defaultEnvSecretKey: 'VOLC_SECRETKEY',
    },
    google_cloud_storage: {
      accessKeyLabel: 'HMAC Access ID',
      secretKeyLabel: 'HMAC Secret',
      defaultEnvAccessKey: 'GOOGLE_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'GOOGLE_SECRET_ACCESS_KEY',
    },
    baidu_bos: {
      accessKeyLabel: 'Access Key ID (AK)',
      secretKeyLabel: 'Secret Access Key (SK)',
      defaultEnvAccessKey: 'BCE_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'BCE_SECRET_ACCESS_KEY',
    },
    kingsoft_ks3: {
      accessKeyLabel: 'Access Key ID',
      secretKeyLabel: 'Secret Access Key',
      defaultEnvAccessKey: 'KS3_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'KS3_SECRET_ACCESS_KEY',
    },
    qiniu_kodo: {
      accessKeyLabel: 'AccessKey',
      secretKeyLabel: 'SecretKey',
      defaultEnvAccessKey: 'QINIU_ACCESS_KEY',
      defaultEnvSecretKey: 'QINIU_SECRET_KEY',
    },
    china_mobile_ecloud: {
      accessKeyLabel: 'Access Key (AK)',
      secretKeyLabel: 'Secret Key (SK)',
      defaultEnvAccessKey: 'ECLOUD_ACCESS_KEY',
      defaultEnvSecretKey: 'ECLOUD_SECRET_KEY',
    },
    china_telecom_eos: {
      accessKeyLabel: 'Access Key (AK)',
      secretKeyLabel: 'Secret Key (SK)',
      defaultEnvAccessKey: 'CTYUN_ACCESS_KEY',
      defaultEnvSecretKey: 'CTYUN_SECRET_KEY',
    },
    china_unicom_wo: {
      accessKeyLabel: 'Access Key (AK)',
      secretKeyLabel: 'Secret Key (SK)',
      defaultEnvAccessKey: 'WO_ACCESS_KEY',
      defaultEnvSecretKey: 'WO_SECRET_KEY',
    },
    minio: {
      accessKeyLabel: 'Access Key',
      secretKeyLabel: 'Secret Key',
      defaultEnvAccessKey: 'MINIO_ROOT_USER',
      defaultEnvSecretKey: 'MINIO_ROOT_PASSWORD',
    },
    cloudflare_r2: {
      accessKeyLabel: 'Access Key ID',
      secretKeyLabel: 'Secret Access Key',
      defaultEnvAccessKey: 'R2_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'R2_SECRET_ACCESS_KEY',
    },
    backblaze_b2: {
      accessKeyLabel: 'Application Key ID',
      secretKeyLabel: 'Application Key',
      defaultEnvAccessKey: 'B2_APPLICATION_KEY_ID',
      defaultEnvSecretKey: 'B2_APPLICATION_KEY',
    },
    wasabi: {
      accessKeyLabel: 'Access Key',
      secretKeyLabel: 'Secret Key',
      defaultEnvAccessKey: 'WASABI_ACCESS_KEY',
      defaultEnvSecretKey: 'WASABI_SECRET_KEY',
    },
    digitalocean_spaces: {
      accessKeyLabel: 'Access Key',
      secretKeyLabel: 'Secret Key',
      defaultEnvAccessKey: 'SPACES_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'SPACES_SECRET_ACCESS_KEY',
    },
    linode_object_storage: {
      accessKeyLabel: 'Access Key',
      secretKeyLabel: 'Secret Key',
      defaultEnvAccessKey: 'LINODE_ACCESS_KEY',
      defaultEnvSecretKey: 'LINODE_SECRET_KEY',
    },
    vultr_object_storage: {
      accessKeyLabel: 'Access Key',
      secretKeyLabel: 'Secret Key',
      defaultEnvAccessKey: 'VULTR_ACCESS_KEY',
      defaultEnvSecretKey: 'VULTR_SECRET_KEY',
    },
    scaleway_object_storage: {
      accessKeyLabel: 'Access Key',
      secretKeyLabel: 'Secret Key',
      defaultEnvAccessKey: 'SCW_ACCESS_KEY',
      defaultEnvSecretKey: 'SCW_SECRET_KEY',
    },
    oracle_cloud_storage: {
      accessKeyLabel: 'Access Key',
      secretKeyLabel: 'Secret Key',
      defaultEnvAccessKey: 'OCI_ACCESS_KEY',
      defaultEnvSecretKey: 'OCI_SECRET_KEY',
    },
    ibm_cos: {
      accessKeyLabel: 'Access Key ID',
      secretKeyLabel: 'Secret Access Key',
      defaultEnvAccessKey: 'IBM_COS_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'IBM_COS_SECRET_ACCESS_KEY',
    },
    alibaba_cloud_international: {
      accessKeyLabel: 'AccessKey ID',
      secretKeyLabel: 'AccessKey Secret',
      defaultEnvAccessKey: 'ALIBABA_CLOUD_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'ALIBABA_CLOUD_ACCESS_KEY_SECRET',
    },
    tencent_cloud_international: {
      accessKeyLabel: 'SecretId',
      secretKeyLabel: 'SecretKey',
      defaultEnvAccessKey: 'COS_SECRET_ID',
      defaultEnvSecretKey: 'COS_SECRET_KEY',
    },
  };

  it('keeps every vendor credential label and env default in step with the server', () => {
    // The server bootstraps one account per cloud kind; the console must not
    // describe fewer, or a picked vendor would have no account behind it.
    const cloudKinds = BACKEND_BUILTIN_PROVIDER_KINDS.filter(
      (kind) => kind !== 'local_filesystem',
    );
    expect(
      Object.keys(BACKEND_VENDOR_CREDENTIAL_FIELDS).sort(),
      'the mirrored credential table covers a different vendor set than the backend catalog',
    ).toEqual([...cloudKinds].sort());

    for (const kind of cloudKinds) {
      const meta = getProviderKindMeta(kind);
      const expected = BACKEND_VENDOR_CREDENTIAL_FIELDS[kind];
      expect(meta.credentialFields, `${kind} exposes no credential fields`).toBeDefined();
      expect(
        meta.credentialFields?.accessKeyLabel,
        `${kind} public-key label drifted from the server bootstrap`,
      ).toBe(expected.accessKeyLabel);
      expect(
        meta.credentialFields?.secretKeyLabel,
        `${kind} secret-key label drifted from the server bootstrap`,
      ).toBe(expected.secretKeyLabel);
      expect(
        meta.credentialFields?.defaultEnvAccessKey,
        `${kind} access-key env default drifted from the server bootstrap`,
      ).toBe(expected.defaultEnvAccessKey);
      expect(
        meta.credentialFields?.defaultEnvSecretKey,
        `${kind} secret-key env default drifted from the server bootstrap`,
      ).toBe(expected.defaultEnvSecretKey);
    }
  });

  it('keeps the mainland and international siblings on the same vendor vocabulary', () => {
    // Alibaba and Tencent each ship twice (mainland + international). The same
    // vendor names its key pair the same way in both consoles, so a sibling
    // pair that disagreed would be a transcription slip, not a product choice.
    const siblings: Array<[string, string]> = [
      ['aliyun_oss', 'alibaba_cloud_international'],
      ['tencent_cos', 'tencent_cloud_international'],
    ];
    for (const [mainland, international] of siblings) {
      const a = getProviderKindMeta(mainland).credentialFields;
      const b = getProviderKindMeta(international).credentialFields;
      expect(b?.accessKeyLabel, `${international} disagrees with ${mainland}`).toBe(a?.accessKeyLabel);
      expect(b?.secretKeyLabel, `${international} disagrees with ${mainland}`).toBe(a?.secretKeyLabel);
    }
  });
});

/**
 * The encryption modes and storage classes each vendor accepts, mirrored from
 * the Rust contract's `supported_sse_modes()` / `supported_storage_classes()`
 * (`sdkwork-drive-storage-contract/src/types.rs`).
 *
 * The console still carries its own copy for the pre-initialization fallback,
 * and that copy is exactly the kind of second table that drifts. The tested
 * failure mode is concrete: an operator opens the editor for a vendor whose
 * capabilities were never fetched, and the dropdown offers `aws:kms` to a
 * vendor that only accepts `KMS` — the request then fails at the store with an
 * error the console cannot explain. Keeping the mirror here turns that into a
 * red test at authoring time.
 *
 * The order matters too, not just membership: `[0]` is what the backend
 * bootstrap writes into a freshly created provider, so a console list that led
 * with a different value would disagree with the row the operator is editing.
 */
const BACKEND_VENDOR_CAPABILITY_DEFAULTS: Record<
  string,
  { serverSideEncryptionModes: string[]; storageClasses: string[] }
> = {
  s3_compatible: {
    serverSideEncryptionModes: ['AES256', 'aws:kms', 'aws:kms:dsse'],
    storageClasses: ['STANDARD', 'STANDARD_IA', 'ONEZONE_IA', 'INTELLIGENT_TIERING', 'GLACIER', 'DEEP_ARCHIVE'],
  },
  aliyun_oss: {
    serverSideEncryptionModes: ['KMS', 'AES256'],
    storageClasses: ['Standard', 'IA', 'Archive', 'ColdArchive', 'DeepColdArchive'],
  },
  tencent_cos: {
    serverSideEncryptionModes: ['AES256', 'KMS'],
    storageClasses: ['STANDARD', 'STANDARD_IA', 'ARCHIVE', 'DEEP_ARCHIVE'],
  },
  huawei_obs: {
    serverSideEncryptionModes: ['kms', 'AES256'],
    storageClasses: ['STANDARD', 'WARM', 'COLD'],
  },
  volcengine_tos: {
    serverSideEncryptionModes: ['AES256', 'KMS'],
    storageClasses: ['STANDARD', 'IA', 'ARCHIVE', 'DEEP_ARCHIVE'],
  },
  google_cloud_storage: {
    serverSideEncryptionModes: ['AES256', 'GOOGLE_DEFAULT_ENCRYPTION'],
    storageClasses: ['STANDARD', 'NEARLINE', 'COLDLINE', 'ARCHIVE'],
  },
  baidu_bos: {
    serverSideEncryptionModes: ['AES256'],
    storageClasses: ['STANDARD', 'STANDARD_IA', 'COLD', 'ARCHIVE'],
  },
  kingsoft_ks3: {
    serverSideEncryptionModes: ['AES256'],
    storageClasses: ['STANDARD', 'STANDARD_IA', 'ARCHIVE'],
  },
  qiniu_kodo: {
    serverSideEncryptionModes: ['AES256'],
    storageClasses: ['STANDARD', 'LINE', 'ARCHIVE', 'ARCHIVE_IA'],
  },
  china_mobile_ecloud: {
    serverSideEncryptionModes: ['AES256'],
    storageClasses: ['STANDARD', 'STANDARD_IA', 'ARCHIVE'],
  },
  china_telecom_eos: {
    serverSideEncryptionModes: ['AES256'],
    storageClasses: ['STANDARD', 'STANDARD_IA', 'ARCHIVE'],
  },
  china_unicom_wo: {
    serverSideEncryptionModes: ['AES256'],
    storageClasses: ['STANDARD', 'STANDARD_IA', 'ARCHIVE'],
  },
  minio: {
    serverSideEncryptionModes: ['AES256'],
    storageClasses: ['STANDARD'],
  },
  cloudflare_r2: {
    serverSideEncryptionModes: ['AES256'],
    storageClasses: ['STANDARD'],
  },
  backblaze_b2: {
    serverSideEncryptionModes: ['AES256'],
    storageClasses: ['STANDARD'],
  },
  wasabi: {
    serverSideEncryptionModes: ['AES256'],
    storageClasses: ['STANDARD'],
  },
  digitalocean_spaces: {
    serverSideEncryptionModes: ['AES256'],
    storageClasses: ['STANDARD'],
  },
  linode_object_storage: {
    serverSideEncryptionModes: ['AES256'],
    storageClasses: ['STANDARD'],
  },
  vultr_object_storage: {
    serverSideEncryptionModes: ['AES256'],
    storageClasses: ['STANDARD'],
  },
  scaleway_object_storage: {
    serverSideEncryptionModes: ['AES256'],
    storageClasses: ['STANDARD', 'ONEZONE_IA', 'GLACIER'],
  },
  oracle_cloud_storage: {
    serverSideEncryptionModes: ['AES256'],
    storageClasses: ['Standard', 'InfrequentAccess', 'Archive'],
  },
  ibm_cos: {
    serverSideEncryptionModes: ['AES256'],
    storageClasses: ['STANDARD', 'VAULT', 'COLD', 'SMART'],
  },
  alibaba_cloud_international: {
    serverSideEncryptionModes: ['KMS', 'AES256'],
    storageClasses: ['Standard', 'IA', 'Archive', 'ColdArchive', 'DeepColdArchive'],
  },
  tencent_cloud_international: {
    serverSideEncryptionModes: ['AES256', 'KMS'],
    storageClasses: ['STANDARD', 'STANDARD_IA', 'ARCHIVE', 'DEEP_ARCHIVE'],
  },
};

describe('provider advanced-control vocabulary', () => {
  it('mirrors the contract vendor table for every cloud kind', () => {
    const cloudKinds = BACKEND_BUILTIN_PROVIDER_KINDS.filter(
      (kind) => kind !== 'local_filesystem',
    );
    expect(
      Object.keys(BACKEND_VENDOR_CAPABILITY_DEFAULTS).sort(),
      'the mirrored capability table covers a different vendor set than the backend catalog',
    ).toEqual([...cloudKinds].sort());

    for (const kind of cloudKinds) {
      const meta = getProviderKindMeta(kind);
      const expected = BACKEND_VENDOR_CAPABILITY_DEFAULTS[kind];
      // Order, not just membership: the backend writes `[0]`, so a console list
      // that led with a different value would disagree with the created row.
      expect(meta.sseModes, `${kind} SSE modes drifted from the contract table`).toEqual(
        expected.serverSideEncryptionModes,
      );
      expect(
        meta.storageClasses,
        `${kind} storage classes drifted from the contract table`,
      ).toEqual(expected.storageClasses);
    }
  });

  it('never offers one vendor a token that belongs to another', () => {
    // The concrete defect the shared-generic-list bug produced: Tencent's and
    // Aliyun's editors were handed AWS's `aws:kms`.
    for (const kind of ['tencent_cos', 'aliyun_oss', 'huawei_obs', 'volcengine_tos']) {
      expect(
        getProviderKindMeta(kind).sseModes,
        `${kind} must not be offered AWS's aws:kms token`,
      ).not.toContain('aws:kms');
    }
    // And the reverse: AWS is the one vendor that does accept it.
    expect(getProviderKindMeta('s3_compatible').sseModes).toContain('aws:kms');
  });

  it('gives the credential-free kind no advanced controls', () => {
    const meta = getProviderKindMeta('local_filesystem');
    expect(meta.features.hasSse).toBe(false);
    expect(meta.features.hasStorageClass).toBe(false);
    expect(meta.sseModes).toEqual([]);
    expect(meta.storageClasses).toEqual([]);
  });
});
