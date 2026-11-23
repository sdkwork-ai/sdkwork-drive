import { describe, expect, it } from 'vitest';
import {
  buildProviderEndpointUrl,
  BUILTIN_PROVIDER_DISPLAY_NAMES,
  getAllProviderKindMeta,
  getProviderKindMeta,
  providerEndpointIsDerivable,
  providerEndpointMatchesRegion,
  providerRegionInputMode,
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
      'https://ks3-cn-beijing.ksyuncs.com',
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

describe('provider endpoint/region cascade taxonomy', () => {
  /**
   * The editor's whole cascade rests on two claims per kind, and both are
   * properties of the catalog rather than of one vendor:
   *
   * 1. a kind that publishes a region list can derive its endpoint from a
   *    region, so picking a region is enough to finish the endpoint field;
   * 2. the kind's own `endpointHint` (the placeholder and the value a new form
   *    opens with) *is* what its `regionHint` derives, so the form never opens
   *    in a state its own region contradicts.
   *
   * Asserting them over the catalog is what stops a newly added vendor from
   * being offered with a region selector that silently does nothing.
   */
  it('derives the endpoint hint from the region hint for every region-driven kind', () => {
    for (const meta of getAllProviderKindMeta()) {
      if (meta.features.isLocal || !meta.features.hasRegion) {
        continue;
      }
      const derived = buildProviderEndpointUrl(String(meta.value), meta.regionHint);
      if (meta.regions.length > 0) {
        expect(
          derived,
          `${meta.value} offers ${meta.regions.length} regions but cannot derive an endpoint from one`,
        ).toBeDefined();
        expect(
          meta.endpointHint,
          `${meta.value} endpoint hint disagrees with the endpoint its region hint derives`,
        ).toBe(derived);
      }
    }
  });

  it('only derives endpoints for hosts the region alone can address', () => {
    for (const meta of getAllProviderKindMeta()) {
      const derivable = providerEndpointIsDerivable(String(meta.value));
      if (meta.regions.length > 0) {
        expect(derivable, `${meta.value} lists regions but derives no endpoint`).toBe(true);
      }
      if (derivable) {
        for (const region of meta.regions.length > 0 ? meta.regions : [{ value: meta.regionHint, label: '' }]) {
          const derived = buildProviderEndpointUrl(String(meta.value), region.value);
          // A derived value is submitted as the real endpoint, so it must never
          // be a placeholder the operator is expected to edit by hand.
          expect(derived, `${meta.value}/${region.value} derived no endpoint`).toBeDefined();
          expect(derived, `${meta.value}/${region.value} derived a placeholder host`).not.toContain('<');
          expect(derived, `${meta.value}/${region.value} derived a non-TLS endpoint`).toMatch(/^https:\/\//);
          expect(
            providerEndpointMatchesRegion(String(meta.value), region.value, derived as string),
            `${meta.value}/${region.value} does not recognize its own derived endpoint`,
          ).toBe(true);
        }
      }
    }
  });

  it('leaves the region field usable for every non-local kind', () => {
    const modes = new Map<string, string>();
    for (const meta of getAllProviderKindMeta()) {
      modes.set(String(meta.value), providerRegionInputMode(meta));
    }
    // Hidden only where there is no region at all.
    expect(modes.get('local_filesystem')).toBe('hidden');
    // Selector where the vendor publishes a closed list.
    expect(modes.get('tencent_cos')).toBe('select');
    expect(modes.get('aliyun_oss')).toBe('select');
    // Free text where the region is real but the catalog does not enumerate it,
    // so a custom gateway is not stuck on one hard-coded region code.
    expect(modes.get('custom')).toBe('text');
    expect(modes.get('minio')).toBe('text');
    expect(modes.get('oracle_cloud_storage')).toBe('text');
    // A protocol constant (`auto`) is shown but not editable.
    expect(modes.get('cloudflare_r2')).toBe('fixed');
    for (const [kind, mode] of modes) {
      if (kind === 'local_filesystem') {
        continue;
      }
      expect(mode, `${kind} renders no region control at all`).not.toBe('hidden');
    }
  });

  it('compares stored endpoints against the region without failing on cosmetic differences', () => {
    // The same host is stored with a trailing slash, upper case, or a stray
    // space by different clients; treating those as a manual override would
    // make the editor warn about rows it would itself have written.
    expect(
      providerEndpointMatchesRegion('tencent_cos', 'ap-beijing', 'https://cos.ap-beijing.myqcloud.com/'),
    ).toBe(true);
    expect(
      providerEndpointMatchesRegion('tencent_cos', 'ap-beijing', '  https://COS.AP-BEIJING.MYQCLOUD.COM  '),
    ).toBe(true);
    // A real disagreement is not normalized away.
    expect(
      providerEndpointMatchesRegion('tencent_cos', 'ap-beijing', 'https://cos.ap-guangzhou.myqcloud.com'),
    ).toBe(false);
    // A kind with no derivation never "matches" — there is nothing to match.
    expect(providerEndpointMatchesRegion('oracle_cloud_storage', 'us-ashburn-1', 'https://x.oraclecloud.com')).toBe(false);
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

/**
 * The configuration names the server's bootstrap writes, mirrored from
 * `BUILTIN_CLOUD_PROVIDERS` / `LOCAL_PROVIDER_NAME` in
 * `provider_account_defaults.rs`.
 *
 * The console keeps its own copy because it is what tells "this row still
 * carries the name the bootstrap gave it" (and may therefore be shown
 * localized) from "an operator named this row" (and must be shown verbatim).
 * A copy that drifts does not fail loudly — it silently stops translating rows,
 * or worse, starts translating a name an operator typed. Mirroring the server
 * table here turns that into a red test.
 */
const BACKEND_BUILTIN_PROVIDER_NAMES: Record<string, string> = {
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

describe('built-in provider configuration names', () => {
  it('covers exactly the backend built-in kinds, local kind included', () => {
    expect(Object.keys(BUILTIN_PROVIDER_DISPLAY_NAMES).sort()).toEqual(
      [...BACKEND_BUILTIN_PROVIDER_KINDS].sort(),
    );
  });

  it('mirrors the name the server bootstrap writes for every kind', () => {
    expect(Object.keys(BACKEND_BUILTIN_PROVIDER_NAMES).sort()).toEqual(
      Object.keys(BUILTIN_PROVIDER_DISPLAY_NAMES).sort(),
    );
    for (const [kind, name] of Object.entries(BACKEND_BUILTIN_PROVIDER_NAMES)) {
      expect(
        BUILTIN_PROVIDER_DISPLAY_NAMES[kind],
        `${kind} built-in name drifted from the server bootstrap`,
      ).toBe(name);
    }
  });
});

/**
 * Endpoint and region the server's bootstrap writes per built-in kind,
 * mirrored from `BUILTIN_CLOUD_PROVIDERS` in `provider_account_defaults.rs`.
 *
 * The console and the server each own a copy of the vendor defaults, and the
 * console's copy is what a new form opens with. When they disagree the operator
 * sees one endpoint in the editor and another in the saved row — the alias-vs-
 * regional AWS host is exactly how that happened — so a divergence here is a
 * defect, and mirroring the server table is what makes it red.
 *
 * The three identifier-embedding vendors (`minio`, `cloudflare_r2`,
 * `oracle_cloud_storage`) intentionally differ: the server stores a placeholder
 * host for its bootstrap row, while the console deliberately leaves the field to
 * the operator and shows the placeholder only as a hint. Only their region is
 * comparable.
 */
const BACKEND_BUILTIN_PROVIDER_DEFAULTS: Record<string, { endpointUrl: string; region: string }> = {
  s3_compatible: { endpointUrl: 'https://s3.us-east-1.amazonaws.com', region: 'us-east-1' },
  aliyun_oss: { endpointUrl: 'https://oss-cn-hangzhou.aliyuncs.com', region: 'cn-hangzhou' },
  tencent_cos: { endpointUrl: 'https://cos.ap-guangzhou.myqcloud.com', region: 'ap-guangzhou' },
  huawei_obs: { endpointUrl: 'https://obs.cn-north-1.myhuaweicloud.com', region: 'cn-north-1' },
  volcengine_tos: { endpointUrl: 'https://tos-cn-beijing.volces.com', region: 'cn-beijing' },
  google_cloud_storage: { endpointUrl: 'https://storage.googleapis.com', region: 'us-central1' },
  baidu_bos: { endpointUrl: 'https://s3.bj.bcebos.com', region: 'bj' },
  kingsoft_ks3: { endpointUrl: 'https://ks3-cn-beijing.ksyuncs.com', region: 'BEIJING' },
  qiniu_kodo: { endpointUrl: 'https://s3-cn-east-1.qiniucs.com', region: 'cn-east-1' },
  china_mobile_ecloud: { endpointUrl: 'https://eos-wuxi-1.cmecloud.cn', region: 'wuxi-1' },
  china_telecom_eos: { endpointUrl: 'https://ooscn-shanghai.ctyunapi.cn', region: 'shanghai' },
  china_unicom_wo: { endpointUrl: 'https://oss-cn-hangzhou.wocloud.com', region: 'hangzhou' },
  minio: { endpointUrl: 'https://minio.internal.sdkwork.local', region: 'us-east-1' },
  cloudflare_r2: { endpointUrl: 'https://placeholder.r2.cloudflarestorage.com', region: 'auto' },
  backblaze_b2: { endpointUrl: 'https://s3.us-west-004.backblazeb2.com', region: 'us-west-004' },
  wasabi: { endpointUrl: 'https://s3.us-east-1.wasabisys.com', region: 'us-east-1' },
  digitalocean_spaces: { endpointUrl: 'https://nyc3.digitaloceanspaces.com', region: 'nyc3' },
  linode_object_storage: { endpointUrl: 'https://us-east-1.linodeobjects.com', region: 'us-east-1' },
  vultr_object_storage: { endpointUrl: 'https://ewr1.vultrobjects.com', region: 'ewr1' },
  scaleway_object_storage: { endpointUrl: 'https://s3.fr-par.scw.cloud', region: 'fr-par' },
  oracle_cloud_storage: {
    endpointUrl: 'https://placeholder.oraclecloud.com',
    region: 'us-ashburn-1',
  },
  ibm_cos: {
    endpointUrl: 'https://s3.us-east.cloud-object-storage.appdomain.cloud',
    region: 'us-east',
  },
  alibaba_cloud_international: {
    endpointUrl: 'https://oss-ap-southeast-1.aliyuncs.com',
    region: 'ap-southeast-1',
  },
  tencent_cloud_international: {
    endpointUrl: 'https://cos.ap-singapore.myqcloud.com',
    region: 'ap-singapore',
  },
};

describe('built-in provider bootstrap endpoints and regions', () => {
  it('covers exactly the cloud kinds the server bootstraps', () => {
    expect(Object.keys(BACKEND_BUILTIN_PROVIDER_DEFAULTS).sort()).toEqual(
      BACKEND_BUILTIN_PROVIDER_KINDS.filter((kind) => kind !== 'local_filesystem').sort(),
    );
  });

  it('opens a new form on the endpoint and region the server would write', () => {
    for (const [kind, expected] of Object.entries(BACKEND_BUILTIN_PROVIDER_DEFAULTS)) {
      const meta = getProviderKindMeta(kind);
      expect(meta.regionHint, `${kind} region hint drifted from the server bootstrap`).toBe(
        expected.region,
      );
      // A kind the console derives must open on the very endpoint the server
      // writes; only the identifier-embedding vendors are exempt.
      if (providerEndpointIsDerivable(kind)) {
        expect(meta.endpointHint, `${kind} endpoint hint drifted from the server bootstrap`).toBe(
          expected.endpointUrl,
        );
      }
    }
  });
});
