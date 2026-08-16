import {
  createGeneratedDriveAdminStorageClient,
  operations,
  sdkMetadata,
} from '@sdkwork/drive-admin-storage-sdk';
import type { DriveRuntimeConfig, DriveSessionTokenManager } from 'sdkwork-drive-pc-core';
import {
  assertStandardSdkWorkPaginationQuery,
  buildGeneratedSdkPath,
  compactQuery,
  isDriveRequestCancellationError,
  normalizeGeneratedSdkBaseUrl,
  normalizeGeneratedSdkError,
  omitAuthProjectionBody,
  omitAuthProjectionQuery,
  type TokenManagerAwareGeneratedSdkClient,
} from 'sdkwork-drive-pc-core';

export type DriveAdminStorageOperationId = keyof typeof operations;

export interface DriveAdminStorageSdkRequest {
  operationId: DriveAdminStorageOperationId;
  pathParams?: Record<string, string | number>;
  query?: Record<string, string | number | boolean | undefined>;
  body?: unknown;
  signal?: AbortSignal;
}

export interface DriveAdminStorageSdkClient {
  metadata: typeof sdkMetadata;
  operations: typeof operations;
  request<T>(request: DriveAdminStorageSdkRequest): Promise<T>;
  setTokenManager(manager: DriveSessionTokenManager): void;
}

export interface DriveAdminStorageSdkClientOptions {
  config: DriveRuntimeConfig;
  sdkClient?: TokenManagerAwareGeneratedSdkClient;
  tokenManager: DriveSessionTokenManager;
}

export class DriveAdminStorageSdkError extends Error {
  readonly operationId: DriveAdminStorageOperationId;
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
    operationId: DriveAdminStorageOperationId;
    status: number;
    title?: string;
    detail?: string;
    code?: number;
    traceId?: string;
  }) {
    super(detail || title || `Drive Admin Storage API ${operationId} failed with HTTP ${status}`);
    this.name = 'DriveAdminStorageSdkError';
    this.operationId = operationId;
    this.status = status;
    this.title = title;
    this.detail = detail;
    this.code = code;
    this.traceId = traceId;
  }
}

function buildSdkError(
  operationId: DriveAdminStorageOperationId,
  error: unknown,
): DriveAdminStorageSdkError {
  const details = normalizeGeneratedSdkError(error);
  return new DriveAdminStorageSdkError({
    operationId,
    status: details.status,
    title: details.title,
    detail: details.detail,
    code: details.code,
    traceId: details.traceId,
  });
}

export function createDriveAdminStorageSdkClient({
  config,
  sdkClient,
  tokenManager,
}: DriveAdminStorageSdkClientOptions): DriveAdminStorageSdkClient {
  const generatedClient = sdkClient ?? createGeneratedDriveAdminStorageClient({
    authMode: 'dual-token',
    baseUrl: normalizeGeneratedSdkBaseUrl(
      config.adminStorageApiBaseUrl,
      sdkMetadata.apiPrefix,
    ),
    tokenManager,
  }) as TokenManagerAwareGeneratedSdkClient;
  generatedClient.setTokenManager(tokenManager);

  return {
    metadata: {
      ...sdkMetadata,
      baseUrl: config.adminStorageApiBaseUrl,
    },
    operations,
    async request<T>({
      operationId,
      pathParams,
      query,
      body,
      signal,
    }: DriveAdminStorageSdkRequest): Promise<T> {
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
}
