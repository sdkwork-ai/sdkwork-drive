import { describe, expect, it } from 'vitest';

import {
  enrichSessionSnapshotFromAccessToken,
  readAccessTokenContextClaims,
  readAuthTokenUserClaims,
} from './accessTokenContextClaims';

function jwtWithClaims(claims: Record<string, unknown>): string {
  const header = btoa(JSON.stringify({ alg: 'none', typ: 'JWT' }))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '');
  const payload = btoa(JSON.stringify(claims))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '');
  return `${header}.${payload}.signature`;
}

describe('readAccessTokenContextClaims', () => {
  it('reads tenant and app context from bootstrap access-token claims', () => {
    const token = jwtWithClaims({
      tenant_id: '100001',
      user_id: '0',
      organization_id: '0',
      app_id: 'sdkwork-birdcoder',
      session_id: 'bootstrap-local-development',
    });

    expect(readAccessTokenContextClaims(token)).toEqual({
      tenantId: '100001',
      userId: '0',
      organizationId: '0',
      appId: 'sdkwork-birdcoder',
      sessionId: 'bootstrap-local-development',
    });
  });
});

describe('readAuthTokenUserClaims', () => {
  it('reads principal identity from auth-token claims', () => {
    const token = jwtWithClaims({ user_id: 'auth-user', session_id: 'auth-session' });
    expect(readAuthTokenUserClaims(token)).toEqual({
      userId: 'auth-user',
      sessionId: 'auth-session',
    });
  });
});

describe('enrichSessionSnapshotFromAccessToken', () => {
  it('hydrates missing context from an access-only bootstrap session', () => {
    const token = jwtWithClaims({
      tenant_id: '100001',
      user_id: '0',
      organization_id: '0',
      app_id: 'sdkwork-birdcoder',
    });

    expect(enrichSessionSnapshotFromAccessToken({ accessToken: token })).toEqual({
      accessToken: token,
      user: { id: '0' },
      context: {
        tenantId: '100001',
        userId: '0',
        organizationId: '0',
        appId: 'sdkwork-birdcoder',
      },
    });
  });

  it('derives tenantId from access token and userId from auth token', () => {
    const accessToken = jwtWithClaims({ tenant_id: '100001', user_id: 'bootstrap-user' });
    const authToken = jwtWithClaims({ user_id: 'auth-user' });

    expect(enrichSessionSnapshotFromAccessToken({ accessToken, authToken })).toEqual({
      accessToken,
      authToken,
      user: { id: 'auth-user' },
      context: {
        tenantId: '100001',
        userId: 'auth-user',
      },
    });
  });

  it('derives identity from tokens instead of host IAM context fields', () => {
    const accessToken = jwtWithClaims({ tenant_id: 'token-tenant', user_id: 'token-user' });
    const authToken = jwtWithClaims({ user_id: 'auth-user' });

    expect(enrichSessionSnapshotFromAccessToken({
      accessToken,
      authToken,
      user: { id: 'iam-user', displayName: 'Ada' },
      context: { tenantId: 'iam-tenant', userId: 'iam-user' },
    })).toEqual({
      accessToken,
      authToken,
      user: { id: 'auth-user', displayName: 'Ada' },
      context: { tenantId: 'token-tenant', userId: 'auth-user' },
    });
  });
});
