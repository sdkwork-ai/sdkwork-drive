import {
  createInMemoryUploaderStateStore,
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

function createPersistentDriveUploaderStateStore(
  storage?: SessionStorageLike,
): DriveUploaderStateStore {
  if (!storage) {
    return createInMemoryUploaderStateStore();
  }

  const readAll = (): Record<string, DriveUploaderStateSnapshot> => {
    try {
      const raw = storage.getItem(DRIVE_UPLOADER_STATE_STORAGE_KEY);
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

  const writeAll = (snapshots: Record<string, DriveUploaderStateSnapshot>): void => {
    try {
      storage.setItem(DRIVE_UPLOADER_STATE_STORAGE_KEY, JSON.stringify(snapshots));
    } catch {
      // Ignore storage persistence failures and keep uploads running in-memory.
    }
  };

  return {
    async get(taskId) {
      return readAll()[taskId];
    },
    async put(snapshot) {
      const snapshots = readAll();
      snapshots[snapshot.taskId] = {
        ...snapshot,
        uploadedParts: snapshot.uploadedParts.map((part) => ({ ...part })),
      };
      writeAll(snapshots);
    },
    async clear(taskId) {
      const snapshots = readAll();
      if (taskId in snapshots) {
        delete snapshots[taskId];
        writeAll(snapshots);
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

export function createDriveAppSdkClient({
  config,
  sdkClient,
  tokenManager,
  uploaderStateStorage,
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
        return await generatedClient.http.request<T>(
          buildGeneratedSdkPath(operation.path, pathParams),
          {
            method: operation.method,
            params: compactQuery(assertStandardSdkWorkPaginationQuery(omitAuthProjectionQuery(query))),
            body: omitAuthProjectionBody(body),
            contentType: body === undefined ? undefined : 'application/json',
            signal,
          },
        );
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
    stateStore: createPersistentDriveUploaderStateStore(
      uploaderStateStorage ?? resolveDefaultUploaderStateStorage(config.auth.tokenStorage),
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
