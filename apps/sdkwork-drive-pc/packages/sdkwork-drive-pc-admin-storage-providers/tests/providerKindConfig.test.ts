import { describe, expect, it } from 'vitest';
import {
  buildProviderEndpointUrl,
  providerKindDefaultPathStyle,
} from '../src/utils/providerKindConfig';

describe('provider kind defaults', () => {
  it('builds mainland and overseas Kingsoft KS3 endpoints', () => {
    expect(buildProviderEndpointUrl('kingsoft_ks3', 'BEIJING')).toBe(
      'https://ks3-cn-beijing.ksyuncs.com',
    );
    expect(buildProviderEndpointUrl('kingsoft_ks3', 'HANGZHOU')).toBe(
      'https://ks3-cn-hangzhou.ksyuncs.com',
    );
    expect(buildProviderEndpointUrl('kingsoft_ks3', 'SINGAPORE')).toBe(
      'https://ks3-sgp.ksyuncs.com',
    );
  });

  it('mirrors the server path-style defaults', () => {
    expect(providerKindDefaultPathStyle('minio')).toBe(true);
    expect(providerKindDefaultPathStyle('s3_compatible')).toBe(true);
    expect(providerKindDefaultPathStyle('custom:my-gateway')).toBe(true);
    expect(providerKindDefaultPathStyle('tencent_cos')).toBe(false);
  });
});
