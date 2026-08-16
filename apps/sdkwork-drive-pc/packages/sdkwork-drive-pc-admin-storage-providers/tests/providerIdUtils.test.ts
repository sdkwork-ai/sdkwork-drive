import { describe, expect, it } from 'vitest';
import {
  buildProviderIdWithUuid,
  createProviderUuidSuffix,
  resolveProviderKindSlug,
  slugifyProviderIdPart,
} from '../src/utils/providerIdUtils';

describe('providerIdUtils', () => {
  it('slugifies provider kind labels', () => {
    expect(slugifyProviderIdPart('COS')).toBe('cos');
    expect(slugifyProviderIdPart('TOS')).toBe('tos');
  });

  it('builds ids as kind prefix plus uuid suffix', () => {
    const id = buildProviderIdWithUuid('cos', '550e8400-e29b-41d4-a716-446655440000');
    expect(id).toBe('cos-550e8400-e29b-41d4-a716-446655440000');
  });

  it('avoids cos-cos style collisions from display names', () => {
    const id = buildProviderIdWithUuid('cos', '7f3a9b2c-e29b-4d1a-b716-446655440000');
    expect(id.startsWith('cos-')).toBe(true);
    expect(id).not.toBe('cos-cos');
  });

  it('resolves custom kind slug with custom fallback', () => {
    expect(resolveProviderKindSlug('custom', '', 'S3')).toBe('custom');
    expect(resolveProviderKindSlug('custom', 'MinIO', 'S3')).toBe('minio');
    expect(resolveProviderKindSlug('tencent_cos', '', 'COS')).toBe('cos');
  });

  it('creates uuid suffix strings', () => {
    expect(createProviderUuidSuffix()).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i,
    );
  });
});
