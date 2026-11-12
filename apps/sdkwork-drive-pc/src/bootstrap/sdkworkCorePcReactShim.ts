import { createRuntimeConfig } from 'sdkwork-drive-pc-core/config/runtimeConfig';
import {
  DEFAULT_SESSION_STORAGE_KEY,
  type SessionAppContextSnapshot,
  type SessionSnapshot,
  type SessionUserSnapshot,
} from 'sdkwork-drive-pc-core/session/sessionStore';

/**
 * Session patch handed over by the shared auth UI (`@sdkwork/core-pc-react`).
 *
 * Tokens are the fields the auth UI owns. `context` and `user` stay typed as
 * `unknown` because that package keeps its own context record with optional
 * ids; this bridge normalizes whatever arrives into the Drive session shape.
 */
export interface PcReactRuntimeSessionPatch {
  accessToken?: string | null;
  authToken?: string | null;
  refreshToken?: string | null;
  sessionId?: string | null;
  context?: unknown;
  user?: unknown;
  updatedAt?: string | null;
}

/**
 * Persisted session record: the Drive `SessionSnapshot` itself.
 *
 * The Drive runtime and the auth flow write the same storage key, so this bridge
 * has to preserve `context` and `user` instead of rewriting the record as three
 * bare tokens. Dropping the context would leave every account-scoped key and
 * cache without an owner.
 */
export type PcReactRuntimeSession = SessionSnapshot;

let runtimeSessionCache: PcReactRuntimeSession = {};

function resolveTokenStorage() {
  return createRuntimeConfig(import.meta.env).auth.tokenStorage;
}

/**
 * Browser storage that persists the session for the configured token mode.
 *
 * `browser-session` must not leave a durable credential behind, so it reads and
 * writes `window.sessionStorage`. Every mode adopts a session persisted by the
 * other mode once, which keeps existing installs signed in.
 */
function readBrowserStorage(): Storage | undefined {
  if (typeof window === 'undefined') {
    return undefined;
  }

  const tokenStorage = resolveTokenStorage();
  if (tokenStorage === 'browser-local') {
    migrateLegacyBrowserSession('local');
    return window.localStorage;
  }
  if (tokenStorage === 'browser-session') {
    migrateLegacyBrowserSession('session');
    return window.sessionStorage;
  }
  return undefined;
}

function migrateLegacyBrowserSession(target: 'local' | 'session'): void {
  const sessionStore = window.sessionStorage as Storage | undefined;
  const localStore = window.localStorage as Storage | undefined;
  const source = target === 'local' ? sessionStore : localStore;
  const destination = target === 'local' ? localStore : sessionStore;
  if (!source || !destination) {
    return;
  }

  try {
    const legacySession = source.getItem(DEFAULT_SESSION_STORAGE_KEY);
    if (!legacySession) {
      return;
    }
    if (!destination.getItem(DEFAULT_SESSION_STORAGE_KEY)) {
      destination.setItem(DEFAULT_SESSION_STORAGE_KEY, legacySession);
    }
    source.removeItem(DEFAULT_SESSION_STORAGE_KEY);
  } catch {
    // Best-effort migration; a failed move only costs one re-login.
  }
}

function normalizeToken(value: unknown): string | undefined {
  const normalized = typeof value === 'string' ? value.trim() : '';
  return normalized.replace(/^Bearer\s+/i, '') || undefined;
}

function normalizeString(value: unknown): string | undefined {
  const normalized = typeof value === 'string' ? value.trim() : '';
  return normalized || undefined;
}

function normalizeStringList(value: unknown): string[] | undefined {
  if (!Array.isArray(value)) {
    return undefined;
  }
  const items = value.filter((item): item is string => typeof item === 'string' && item.trim() !== '');
  return items.length > 0 ? items : undefined;
}

/**
 * Keep only a context that names an account.
 *
 * A context without both ids cannot scope anything, so it is treated as absent
 * rather than stored as a half-known owner.
 */
function normalizeContext(value: unknown): SessionAppContextSnapshot | undefined {
  if (!value || typeof value !== 'object') {
    return undefined;
  }
  const record = value as Record<string, unknown>;
  const tenantId = normalizeString(record.tenantId);
  const userId = normalizeString(record.userId);
  if (!tenantId || !userId) {
    return undefined;
  }

  const context: SessionAppContextSnapshot = { tenantId, userId };
  const organizationId = normalizeString(record.organizationId);
  if (organizationId) {
    context.organizationId = organizationId;
  }
  const sessionId = normalizeString(record.sessionId);
  if (sessionId) {
    context.sessionId = sessionId;
  }
  for (const key of [
    'appId',
    'environment',
    'deploymentMode',
    'authLevel',
    'actorId',
    'actorKind',
    'deviceId',
  ] as const) {
    const normalized = normalizeString(record[key]);
    if (normalized) {
      context[key] = normalized;
    }
  }
  const dataScope = normalizeStringList(record.dataScope);
  if (dataScope) {
    context.dataScope = dataScope;
  }
  const permissionScope = normalizeStringList(record.permissionScope);
  if (permissionScope) {
    context.permissionScope = permissionScope;
  }
  return context;
}

function normalizeUser(value: unknown): SessionUserSnapshot | undefined {
  if (!value || typeof value !== 'object') {
    return undefined;
  }
  const record = value as Record<string, unknown>;
  const id = normalizeString(record.id) ?? normalizeString(record.userId);
  if (!id) {
    return undefined;
  }

  const user: SessionUserSnapshot = { id };
  const displayName = normalizeString(record.displayName);
  if (displayName) {
    user.displayName = displayName;
  }
  const avatarUrl = normalizeString(record.avatarUrl);
  if (avatarUrl) {
    user.avatarUrl = avatarUrl;
  }
  const email = normalizeString(record.email);
  if (email) {
    user.email = email;
  }
  return user;
}

/** Normalize a stored or committed record to the shape the Drive runtime reads. */
function toPersistedSession(value: unknown): PcReactRuntimeSession {
  if (!value || typeof value !== 'object') {
    return {};
  }
  const record = value as Record<string, unknown>;
  const next: PcReactRuntimeSession = {};
  const accessToken = normalizeToken(record.accessToken);
  if (accessToken) {
    next.accessToken = accessToken;
  }
  const authToken = normalizeToken(record.authToken);
  if (authToken) {
    next.authToken = authToken;
  }
  const refreshToken = normalizeToken(record.refreshToken);
  if (refreshToken) {
    next.refreshToken = refreshToken;
  }
  const sessionId = normalizeString(record.sessionId);
  if (sessionId) {
    next.sessionId = sessionId;
  }
  const context = normalizeContext(record.context);
  if (context) {
    next.context = context;
  }
  const user = normalizeUser(record.user);
  if (user) {
    next.user = user;
  }
  const updatedAt = normalizeString(record.updatedAt);
  if (updatedAt) {
    next.updatedAt = updatedAt;
  }
  return next;
}

function readStoredSession(): PcReactRuntimeSession {
  if (resolveTokenStorage() === 'os-secure-storage') {
    return runtimeSessionCache;
  }

  const storage = readBrowserStorage();
  if (!storage) {
    return {};
  }

  try {
    const raw = storage.getItem(DEFAULT_SESSION_STORAGE_KEY);
    if (!raw) {
      return {};
    }
    return toPersistedSession(JSON.parse(raw) as unknown);
  } catch {
    return {};
  }
}

export function primePcReactRuntimeSessionCache(session: SessionSnapshot): void {
  runtimeSessionCache = toPersistedSession(session);
}

export function readPcReactRuntimeSession(): PcReactRuntimeSession {
  return readStoredSession();
}

export function persistPcReactRuntimeSession(
  session: PcReactRuntimeSessionPatch,
): PcReactRuntimeSession {
  const next: PcReactRuntimeSession = { ...readStoredSession() };

  // Tokens are patch semantics: an omitted field keeps the stored value, an
  // empty one clears it.
  const tokenFields = [
    ['accessToken', session.accessToken],
    ['authToken', session.authToken],
    ['refreshToken', session.refreshToken],
  ] as const;
  for (const [field, value] of tokenFields) {
    if (value === undefined) {
      continue;
    }
    const normalized = normalizeToken(value);
    if (normalized) {
      next[field] = normalized;
    } else {
      delete next[field];
    }
  }

  // Identity is owned by whoever commits the session: a new context/user must
  // replace the previous account's so scoped keys and caches follow the sign-in.
  if (session.context !== undefined) {
    const context = normalizeContext(session.context);
    if (context) {
      next.context = context;
    } else {
      delete next.context;
    }
  }
  if (session.user !== undefined) {
    const user = normalizeUser(session.user);
    if (user) {
      next.user = user;
    } else {
      delete next.user;
    }
  }
  if (session.sessionId !== undefined) {
    const sessionId = normalizeString(session.sessionId);
    if (sessionId) {
      next.sessionId = sessionId;
    } else {
      delete next.sessionId;
    }
  }

  const hasToken = Boolean(next.accessToken || next.authToken || next.refreshToken);
  const storage = readBrowserStorage();
  if (!hasToken) {
    runtimeSessionCache = {};
    try {
      storage?.removeItem(DEFAULT_SESSION_STORAGE_KEY);
    } catch {
      // Logout must remain resilient when storage cleanup fails.
    }
    return {};
  }

  next.updatedAt = normalizeString(session.updatedAt) ?? new Date().toISOString();
  runtimeSessionCache = next;
  if (storage) {
    try {
      storage.setItem(DEFAULT_SESSION_STORAGE_KEY, JSON.stringify(next));
    } catch {
      // Session recovery is best-effort; the in-memory snapshot still works.
    }
  }
  return next;
}

export function clearPcReactRuntimeSession(): void {
  runtimeSessionCache = {};
  try {
    readBrowserStorage()?.removeItem(DEFAULT_SESSION_STORAGE_KEY);
  } catch {
    // Logout must remain resilient when storage cleanup fails.
  }
}

export function resolveAppClientAccessToken(): string {
  return normalizeToken(readStoredSession().accessToken) ?? '';
}

export function getAppClientWithSession(): never {
  throw new Error(
    'Drive PC does not expose a generic app client. Use the injected appbase IAM runtime or Drive SDK service boundary.',
  );
}
