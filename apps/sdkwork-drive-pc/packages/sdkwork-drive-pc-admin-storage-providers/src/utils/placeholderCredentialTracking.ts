import type { StorageProviderAccountDefaultView } from '../types/storageProviderAdminTypes';

/**
 * Ids of providers whose credential is still the bootstrap placeholder.
 *
 * "Configured" and "ready to use" are deliberately different questions. The
 * server's `credentialConfigured` only means an active credential exists, and
 * a placeholder is one — so a freshly initialized plane reports every provider
 * as configured while none of them can actually reach a store. Marking the
 * placeholder rows is what turns the list into an operator worklist instead of
 * a wall of green checks.
 *
 * The set is *merged* rather than replaced on each initialization run: a
 * re-run returns `credentialSeeded: false` for every row it found already
 * present, and replacing on that would forget the placeholders a previous run
 * seeded. Providers that were never seeded (the credential-free kind, or a
 * row an operator created by hand) are left out of the set entirely.
 */
export function mergePlaceholderProviders(
  current: ReadonlySet<string>,
  rows: readonly StorageProviderAccountDefaultView[],
  providerIds: readonly string[],
): Set<string> {
  const next = new Set(current);
  for (const row of rows) {
    if (!row.credentialSeeded) {
      continue;
    }
    if (providerIds.includes(row.providerId)) {
      next.add(row.providerId);
    }
  }
  return next;
}

/**
 * Drop any provider whose credential is no longer the placeholder.
 *
 * A provider whose placeholder an operator has since replaced with real keys
 * must lose the "待配置" hint, whatever else changed. The signals are:
 *   * it is no longer in the provider list (deleted); or
 *   * it is now reachable, which a placeholder never is; or
 *   * the operator edited it, surfaced as a `credentialRef` / account change.
 *
 * `reachableProviderIds` is the post-save or post-self-check connectivity
 * result, so this reads the same evidence the operator does rather than
 * guessing from a flag.
 */
export function pruneResolvedPlaceholders(
  placeholders: ReadonlySet<string>,
  liveProviderIds: readonly string[],
  reachableProviderIds: readonly string[],
): Set<string> {
  const reachable = new Set(reachableProviderIds);
  const live = new Set(liveProviderIds);
  const next = new Set<string>();
  for (const id of placeholders) {
    if (live.has(id) && !reachable.has(id)) {
      next.add(id);
    }
  }
  return next;
}

/**
 * Whether this provider should be shown as "待配置" (placeholder, pending edit).
 *
 * Extracted so the answer is testable without rendering: a provider is pending
 * when it was seeded from a placeholder and is still not reachable. A disabled
 * provider is not pending — a deliberately parked provider is not a gap.
 */
export function isPendingPlaceholder(
  providerId: string,
  placeholders: ReadonlySet<string>,
  reachable: boolean,
  status: string,
): boolean {
  return status === 'active' && placeholders.has(providerId) && !reachable;
}
