import { describe, expect, it } from 'vitest';
import {
  buildDriveAuthLoginRedirect,
  resolveDriveAuthGateDecision,
  sanitizeDriveAuthRedirect,
} from './authGate';

describe('drive auth gate', () => {
  it('redirects anonymous protected routes to the appbase IAM login entry with a normalized return target', () => {
    expect(
      buildDriveAuthLoginRedirect({
        pathname: 'files',
        search: '?view=grid',
        hash: '#selected',
      }),
    ).toBe('/auth/login?redirect=%2Ffiles%3Fview%3Dgrid%23selected');

    expect(
      resolveDriveAuthGateDecision({
        hasSession: false,
        location: {
          pathname: '/drive',
          search: '?space=personal',
          hash: '#node-001',
        },
      }),
    ).toEqual({
      kind: 'redirect',
      to: '/auth/login?redirect=%2Fdrive%3Fspace%3Dpersonal%23node-001',
      replace: true,
    });
  });

  it('keeps auth routes public and sends authenticated users back to a safe target', () => {
    expect(
      resolveDriveAuthGateDecision({
        hasSession: false,
        location: { pathname: '/auth/login', search: '', hash: '' },
      }),
    ).toEqual({ kind: 'auth-route' });

    expect(
      resolveDriveAuthGateDecision({
        hasSession: false,
        location: { pathname: '/auth/login', search: '?flow=register', hash: '' },
      }),
    ).toEqual({ kind: 'auth-route' });

    expect(
      resolveDriveAuthGateDecision({
        hasSession: false,
        location: { pathname: '/auth/register', search: '', hash: '' },
      }),
    ).toEqual({ kind: 'auth-route' });

    expect(
      resolveDriveAuthGateDecision({
        hasSession: true,
        location: {
          pathname: '/auth/login',
          search: '?redirect=%2Fdrive%3Fspace%3Dteam',
          hash: '',
        },
      }),
    ).toEqual({
      kind: 'redirect',
      to: '/drive?space=team',
      replace: true,
    });
  });

  it('rejects unsafe auth redirect targets and auth-route loops', () => {
    expect(sanitizeDriveAuthRedirect('https://evil.example/drive')).toBe('/');
    expect(sanitizeDriveAuthRedirect('//evil.example/drive')).toBe('/');
    expect(sanitizeDriveAuthRedirect('/auth/login')).toBe('/');
    expect(sanitizeDriveAuthRedirect('/auth/login?redirect=%2Fdrive')).toBe('/');
    expect(sanitizeDriveAuthRedirect('/drive?tab=recent#node-1')).toBe(
      '/drive?tab=recent#node-1',
    );
  });

  it('allows anonymous share-link claim routes without forcing IAM login first', () => {
    expect(
      resolveDriveAuthGateDecision({
        hasSession: false,
        location: {
          pathname: '/share/public-share-token',
          search: '',
          hash: '',
        },
      }),
    ).toEqual({ kind: 'product-route' });
  });

  it('lets host-managed integrations avoid mounting duplicate IAM routes', () => {
    expect(
      resolveDriveAuthGateDecision({
        hasSession: false,
        integrationMode: 'host-managed',
        location: {
          pathname: '/drive',
          search: '?space=embedded',
          hash: '',
        },
      }),
    ).toEqual({ kind: 'product-route' });

    expect(
      resolveDriveAuthGateDecision({
        hasSession: true,
        integrationMode: 'host-managed',
        location: {
          pathname: '/auth/login',
          search: '?redirect=%2Fdrive',
          hash: '',
        },
      }),
    ).toEqual({ kind: 'product-route' });
  });
});
