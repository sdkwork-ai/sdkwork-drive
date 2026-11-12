import {
  createDriveUploaderClient,
  createGeneratedDriveAppClient,
  operations,
  sdkMetadata,
  type DriveUploaderClient,
  type DriveUploaderStateSnapshot,
  type DriveUploaderStateStore,
  type DriveUploaderTransport,
} from '@sdkwork/drive-app-sdk';
import type { DriveRuntimeConfig } from '../config/runtimeConfig';
import {
  driveStorageScopeKey,
  scopeDriveStorageKey,
  type DriveStorageScope,
} from '../session/driveStorageScope';
import type { SessionStorageLike } from '../session/sessionStore';
import type { DriveSessionTokenManager } from '../session/sessionTokenManager';
import { omitAuthProjectionBody, omitAuthProjectionQuery } from './authProjection';
import {
  assertStandardSdkWorkPaginationQuery,
  buildGeneratedSdkPath,
  compactQuery,
  isDriveRequestCancellationError,
  normalizeGeneratedSdkBaseUrl,
  normalizeGeneratedSdkError,
  type TokenManagerAwareGeneratedSdkClient,
} from './generatedSdkTransport';

export type DriveAppOperationId = keyof typeof operations;

export interface DriveAppSdkRequest {
  operationId: DriveAppOperationId;
  pathParams?: Record<string, string | number>;
  query?: Record<string, string | number | boolean | undefined>;
  body?: unknown;
  signal?: AbortSignal;
}

export interface DriveAppSdkClient {
  metadata: typeof sdkMetadata;
  operations: typeof operations;
  uploader: DriveUploaderClient;
  request<T>(request: DriveAppSdkRequest): Promise<T>;
  setTokenManager(manager: DriveSessionTokenManager): void;
}

export interface DriveAppSdkClientOptions {
  config: DriveRuntimeConfig;
  sdkClient?: TokenManagerAwareGeneratedSdkClient;
  tokenManager: DriveSessionTokenManager;
  uploaderStateStorage?: SessionStorageLike;
  /**
   * Resolves the account that owns persisted uploader state.
   *
   * Returning undefined (signed out, or a session without tenant/user context)
   * keeps uploader state in memory. An unscoped key would otherwise let the next
   * account on the same device resume the previous account's upload sessions.
   */
  resolveUploaderStorageScope?: () => DriveStorageScope | undefined;
}

const DRIVE_UPLOADER_STATE_STORAGE_KEY = 'sdkwork.drive.pc.uploader.state.v1';

function isUploaderStateSnapshot(value: unknown): value is DriveUploaderStateSnapshot {
  if (typeof value !== 'object' || value === null) {
    return false;
  }
  const record = value as Record<string, unknown>;
  return (
    typeof record.taskId === 'string'
    && typeof record.uploadItemId === 'string'
    && typeof record.uploadSessionId === 'string'
    && typeof record.updatedAtEpochMs === 'number'
    && Array.isArray(record.uploadedParts)
  );
}

/**
 * Uploader resume state: `taskId` -> server-side upload session and uploaded parts.
 *
 * The SDK looks a snapshot up by task id, so the owning account has to be
 * resolved on every call - one renderer outlives a sign-in. Entries that belong
 * to no account (or to a runtime without browser storage) live in memory only
 * and are dropped as soon as the owner changes.
 */
export function createScopedDriveUploaderStateStore(
  storage: SessionStorageLike | undefined,
  resolveScope?: () => DriveStorageScope | undefined,
): DriveUploaderStateStore {
  let memory: Record<string, DriveUploaderStateSnapshot> = {};
  let memoryOwnerKey: string | null = null;
  let legacyUnscopedKeyDropped = false;

  const resolveOwner = (): { ownerKey: string; storageKey: string | null } => {
    const scope = resolveScope?.();
    return {
      ownerKey: driveStorageScopeKey(scope),
      storageKey: scope && storage
        ? scopeDriveStorageKey(DRIVE_UPLOADER_STATE_STORAGE_KEY, scope)
        : null,
    };
  };

  const syncOwner = (ownerKey: string): void => {
    if (memoryOwnerKey === ownerKey) {
      return;
    }
    // A different account (or sign-out) must not resume the previous owner's
    // in-memory sessions either.
    memoryOwnerKey = ownerKey;
    memory = {};
  };

  /** Older builds wrote one unscoped record for every account; it has no owner. */
  const dropLegacyUnscopedState = (): void => {
    if (legacyUnscopedKeyDropped || !storage) {
      return;
    }
    legacyUnscopedKeyDropped = true;
    try {
      storage.removeItem(DRIVE_UPLOADER_STATE_STORAGE_KEY);
    } catch {
      // Best-effort cleanup; a failed removal only leaves unread legacy bytes.
    }
  };

  const readAll = (storageKey: string | null): Record<string, DriveUploaderStateSnapshot> => {
    if (!storageKey || !storage) {
      return memory;
    }
    try {
      const raw = storage.getItem(storageKey);
      if (!raw) {
        return {};
      }
      const parsed = JSON.parse(raw) as Record<string, unknown>;
      const snapshots: Record<string, DriveUploaderStateSnapshot> = {};
      for (const [taskId, snapshot] of Object.entries(parsed)) {
        if (isUploaderStateSnapshot(snapshot) && snapshot.taskId === taskId) {
          snapshots[taskId] = snapshot;
        }
      }
      return snapshots;
    } catch {
      return {};
    }
  };

  const writeAll = (
    storageKey: string | null,
    snapshots: Record<string, DriveUploaderStateSnapshot>,
  ): void => {
    if (!storageKey || !storage) {
      return;
    }
    try {
      storage.setItem(storageKey, JSON.stringify(snapshots));
    } catch {
      // Ignore storage persistence failures and keep uploads running in-memory.
    }
  };

  const cloneSnapshot = (snapshot: DriveUploaderStateSnapshot): DriveUploaderStateSnapshot => ({
    ...snapshot,
    uploadedParts: snapshot.uploadedParts.map((part) => ({ ...part })),
  });

  return {
    async get(taskId) {
      const { ownerKey, storageKey } = resolveOwner();
      syncOwner(ownerKey);
      if (storageKey) {
        dropLegacyUnscopedState();
      }
      return readAll(storageKey)[taskId];
    },
    async put(snapshot) {
      const { ownerKey, storageKey } = resolveOwner();
      syncOwner(ownerKey);
      if (storageKey) {
        dropLegacyUnscopedState();
      }
      const snapshots = readAll(storageKey);
      snapshots[snapshot.taskId] = cloneSnapshot(snapshot);
      writeAll(storageKey, snapshots);
    },
    async clear(taskId) {
      const { ownerKey, storageKey } = resolveOwner();
      syncOwner(ownerKey);
      if (storageKey) {
        dropLegacyUnscopedState();
      }
      const snapshots = readAll(storageKey);
      if (taskId in snapshots) {
        delete snapshots[taskId];
        writeAll(storageKey, snapshots);
      }
    },
  };
}

export class DriveAppSdkError extends Error {
  readonly operationId: DriveAppOperationId;
  readonly status: number;
  readonly title?: string;
  readonly detail?: string;
  readonly code?: number;
  readonly traceId?: string;

  constructor({
    operationId,
    status,
    title,
    detail,
    code,
    traceId,
  }: {
    operationId: DriveAppOperationId;
    status: number;
    title?: string;
    detail?: string;
    code?: number;
    traceId?: string;
  }) {
    super(detail || title || `Drive App API ${operationId} failed with HTTP ${status}`);
    this.name = 'DriveAppSdkError';
    this.operationId = operationId;
    this.status = status;
    this.title = title;
    this.detail = detail;
    this.code = code;
    this.traceId = traceId;
  }
}

function buildSdkError(
  operationId: DriveAppOperationId,
  error: unknown,
): DriveAppSdkError {
  const details = normalizeGeneratedSdkError(error);
  return new DriveAppSdkError({
    operationId,
    status: details.status,
    title: details.title,
    detail: details.detail,
    code: details.code,
    traceId: details.traceId,
  });
}

/**
 * Normalizes a single-resource response to the resource itself.
 *
 * The generated transport strips only the `SdkWorkApiResponse.data` envelope,
 * so a single-resource operation (`spaces.create`, `nodes.retrieve`,
 * `uploadSessions.complete`, ...) resolves to `{ item: T }`. The typed
 * generated methods pass `sdkworkUnwrapKind: 'item'` for exactly those
 * operations; this generic dispatch cannot read that per-operation metadata, so
 * it applies the same unwrapping by shape. List (`{ items, pageInfo }`) and
 * command (`{ node, uploadSession }`, `{ downloadUrl }`, ...) payloads never
 * carry a sole `item` key and pass through unchanged.
 */
function unwrapResourceItem<T>(payload: unknown): T {
  if (
    typeof payload !== 'object'
    || payload === null
    || Array.isArray(payload)
    || !('item' in payload)
    || Object.keys(payload).length !== 1
  ) {
    return payload as T;
  }
  return (payload as { item: T }).item;
}

export function createDriveAppSdkClient({
  config,
  sdkClient,
  tokenManager,
  uploaderStateStorage,
  resolveUploaderStorageScope,
}: DriveAppSdkClientOptions): DriveAppSdkClient {
  const generatedClient = sdkClient ?? createGeneratedDriveAppClient({
    authMode: 'dual-token',
    baseUrl: normalizeGeneratedSdkBaseUrl(config.appApiBaseUrl, sdkMetadata.apiPrefix),
    tokenManager,
  }) as TokenManagerAwareGeneratedSdkClient;
  generatedClient.setTokenManager(tokenManager);

  const client = {
    metadata: {
      ...sdkMetadata,
      baseUrl: config.appApiBaseUrl,
    },
    operations,
    uploader: undefined as unknown as DriveUploaderClient,
    async request<T>({
      operationId,
      pathParams,
      query,
      body,
      signal,
    }: DriveAppSdkRequest): Promise<T> {
      const operation = operations[operationId];
      try {
        const payload = await generatedClient.http.request<unknown>(
          buildGeneratedSdkPath(operation.path, pathParams),
          {
            method: operation.method,
            params: compactQuery(assertStandardSdkWorkPaginationQuery(omitAuthProjectionQuery(query))),
            body: omitAuthProjectionBody(body),
            contentType: body === undefined ? undefined : 'application/json',
            signal,
          },
        );
        return unwrapResourceItem<T>(payload);
      } catch (error) {
        if (isDriveRequestCancellationError(error)) {
          throw error;
        }
        throw buildSdkError(operationId, error);
      }
    },
    setTokenManager(manager: DriveSessionTokenManager) {
      generatedClient.setTokenManager(manager);
    },
  };
  client.uploader = createDriveUploaderClient({
    transport: createDriveUploaderTransport(client),
    stateStore: createScopedDriveUploaderStateStore(
      uploaderStateStorage ?? resolveDefaultUploaderStateStorage(config.auth.tokenStorage),
      resolveUploaderStorageScope,
    ),
  });
  return client;
}

export function createDriveUploaderTransport(
  client: Pick<DriveAppSdkClient, 'request'>,
): DriveUploaderTransport {
  return {
    drive: {
      uploader: {
        uploads: {
          create: (body, options) =>
            client.request({
              operationId: 'uploader.uploads.create',
              body,
              signal: options?.signal,
            }),
          parts: {
            update: (uploadItemId, partNo, body, options) =>
              client.request({
                operationId: 'uploader.uploads.parts.update',
                pathParams: {
                  uploadItemId,
                  partNo,
                },
                body,
                signal: options?.signal,
              }),
          },
        },
      },
      uploadSessions: {
        create: (body, options) =>
          client.request({
            operationId: 'uploadSessions.create',
            body,
            signal: options?.signal,
          }),
        parts: {
          update: (uploadSessionId, partNo, body, options) =>
            client.request({
              operationId: 'uploadSessions.parts.update',
              pathParams: {
                uploadSessionId,
                partNo,
              },
              body,
              signal: options?.signal,
            }),
        },
        complete: (uploadSessionId, body, options) =>
          client.request({
            operationId: 'uploadSessions.complete',
            pathParams: {
              uploadSessionId,
            },
            body,
            signal: options?.signal,
          }),
        abort: (uploadSessionId, body, options) =>
          client.request({
            operationId: 'uploadSessions.abort',
            pathParams: {
              uploadSessionId,
            },
            body,
            signal: options?.signal,
          }),
      },
    },
  };
}

function resolveDefaultUploaderStateStorage(
  tokenStorage: DriveRuntimeConfig['auth']['tokenStorage'],
): SessionStorageLike | undefined {
  if (typeof window === 'undefined') {
    return undefined;
  }
  if (tokenStorage === 'browser-local') {
    return window.localStorage;
  }
  if (tokenStorage === 'browser-session') {
    return window.sessionStorage;
  }
  return undefined;
}
