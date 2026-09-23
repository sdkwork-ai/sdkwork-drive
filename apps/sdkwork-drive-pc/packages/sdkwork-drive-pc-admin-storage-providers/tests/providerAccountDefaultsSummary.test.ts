import { describe, expect, it } from 'vitest';
import type { StorageProviderAccountDefaultView } from '../src/types/storageProviderAdminTypes';
import { summarizeProviderAccountDefaults } from '../src/utils/providerAccountDefaultsSummary';

function row(
  overrides: Partial<StorageProviderAccountDefaultView> = {},
): StorageProviderAccountDefaultView {
  return {
    providerKind: 's3_compatible',
    providerId: 'builtin-storage-provider-s3-compatible',
    providerCreated: false,
    accountCreated: false,
    credentialSeeded: false,
    ...overrides,
  };
}

describe('summarizeProviderAccountDefaults', () => {
  it('counts each flag independently on a first run', () => {
    const summary = summarizeProviderAccountDefaults([
      // local_filesystem 无凭证：只有服务商配置，没有账号，也就没有占位密钥。
      row({ providerKind: 'local_filesystem', providerCreated: true }),
      row({
        providerKind: 'aliyun_oss',
        vendorCode: 'aliyun',
        providerAccountId: 'iampacct-018f-aliyun',
        accountCode: 'builtin-aliyun-storage',
        providerCreated: true,
        accountCreated: true,
        credentialSeeded: true,
      }),
      // 服务商已存在、账号是新建的（运维删过账号中心那一行）。
      row({
        providerKind: 'tencent_cos',
        vendorCode: 'tencent',
        providerAccountId: 'iampacct-018f-tencent',
        accountCode: 'builtin-tencent-storage',
        accountCreated: true,
        credentialSeeded: true,
      }),
    ]);

    expect(summary).toEqual({ total: 3, providers: 2, accounts: 2, credentials: 2 });
  });

  it('reports all-zero counts on a rerun that finds everything already in place', () => {
    // 这是整条 UI 提示语的立足点：三个计数全零 = 本次没有覆盖任何已存在的条目，
    // 尤其是没有动运维已经填过的真实密钥。若这里算错，页面就会谎报"又新建了一批"。
    const summary = summarizeProviderAccountDefaults([
      row({ providerKind: 'local_filesystem' }),
      row({
        providerKind: 'aliyun_oss',
        vendorCode: 'aliyun',
        providerAccountId: 'iampacct-018f-aliyun',
        accountCode: 'builtin-aliyun-storage',
      }),
    ]);

    expect(summary).toEqual({ total: 2, providers: 0, accounts: 0, credentials: 0 });
  });

  it('returns zeroed counts for an empty response instead of throwing', () => {
    expect(summarizeProviderAccountDefaults([])).toEqual({
      total: 0,
      providers: 0,
      accounts: 0,
      credentials: 0,
    });
  });

  it('does not count a provider whose account row came back without flags', () => {
    // 契约里 `credentialSeeded` 缺失 = false。缺字段必须是"没写"，不是"可能写了"：
    // 宁可少报一条新建，也不能让运维以为占位密钥已经铺好。
    const summary = summarizeProviderAccountDefaults([
      {
        providerKind: 'huawei_obs',
        providerId: 'builtin-storage-provider-huawei-obs',
        providerCreated: true,
      } as StorageProviderAccountDefaultView,
    ]);

    expect(summary).toEqual({ total: 1, providers: 1, accounts: 0, credentials: 0 });
  });
});
