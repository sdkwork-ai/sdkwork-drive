import type { StorageProviderAccountScope } from '../types/storageProviderAdminTypes';

/**
 * The tenant that owns installation-wide configuration.
 *
 * Mirrors `PLATFORM_TENANT_ID` in
 * `crates/sdkwork-iam-provider-account-service/src/model.rs`, which is the value
 * the account centre stamps on a `platform`-scope row and compares against
 * before it accepts one. It is also the tenant this application's backend-admin
 * profile runs as (`sdkwork.app.config.json` → `backend.profileKey`
 * `backend-root-admin`, `backend.tenantId`). No shared front-end package exports
 * it, so it is restated once here rather than repeated per component.
 */
export const PLATFORM_TENANT_ID = '100001';

/**
 * Which scope a newly registered cloud account should default to.
 *
 * The platform tenant is where installation-wide accounts belong. `platform` is
 * the only scope that every tenant can both *see* in its account picker and
 * *resolve* through, so it is what makes one cloud account reusable across the
 * whole installation — the reason the storage console exists. Every other tenant
 * can only publish as far as `tenant`; asking for `platform` from one is refused
 * by the server with a 403, so defaulting it there would offer a choice that
 * cannot succeed.
 */
export function defaultAccountScopeForTenant(
  tenantId: string | undefined,
): StorageProviderAccountScope {
  return tenantId?.trim() === PLATFORM_TENANT_ID ? 'platform' : 'tenant';
}

/**
 * Which scopes the "new account" form may offer, narrowest first.
 *
 * Always includes `tenant`, and includes `platform` only where the default is
 * `platform` — i.e. only on the platform tenant, because the server reserves
 * that scope: drive's create handler answers 403 `PermissionRequired` before the
 * account centre is even consulted, so anywhere else it is a choice that cannot
 * succeed. (`user` is excluded for a different reason: a shared provider cannot
 * reference a personal account, so it would fail with 409 on bind. Personal
 * accounts stay visible under the picker's "mine" filter, which is what explains
 * why they are not bindable.)
 *
 * Taking the already-derived default rather than re-reading the tenant keeps the
 * pre-selected value and the option set provably consistent: they cannot drift,
 * because one is computed from the other.
 */
export function creatableAccountScopes(
  defaultAccountScope: StorageProviderAccountScope,
): readonly StorageProviderAccountScope[] {
  return defaultAccountScope === 'platform' ? ['tenant', 'platform'] : ['tenant'];
}
