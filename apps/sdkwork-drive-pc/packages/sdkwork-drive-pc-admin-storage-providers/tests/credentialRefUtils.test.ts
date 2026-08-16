import { describe, expect, it } from 'vitest';
import {
  buildCredentialRef,
  isCredentialRefMasked,
  parseCredentialRef,
} from '../src/utils/credentialRefUtils';
import { buildProviderEndpointUrl } from '../src/utils/providerKindConfig';

describe('credentialRefUtils', () => {
  it('builds and parses plain credential refs', () => {
    const ref = buildCredentialRef('direct', {
      accessKey: 'AKID123',
      secretKey: 'secret456',
    });
    expect(ref).toBe('plain:AKID123:secret456');
    expect(parseCredentialRef(ref!)).toEqual({
      mode: 'direct',
      direct: { accessKey: 'AKID123', secretKey: 'secret456', sessionToken: undefined },
    });
  });

  it('builds and parses env credential refs with defaults for COS/TOS', () => {
    const cosRef = buildCredentialRef('env', {
      accessKeyEnv: 'COS_SECRET_ID',
      secretKeyEnv: 'COS_SECRET_KEY',
    });
    expect(cosRef).toBe('env:COS_SECRET_ID:COS_SECRET_KEY');

    const tosRef = buildCredentialRef('env', {
      accessKeyEnv: 'TOS_ACCESS_KEY_ID',
      secretKeyEnv: 'TOS_SECRET_ACCESS_KEY',
    });
    expect(tosRef).toBe('env:TOS_ACCESS_KEY_ID:TOS_SECRET_ACCESS_KEY');
  });

  it('detects masked credential refs from admin API responses', () => {
    expect(isCredentialRefMasked('plain:***')).toBe(true);
    expect(isCredentialRefMasked('env:***')).toBe(true);
    expect(isCredentialRefMasked('secret:***')).toBe(true);
    expect(isCredentialRefMasked('plain:AKID:secret')).toBe(false);
  });
});

describe('buildProviderEndpointUrl', () => {
  it('builds Tencent COS endpoints from region', () => {
    expect(buildProviderEndpointUrl('tencent_cos', 'ap-guangzhou')).toBe(
      'https://cos.ap-guangzhou.myqcloud.com',
    );
    expect(buildProviderEndpointUrl('tencent_cos', 'ap-shanghai')).toBe(
      'https://cos.ap-shanghai.myqcloud.com',
    );
  });

  it('builds Volcengine TOS endpoints from region', () => {
    expect(buildProviderEndpointUrl('volcengine_tos', 'cn-beijing')).toBe(
      'https://tos-cn-beijing.volces.com',
    );
    expect(buildProviderEndpointUrl('volcengine_tos', 'cn-shanghai')).toBe(
      'https://tos-cn-shanghai.volces.com',
    );
  });

  it('builds AWS S3 endpoints from region', () => {
    expect(buildProviderEndpointUrl('s3_compatible', 'us-east-1')).toBe('https://s3.amazonaws.com');
    expect(buildProviderEndpointUrl('s3_compatible', 'ap-southeast-1')).toBe(
      'https://s3.ap-southeast-1.amazonaws.com',
    );
  });

  it('builds Aliyun OSS endpoints from region', () => {
    expect(buildProviderEndpointUrl('aliyun_oss', 'cn-hangzhou')).toBe(
      'https://oss-cn-hangzhou.aliyuncs.com',
    );
  });

  it('builds Huawei OBS endpoints from region', () => {
    expect(buildProviderEndpointUrl('huawei_obs', 'cn-north-1')).toBe(
      'https://obs.cn-north-1.myhuaweicloud.com',
    );
  });
});
