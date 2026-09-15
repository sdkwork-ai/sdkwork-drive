import { describe, expect, it } from 'vitest';
import { buildProviderEndpointUrl, getAllProviderKindMeta } from '../src/utils/providerKindConfig';

describe('providerKindConfig region metadata', () => {
  it('provides readable and internally consistent region options', () => {
    for (const provider of getAllProviderKindMeta()) {
      if (provider.regions.length === 0) {
        continue;
      }

      const values = provider.regions.map((region) => region.value);
      expect(new Set(values).size, `${provider.value} contains duplicate region values`).toBe(values.length);
      expect(values, `${provider.value} is missing its default region`).toContain(provider.regionHint);

      for (const region of provider.regions) {
        expect(region.label, `${provider.value}:${region.value} contains a replacement character`)
          .not.toContain('\uFFFD');
        expect(region.label, `${provider.value}:${region.value} hides the stable region code`)
          .toContain(region.value);
      }
    }
  });

  it('auto-fills an endpoint for every region-driven provider kind offered by the picker', () => {
    // Regression: `google_cloud_storage` is selectable and exposes a region
    // list, but had no entry in the region -> endpoint builders, so choosing a
    // region produced an empty endpoint instead of the vendor's global host.
    for (const provider of getAllProviderKindMeta()) {
      if (!provider.features.hasRegion || provider.regions.length === 0) {
        continue;
      }

      const endpoint = buildProviderEndpointUrl(provider.value, provider.regionHint);
      expect(endpoint, `${provider.value} has no region-driven endpoint`)
        .toBeDefined();
      expect(endpoint, `${provider.value} endpoint must be an https origin`)
        .toMatch(/^https:\/\/[a-z0-9.-]+$/);
    }
  });

  it('keeps the Google Cloud Storage endpoint global across regions', () => {
    expect(buildProviderEndpointUrl('google_cloud_storage', 'us-central1'))
      .toBe('https://storage.googleapis.com');
    expect(buildProviderEndpointUrl('google_cloud_storage', 'asia-northeast1'))
      .toBe('https://storage.googleapis.com');
  });

  it('returns no endpoint for kinds whose endpoint is operator-supplied', () => {
    expect(buildProviderEndpointUrl('local_filesystem', 'us-east-1')).toBeUndefined();
    expect(buildProviderEndpointUrl('custom:minio', 'us-east-1')).toBeUndefined();
    expect(buildProviderEndpointUrl('s3_compatible', '   ')).toBeUndefined();
  });
});
