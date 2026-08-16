import {
  createGeneratedDriveBackendClient,
  operations,
  sdkMetadata,
} from '@sdkwork/drive-backend-sdk';
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

export type DriveBackendOperationId = keyof typeof operations;

export interface DriveBackendSdkRequest {
  operationId: DriveBackendOperationId;
  pathParams?: Record<string, string | number>;
  query?: Record<string, string | number | boolean | undefined>;
  body?: unknown;
  signal?: AbortSignal;
}

export interface DriveBackendSdkClient {
  metadata: typeof sdkMetadata;
  operations: typeof operations;
  request<T>(request: DriveBackendSdkRequest): Promise<T>;
  setTokenManager(manager: DriveSessionTokenManager): void;
}

export interface DriveBackendSdkClientOptions {
  config: DriveRuntimeConfig;
  sdkClient?: TokenManagerAwareGeneratedSdkClient;
  tokenManager: DriveSessionTokenManager;
}

export class DriveBackendSdkError extends Error {
  readonly operationId: DriveBackendOperationId;
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
    operationId: DriveBackendOperationId;
    status: number;
    title?: string;
    detail?: string;
    code?: number;
    traceId?: string;
  }) {
    super(detail || title || `Drive Backend API ${operationId} failed with HTTP ${status}`);
    this.name = 'DriveBackendSdkError';
    this.operationId = operationId;
    this.status = status;
    this.title = title;
    this.detail = detail;
    this.code = code;
    this.traceId = traceId;
  }
}

function buildSdkError(
  operationId: DriveBackendOperationId,
  error: unknown,
): DriveBackendSdkError {
  const details = normalizeGeneratedSdkError(error);
  return new DriveBackendSdkError({
    operationId,
    status: details.status,
    title: details.title,
    detail: details.detail,
    code: details.code,
    traceId: details.traceId,
  });
}

export function createDriveBackendSdkClient({
  config,
  sdkClient,
  tokenManager,
}: DriveBackendSdkClientOptions): DriveBackendSdkClient {
  const generatedClient = sdkClient ?? createGeneratedDriveBackendClient({
    authMode: 'dual-token',
    baseUrl: normalizeGeneratedSdkBaseUrl(
      config.backendApiBaseUrl,
      sdkMetadata.apiPrefix,
    ),
    tokenManager,
  }) as TokenManagerAwareGeneratedSdkClient;
  generatedClient.setTokenManager(tokenManager);

  return {
    metadata: {
      ...sdkMetadata,
      baseUrl: config.backendApiBaseUrl,
    },
    operations,
    async request<T>({
      operationId,
      pathParams,
      query,
      body,
      signal,
    }: DriveBackendSdkRequest): Promise<T> {
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
