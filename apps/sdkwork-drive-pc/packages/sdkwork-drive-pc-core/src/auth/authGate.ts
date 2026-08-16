import React, { useEffect, useMemo, useState } from 'react';
import type { SessionSnapshot, SessionStore } from '../session/sessionStore';

export interface DriveAuthLocationLike {
  hash?: string;
  pathname: string;
  search?: string;
}

export type DriveAuthGateDecision =
  | { kind: 'product-route' }
  | { kind: 'auth-route' }
  | { kind: 'redirect'; replace: true; to: string };

export type DriveAuthIntegrationMode = 'standalone' | 'host-managed';

export interface DriveAuthGateProps {
  authRoutes?: React.ReactNode;
  children: React.ReactNode;
  homePath?: string;
  integrationMode?: DriveAuthIntegrationMode;
  location?: DriveAuthLocationLike;
  navigate?: (to: string, options: { replace: true }) => void;
  session: SessionStore;
}

const DEFAULT_HOME_PATH = '/';
const AUTH_BASE_PATH = '/auth';
const AUTH_LOGIN_PATH = '/auth/login';

export function isDriveShareLinkClaimPath(pathname: string): boolean {
  const normalized = pathname.replace(/\/+$/, '') || '/';
  return /^\/share\/[^/]+$/u.test(normalized);
}

export function hasDriveIamSession(session: SessionSnapshot): boolean {
  return Boolean(session.authToken && session.accessToken && session.context?.tenantId);
}

export function buildDriveAuthLoginRedirect(location: DriveAuthLocationLike): string {
  const returnPath = `${normalizeDrivePathname(location.pathname)}${location.search ?? ''}${
    location.hash ?? ''
  }`;
  return `${AUTH_LOGIN_PATH}?redirect=${encodeURIComponent(returnPath)}`;
}

export function sanitizeDriveAuthRedirect(value: string | null | undefined): string {
  if (!value) {
    return DEFAULT_HOME_PATH;
  }

  let decoded = value;
  try {
    decoded = decodeURIComponent(value);
  } catch {
    return DEFAULT_HOME_PATH;
  }

  if (!decoded.startsWith('/') || decoded.startsWith('//')) {
    return DEFAULT_HOME_PATH;
  }

  const redirectUrl = new URL(decoded, 'http://sdkwork-drive.local');
  if (isDriveAuthRoute(redirectUrl.pathname)) {
    return DEFAULT_HOME_PATH;
  }

  return `${redirectUrl.pathname}${redirectUrl.search}${redirectUrl.hash}`;
}

export function resolveDriveAuthGateDecision({
  hasSession,
  homePath = DEFAULT_HOME_PATH,
  integrationMode = 'standalone',
  location,
}: {
  hasSession: boolean;
  homePath?: string;
  integrationMode?: DriveAuthIntegrationMode;
  location: DriveAuthLocationLike;
}): DriveAuthGateDecision {
  if (integrationMode === 'host-managed') {
    return { kind: 'product-route' };
  }

  const pathname = normalizeDrivePathname(location.pathname);
  if (isDriveAuthRoute(pathname)) {
    if (!hasSession) {
      return { kind: 'auth-route' };
    }

    const redirect = new URLSearchParams((location.search ?? '').replace(/^\?/, '')).get(
      'redirect',
    );
    return {
      kind: 'redirect',
      replace: true,
      to: sanitizeDriveAuthRedirect(redirect) || normalizeDrivePathname(homePath),
    };
  }

  if (isDriveShareLinkClaimPath(pathname)) {
    return { kind: 'product-route' };
  }

  if (!hasSession) {
    return {
      kind: 'redirect',
      replace: true,
      to: buildDriveAuthLoginRedirect(location),
    };
  }

  return { kind: 'product-route' };
}

export function DriveAuthGate({
  authRoutes,
  children,
  homePath = DEFAULT_HOME_PATH,
  integrationMode = 'standalone',
  location,
  navigate,
  session,
}: DriveAuthGateProps) {
  const [snapshot, setSnapshot] = useState(() => session.getSnapshot());
  const currentLocation = useBrowserLocation(location);

  useEffect(() => {
    setSnapshot(session.refreshSession());
    return session.subscribe(setSnapshot);
  }, [session, currentLocation.pathname, currentLocation.search, currentLocation.hash]);

  const decision = useMemo(
    () =>
      resolveDriveAuthGateDecision({
        hasSession: hasDriveIamSession(snapshot),
        homePath,
        integrationMode,
        location: currentLocation,
      }),
    [currentLocation, homePath, integrationMode, snapshot],
  );

  useEffect(() => {
    if (decision.kind !== 'redirect') {
      return;
    }
    if (navigate) {
      navigate(decision.to, { replace: true });
      return;
    }
    if (typeof window !== 'undefined') {
      window.location.replace(decision.to);
    }
  }, [decision, navigate]);

  if (decision.kind === 'redirect') {
    return null;
  }

  if (decision.kind === 'auth-route') {
    return React.createElement(React.Fragment, null, authRoutes);
  }

  return React.createElement(React.Fragment, null, children);
}

export function isDriveAuthRoute(pathname: string): boolean {
  return pathname === AUTH_BASE_PATH || pathname.startsWith(`${AUTH_BASE_PATH}/`);
}

function normalizeDrivePathname(pathname: string): string {
  const normalized = pathname.trim();
  if (!normalized) {
    return DEFAULT_HOME_PATH;
  }
  return normalized.startsWith('/') ? normalized : `/${normalized}`;
}

function useBrowserLocation(location: DriveAuthLocationLike | undefined): DriveAuthLocationLike {
  const [browserLocation, setBrowserLocation] = useState<DriveAuthLocationLike>(() =>
    location ?? readBrowserLocation(),
  );

  useEffect(() => {
    if (location) {
      setBrowserLocation(location);
      return undefined;
    }
    if (typeof window === 'undefined') {
      return undefined;
    }

    const update = () => setBrowserLocation(readBrowserLocation());
    window.addEventListener('popstate', update);
    window.addEventListener('hashchange', update);
    window.addEventListener('storage', update);
    return () => {
      window.removeEventListener('popstate', update);
      window.removeEventListener('hashchange', update);
      window.removeEventListener('storage', update);
    };
  }, [location]);

  return browserLocation;
}

function readBrowserLocation(): DriveAuthLocationLike {
  if (typeof window === 'undefined') {
    return { pathname: DEFAULT_HOME_PATH, search: '', hash: '' };
  }
  return {
    pathname: window.location.pathname,
    search: window.location.search,
    hash: window.location.hash,
  };
}
