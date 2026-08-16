import { createRuntimeConfig } from 'sdkwork-drive-pc-core/config/runtimeConfig';
import {
  DEFAULT_SESSION_STORAGE_KEY,
  type SessionSnapshot,
} from 'sdkwork-drive-pc-core/session/sessionStore';

export interface PcReactRuntimeSession {
  accessToken?: string;
  authToken?: string;
  refreshToken?: string;
}

const SESSION_STORAGE_KEY = DEFAULT_SESSION_STORAGE_KEY;
let runtimeSessionCache: PcReactRuntimeSession = {};

function readBrowserStorage(): Storage | undefined {
  if (typeof window === 'undefined') {
    return undefined;
  }

  const tokenStorage = createRuntimeConfig(import.meta.env).auth.tokenStorage;
  if (tokenStorage === 'browser-local' || tokenStorage === 'browser-session') {
    migrateLegacyBrowserSession();
    return window.localStorage;
  }
  return undefined;
}

function migrateLegacyBrowserSession(): void {
  const legacySession = window.sessionStorage.getItem(SESSION_STORAGE_KEY);
  if (legacySession && !window.localStorage.getItem(SESSION_STORAGE_KEY)) {
    window.localStorage.setItem(SESSION_STORAGE_KEY, legacySession);
  }
  if (legacySession) {
    window.sessionStorage.removeItem(SESSION_STORAGE_KEY);
  }
}

function normalizeToken(value: unknown): string | undefined {
  const normalized = typeof value === 'string' ? value.trim() : '';
  return normalized.replace(/^Bearer\s+/i, '') || undefined;
}

function toRuntimeSession(session: SessionSnapshot): PcReactRuntimeSession {
  return {
    accessToken: normalizeToken(session.accessToken),
    authToken: normalizeToken(session.authToken),
    refreshToken: normalizeToken(session.refreshToken),
  };
}

export function primePcReactRuntimeSessionCache(session: SessionSnapshot): void {
  runtimeSessionCache = toRuntimeSession(session);
}

function readStoredSession(): PcReactRuntimeSession {
  const tokenStorage = createRuntimeConfig(import.meta.env).auth.tokenStorage;
  if (tokenStorage === 'os-secure-storage') {
    return runtimeSessionCache;
  }

  const storage = readBrowserStorage();
  if (!storage) {
    return {};
  }

  try {
    const raw = storage.getItem(SESSION_STORAGE_KEY);
    if (!raw) {
      return {};
    }
    const parsed = JSON.parse(raw) as PcReactRuntimeSession;
    return {
      accessToken: normalizeToken(parsed.accessToken),
      authToken: normalizeToken(parsed.authToken),
      refreshToken: normalizeToken(parsed.refreshToken),
    };
  } catch {
    return {};
  }
}

export function readPcReactRuntimeSession(): PcReactRuntimeSession {
  return readStoredSession();
}

export function persistPcReactRuntimeSession(
  tokens: PcReactRuntimeSession,
): PcReactRuntimeSession {
  const current = readStoredSession();
  const next = {
    accessToken: tokens.accessToken !== undefined
      ? normalizeToken(tokens.accessToken)
      : current.accessToken,
    authToken: tokens.authToken !== undefined
      ? normalizeToken(tokens.authToken)
      : current.authToken,
    refreshToken: tokens.refreshToken !== undefined
      ? normalizeToken(tokens.refreshToken)
      : current.refreshToken,
  };
  runtimeSessionCache = next;
  const storage = readBrowserStorage();

  if (storage) {
    if (!next.accessToken && !next.authToken && !next.refreshToken) {
      storage.removeItem(SESSION_STORAGE_KEY);
    } else {
      storage.setItem(SESSION_STORAGE_KEY, JSON.stringify(next));
    }
  }

  return next;
}

export function clearPcReactRuntimeSession(): void {
  runtimeSessionCache = {};
  readBrowserStorage()?.removeItem(SESSION_STORAGE_KEY);
}

export function resolveAppClientAccessToken(): string {
  return normalizeToken(readStoredSession().accessToken) ?? '';
}

export function getAppClientWithSession(): never {
  throw new Error(
    'Drive PC does not expose a generic app client. Use the injected appbase IAM runtime or Drive SDK service boundary.',
  );
}
