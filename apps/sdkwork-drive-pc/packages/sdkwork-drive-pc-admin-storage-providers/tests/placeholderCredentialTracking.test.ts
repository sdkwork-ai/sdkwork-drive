import { describe, expect, it } from 'vitest';
import {
  isPendingPlaceholder,
  mergePlaceholderProviders,
  pruneResolvedPlaceholders,
} from '../src/utils/placeholderCredentialTracking';
import type { StorageProviderAccountDefaultView } from '../src/types/storageProviderAdminTypes';

function row(overrides: Partial<StorageProviderAccountDefaultView>): StorageProviderAccountDefaultView {
  return {
    providerKind: 'tencent_cos',
    providerId: 'builtin-storage-provider-tencent-cos',
    providerCreated: true,
    accountCreated: true,
    credentialSeeded: true,
    ...overrides,
  };
}

describe('placeholder credential tracking', () => {
  it('marks only the providers whose credential this run seeded', () => {
    const rows = [
      row({ providerId: 'a' }),
      // A re-run finds this one already present, so it seeded nothing.
      row({ providerId: 'b', credentialSeeded: false, providerCreated: false, accountCreated: false }),
    ];
    const merged = mergePlaceholderProviders(new Set<string>(), rows, ['a', 'b']);
    expect([...merged]).toEqual(['a']);
  });

  it('adds without dropping, leaving removal to the prune step', () => {
    // Deliberate separation of concerns: this merge only ever *adds*. Dropping
    // is `pruneResolvedPlaceholders`'s job, because only it knows the live
    // provider list and what has become reachable. Folding removal in here
    // would delete a placeholder merely because a run happened to list fewer
    // providers.
    const merged = mergePlaceholderProviders(new Set(['stale']), [row({ providerId: 'a' })], ['a']);
    expect([...merged].sort()).toEqual(['a', 'stale']);
  });

  it('drops a stale id when the prune step is given the live list', () => {
    const merged = mergePlaceholderProviders(new Set(['stale']), [row({ providerId: 'a' })], ['a']);
    const pruned = pruneResolvedPlaceholders(merged, ['a'], []);
    expect([...pruned]).toEqual(['a']);
  });

  it('merges rather than replaces, so a re-run does not forget earlier placeholders', () => {
    // The exact regression this guards: the second bootstrap run returns
    // `credentialSeeded: false` for every pre-existing row, so replacing on it
    // would silently clear every pending hint the first run had set.
    const first = mergePlaceholderProviders(new Set<string>(), [row({ providerId: 'a' })], ['a']);
    const second = mergePlaceholderProviders(
      first,
      [row({ providerId: 'a', credentialSeeded: false, providerCreated: false, accountCreated: false })],
      ['a'],
    );
    expect([...second]).toEqual(['a']);
  });

  it('clears a placeholder once the provider becomes reachable', () => {
    const pruned = pruneResolvedPlaceholders(new Set(['a', 'b']), ['a', 'b'], ['a']);
    expect([...pruned]).toEqual(['b']);
  });

  it('clears a placeholder whose provider was removed from the list', () => {
    const pruned = pruneResolvedPlaceholders(new Set(['a', 'b']), ['a'], ['a']);
    // `a` became reachable too, so nothing is left pending.
    expect([...pruned]).toEqual([]);
  });

  it('reports a provider as pending only while active, seeded, and unreachable', () => {
    const placeholders = new Set(['a']);
    expect(isPendingPlaceholder('a', placeholders, false, 'active')).toBe(true);
    expect(isPendingPlaceholder('a', placeholders, true, 'active')).toBe(false);
    expect(isPendingPlaceholder('a', placeholders, false, 'inactive')).toBe(false);
    expect(isPendingPlaceholder('unknown', placeholders, false, 'active')).toBe(false);
  });
});
