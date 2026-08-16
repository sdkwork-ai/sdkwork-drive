import { createDriveSessionTokenManager } from 'sdkwork-drive-pc-core';
import type {
  DriveRuntimeConfig,
  DriveSessionTokenManager,
  SessionAppContextSnapshot,
  SessionSnapshot,
  SessionStore,
  SessionUserSnapshot,
} from 'sdkwork-drive-pc-core';
import {
  createClient,
  type SdkworkAppClient,
} from '@sdkwork/iam-app-sdk';
import {
  createSdkworkAppbasePcAuthRuntime,
  type SdkworkAppbasePcAuthRuntimeComposition,
  type SdkworkAppbasePcAuthRuntimeSdkClient,
  type SdkworkAppbasePcAuthSessionBridgeSession,
} from '@sdkwork/auth-runtime-pc-react';
import type {
  IamAppContext,
  IamDeploymentMode,
  IamEnvironment,
} from '@sdkwork/iam-contracts';

const IAM_APP_SDK_FAMILY_ID = 'sdkwork-iam-app-sdk';
const APP_API_PREFIX = '/app/v3/api';

export type DriveIamRuntime = ReturnType<
  SdkworkAppbasePcAuthRuntimeComposition['getRuntime']
> & {
  onCurrentUserChanged?: (user: DriveIamUserLike | undefined) => void;
};

export interface CreateDriveIamRuntimeOptions {
  appClient?: unknown;
  config: DriveRuntimeConfig;
  localeProvider?: () => string | undefined;
  sdkClients?: Array<{ setTokenManager(manager: DriveSessionTokenManager): unknown }>;
  session: SessionStore;
  tokenManager?: DriveSessionTokenManager;
}

interface DriveIamUserLike {
  avatar?: unknown;
  displayName?: string;
  email?: string;
  id?: string;
  name?: string;
  nickname?: string;
  userId?: string;
  username?: string;
}

interface DriveIamSessionLike extends SdkworkAppbasePcAuthSessionBridgeSession {
  context?: IamAppContext;
  sessionId?: string;
  user?: DriveIamUserLike;
  userInfo?: DriveIamUserLike;
}

export function createDriveIamRuntime({
  appClient,
  config,
  localeProvider,
  sdkClients = [],
  session,
  tokenManager,
}: CreateDriveIamRuntimeOptions): DriveIamRuntime {
  const globalTokenManager = tokenManager ?? createDriveSessionTokenManager(session);
  const generatedAppClient =
    appClient ?? createDriveGeneratedAppClient({ config, tokenManager: globalTokenManager });
  const composition = createSdkworkAppbasePcAuthRuntime({
    app: {
      appId: config.appKey,
      deploymentMode: toIamDeploymentMode(config.deploymentProfile),
      environment: toIamEnvironment(config.environment),
      platform: 'pc',
    },
    baseUrls: {
      appbaseAppApiBaseUrl: resolveAppbaseAppApiBaseUrl(config),
    },
    createAppbaseAppClient: () => generatedAppClient as SdkworkAppClient,
    localeProvider,
    sdkClients: sdkClients as SdkworkAppbasePcAuthRuntimeSdkClient[],
    sessionBridge: {
      clearSession: () => {
        session.clearSession();
      },
      commitSession: (nextSession) =>
        commitDriveIamRuntimeSession(session, nextSession as DriveIamSessionLike),
      readSession: () => toDriveIamBridgeSession(session.getSnapshot()),
    },
    tokenManager: globalTokenManager as never,
  });
  const runtime = composition.runtime as DriveIamRuntime;

  runtime.onCurrentUserChanged = (user) => {
    mergeDriveSession(session, {
      user: user ? toDriveSessionUser(user) : undefined,
    });
  };
  bindDriveSessionProjection(runtime, session);

  return runtime;
}

function createDriveGeneratedAppClient({
  config,
  tokenManager,
}: {
  config: DriveRuntimeConfig;
  tokenManager: DriveSessionTokenManager;
}): SdkworkAppClient {
  const client = createClient({
    authMode: 'dual-token',
    baseUrl: normalizeGeneratedSdkBaseUrl(
      resolveAppbaseAppApiBaseUrl(config),
      APP_API_PREFIX,
    ),
    tokenManager: tokenManager as never,
  });

  return ensureIamTenantSelectionCompat(client);
}

function ensureIamTenantSelectionCompat(client: SdkworkAppClient): SdkworkAppClient {
  const sessions = (client as unknown as {
    auth?: {
      sessions?: Record<string, unknown>;
    };
  }).auth?.sessions;
  if (!sessions || sessions.tenantSelection) {
    return client;
  }

  const organizationSelection = sessions.organizationSelection as
    | { create?: (...args: unknown[]) => unknown }
    | undefined;
  if (organizationSelection?.create) {
    sessions.tenantSelection = {
      create: organizationSelection.create.bind(organizationSelection),
    };
    return client;
  }

  sessions.tenantSelection = {
    create: async () => {
      throw new Error('appbase app SDK is missing sessions.tenantSelection.create');
    },
  };
  return client;
}

function resolveAppbaseAppApiBaseUrl(config: DriveRuntimeConfig): string {
  return config.sdkBaseUrls.dependencySdkBaseUrls[IAM_APP_SDK_FAMILY_ID]?.appApiBaseUrl
    ?? config.appApiBaseUrl;
}

function normalizeGeneratedSdkBaseUrl(
  baseUrl: string,
  apiPrefix: string,
): string {
  const normalizedBaseUrl = baseUrl.replace(/\/+$/, '');
  const normalizedApiPrefix = apiPrefix.replace(/\/+$/, '');
  if (normalizedBaseUrl.endsWith(normalizedApiPrefix)) {
    return normalizedBaseUrl.slice(0, -normalizedApiPrefix.length) || normalizedBaseUrl;
  }
  return normalizedBaseUrl;
}

function bindDriveSessionProjection(
  runtime: DriveIamRuntime,
  session: SessionStore,
): void {
  const auth = runtime.service.auth;
  wrapIamSessionMethod(auth.registrations, 'create', session, () =>
    hydrateDriveCurrentSession(runtime, session),
  );
  wrapIamSessionMethod(auth.sessions, 'create', session, () =>
    hydrateDriveCurrentSession(runtime, session),
  );
  wrapIamSessionMethod(auth.sessions, 'refresh', session, () =>
    hydrateDriveCurrentSession(runtime, session),
  );
  wrapIamSessionMethod(auth.sessions.current, 'retrieve', session);
  wrapIamSessionMethod(auth.sessions.current, 'update', session, () =>
    hydrateDriveCurrentSession(runtime, session),
  );

  const oauth = runtime.service.oauth;
  wrapIamSessionMethod(oauth.deviceAuthorizations, 'create', session);
  wrapIamSessionMethod(oauth.deviceAuthorizations, 'retrieve', session);
  wrapIamSessionMethod(oauth.deviceAuthorizations.passwordCompletions, 'create', session, () =>
    hydrateDriveCurrentSession(runtime, session),
  );
  wrapIamSessionMethod(oauth.deviceAuthorizations.scans, 'create', session);

  const usersCurrent = runtime.service.iam.users.current as {
    retrieve: () => Promise<DriveIamUserLike>;
  };
  const retrieveCurrentUser = usersCurrent.retrieve.bind(usersCurrent);
  usersCurrent.retrieve = async () => {
    const user = await retrieveCurrentUser();
    runtime.onCurrentUserChanged?.(user);
    return user;
  };
}

function wrapIamSessionMethod(
  resource: object,
  methodName: string,
  session: SessionStore,
  hydrateContext?: () => Promise<void>,
): void {
  const mutableResource = resource as Record<string, unknown>;
  const original = mutableResource[methodName];
  if (typeof original !== 'function') {
    return;
  }

  mutableResource[methodName] = async (...args: unknown[]) => {
    const result = await original.apply(resource, args);
    syncDriveIamSession(session, result as DriveIamSessionLike);
    if (hydrateContext && shouldHydrateDriveAppContext(result, session)) {
      await hydrateContext();
    }
    return result;
  };
}

function commitDriveIamRuntimeSession(
  session: SessionStore,
  iamSession: DriveIamSessionLike,
): DriveIamSessionLike | undefined {
  const nextSession: SessionSnapshot = {
    ...session.getSnapshot(),
    accessToken: iamSession.accessToken,
    authToken: iamSession.authToken,
    refreshToken: iamSession.refreshToken,
    sessionId: iamSession.sessionId ?? iamSession.context?.sessionId,
  };

  if (iamSession.context) {
    nextSession.context = toDriveSessionContext(iamSession.context);
  } else {
    delete nextSession.context;
  }

  replaceDriveSession(session, nextSession);

  return toDriveIamBridgeSession(session.getSnapshot()) ?? undefined;
}

function toDriveIamBridgeSession(
  snapshot: SessionSnapshot,
): DriveIamSessionLike | null {
  if (!snapshot.authToken && !snapshot.accessToken && !snapshot.refreshToken) {
    return null;
  }

  const context = toIamAppContext(snapshot.context);
  return {
    ...(snapshot.accessToken ? { accessToken: snapshot.accessToken } : {}),
    ...(snapshot.authToken ? { authToken: snapshot.authToken } : {}),
    ...(snapshot.refreshToken ? { refreshToken: snapshot.refreshToken } : {}),
    ...(snapshot.sessionId ? { sessionId: snapshot.sessionId } : {}),
    ...(context ? { context } : {}),
  };
}

async function hydrateDriveCurrentSession(
  runtime: DriveIamRuntime,
  session: SessionStore,
): Promise<void> {
  if (session.getSnapshot().context?.tenantId) {
    return;
  }
  await runtime.service.auth.sessions.current.retrieve();
}

function shouldHydrateDriveAppContext(
  value: unknown,
  session: SessionStore,
): boolean {
  if (session.getSnapshot().context?.tenantId) {
    return false;
  }
  const sessionLike = value as DriveIamSessionLike | undefined;
  return Boolean(sessionLike?.authToken && sessionLike.accessToken && !sessionLike.context);
}

function syncDriveIamSession(
  session: SessionStore,
  iamSession: DriveIamSessionLike,
): void {
  mergeDriveSession(session, {
    accessToken: iamSession.accessToken,
    authToken: iamSession.authToken,
    context: iamSession.context
      ? toDriveSessionContext(iamSession.context)
      : undefined,
    refreshToken: iamSession.refreshToken,
    sessionId: iamSession.sessionId ?? iamSession.context?.sessionId,
    user: iamSession.user || iamSession.userInfo
      ? toDriveSessionUser((iamSession.user ?? iamSession.userInfo)!)
      : undefined,
  });
}

function toDriveSessionContext(context: IamAppContext): SessionAppContextSnapshot {
  return {
    tenantId: context.tenantId,
    userId: context.userId,
    organizationId: context.organizationId,
    sessionId: context.sessionId,
    appId: context.appId,
    environment: context.environment,
    deploymentMode: context.deploymentMode,
    authLevel: context.authLevel,
    dataScope: [...context.dataScope],
    permissionScope: [...context.permissionScope],
    actorId: context.actorId || context.userId,
    actorKind: context.actorKind || 'user',
  };
}

function toIamAppContext(
  context: SessionAppContextSnapshot | undefined,
): IamAppContext | undefined {
  if (!context?.tenantId || !context.userId || !context.sessionId) {
    return undefined;
  }

  return {
    appId: context.appId ?? 'sdkwork-drive-pc',
    authLevel: toIamAuthLevel(context.authLevel),
    dataScope: [...(context.dataScope ?? [])],
    deploymentMode: toIamDeploymentMode(context.deploymentMode),
    environment: toIamEnvironment(context.environment),
    organizationId: context.organizationId,
    permissionScope: [...(context.permissionScope ?? [])],
    sessionId: context.sessionId,
    tenantId: context.tenantId,
    userId: context.userId,
  };
}

function toDriveSessionUser(user: DriveIamUserLike): SessionUserSnapshot {
  const id = normalizeScalar(user.id) ?? normalizeScalar(user.userId) ?? 'drive-user';
  const displayName =
    normalizeScalar(user.displayName)
    ?? normalizeScalar(user.name)
    ?? normalizeScalar(user.nickname)
    ?? normalizeScalar(user.username);

  return {
    id,
    displayName,
    avatarUrl: resolveMediaUrl(user.avatar),
    email: normalizeScalar(user.email),
  };
}

function mergeDriveSession(
  session: SessionStore,
  patch: Partial<SessionSnapshot>,
): void {
  replaceDriveSession(session, {
    ...session.getSnapshot(),
    ...compactSessionPatch(patch),
  });
}

function replaceDriveSession(
  session: SessionStore,
  nextSession: SessionSnapshot,
): void {
  const compact = compactSessionPatch(nextSession) as SessionSnapshot;
  if (!compact.authToken && !compact.accessToken && !compact.refreshToken) {
    session.clearSession();
    return;
  }

  session.setSession(compact);
}

function compactSessionPatch<T extends object>(value: T): Partial<T> {
  return Object.fromEntries(
    Object.entries(value).filter(([, entry]) => entry !== undefined),
  ) as Partial<T>;
}

function toIamDeploymentMode(
  value: DriveRuntimeConfig['deploymentProfile'] | string | undefined,
): IamDeploymentMode {
  if (value === 'cloud' || value === 'saas') {
    return 'saas';
  }
  return 'private';
}

function toIamEnvironment(value: string | undefined): IamEnvironment {
  const normalized = String(value ?? '').trim().toLowerCase();
  if (normalized === 'prod' || normalized === 'production' || normalized === 'staging') {
    return 'prod';
  }
  if (normalized === 'test' || normalized === 'testing') {
    return 'test';
  }
  return 'dev';
}

function toIamAuthLevel(value: string | undefined): IamAppContext['authLevel'] {
  if (value === 'anonymous' || value === 'password' || value === 'mfa' || value === 'system') {
    return value;
  }
  return 'password';
}

function normalizeScalar(value: unknown): string | undefined {
  const normalized = typeof value === 'number' && Number.isFinite(value)
    ? String(value)
    : typeof value === 'string'
      ? value.trim()
      : '';
  return normalized || undefined;
}

function resolveMediaUrl(value: unknown): string | undefined {
  if (typeof value === 'string') {
    return normalizeScalar(value);
  }
  if (!value || typeof value !== 'object') {
    return undefined;
  }

  const record = value as Record<string, unknown>;
  return normalizeScalar(record.url)
    ?? normalizeScalar(record.deliveryUrl)
    ?? normalizeScalar(record.publicUrl)
    ?? normalizeScalar(record.cdnUrl);
}
