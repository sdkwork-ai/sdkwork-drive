import { describe, expect, it } from 'vitest';
import {
  PLATFORM_TENANT_ID,
  creatableAccountScopes,
  defaultAccountScopeForTenant,
} from '../src/utils/accountScopeDefaults';

describe('accountScopeDefaults', () => {
  it('pins the platform tenant to the value the account centre stamps', () => {
    // Mirrors `PLATFORM_TENANT_ID` in
    // `crates/sdkwork-iam-provider-account-service/src/model.rs`. Asserted as a
    // literal on purpose: if the server side ever moves, this test is the thing
    // that goes red and says the console is now defaulting the wrong scope.
    expect(PLATFORM_TENANT_ID).toBe('100001');
  });

  it('defaults a new account to the platform scope under the platform tenant', () => {
    expect(defaultAccountScopeForTenant(PLATFORM_TENANT_ID)).toBe('platform');
  });

  it('defaults a new account to the tenant scope anywhere else', () => {
    // `platform` is refused with 403 from a non-platform tenant, so offering it
    // as the default would present a choice that cannot succeed.
    expect(defaultAccountScopeForTenant('tenant-001')).toBe('tenant');
    expect(defaultAccountScopeForTenant('0')).toBe('tenant');
  });

  it('falls back to the tenant scope when the session carries no tenant', () => {
    // The page reads `getSession().context?.tenantId`, which is absent before a
    // session is hydrated. Falling back to the *narrowest* scope keeps an
    // unhydrated console from claiming a platform-wide publish it cannot make.
    expect(defaultAccountScopeForTenant(undefined)).toBe('tenant');
    expect(defaultAccountScopeForTenant('')).toBe('tenant');
    expect(defaultAccountScopeForTenant('   ')).toBe('tenant');
  });

  it('tolerates surrounding whitespace on the platform tenant', () => {
    expect(defaultAccountScopeForTenant('  100001  ')).toBe('platform');
  });

  it('never derives the personal scope', () => {
    // `user` belongs to one person; publishing it as a default would make a
    // shared provider depend on a personal credential. It stays an explicit
    // choice only.
    const inputs = [undefined, '', '   ', PLATFORM_TENANT_ID, ' 100001 ', 'tenant-001'];
    for (const input of inputs) {
      expect(defaultAccountScopeForTenant(input)).not.toBe('user');
    }
  });
});

describe('creatableAccountScopes', () => {
  it('offers platform only where the default is platform', () => {
    expect(creatableAccountScopes('platform')).toEqual(['tenant', 'platform']);
    expect(creatableAccountScopes('tenant')).toEqual(['tenant']);
  });

  it('keeps the preselected scope among the offered options', () => {
    // The whole point of deriving one from the other: the selector can never be
    // seeded with a value it cannot show.
    for (const scope of ['tenant', 'platform'] as const) {
      expect(creatableAccountScopes(scope)).toContain(scope);
    }
  });

  it('never offers the personal scope', () => {
    // A shared provider cannot reference a personal account (the binding gate
    // answers 409), so offering it would fail on submit.
    for (const scope of ['tenant', 'platform'] as const) {
      expect(creatableAccountScopes(scope)).not.toContain('user');
    }
  });

  it('agrees with the tenant derivation it is paired with', () => {
    // The coupling that keeps the two in step: a non-platform tenant gets the
    // tenant sub-window whose scope list excludes the reserved platform scope.
    const tenant = 'tenant-001';
    const platform = PLATFORM_TENANT_ID;
    expect(creatableAccountScopes(defaultAccountScopeForTenant(platform))).toContain('platform');
    expect(creatableAccountScopes(defaultAccountScopeForTenant(tenant))).not.toContain('platform');
    expect(creatableAccountScopes(defaultAccountScopeForTenant(undefined))).not.toContain('platform');
  });
});
