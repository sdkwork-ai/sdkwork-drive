import { describe, expect, it } from 'vitest';
import { getAllProviderKindMeta } from '../src/utils/providerKindConfig';

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
});
