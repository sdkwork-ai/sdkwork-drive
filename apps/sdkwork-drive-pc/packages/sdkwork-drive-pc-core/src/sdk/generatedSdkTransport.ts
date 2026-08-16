import type { DriveSessionTokenManager } from '../session/sessionTokenManager';

export interface GeneratedSdkHttpClient {
  request<T>(
    path: string,
    options?: GeneratedSdkRequestOptions,
  ): Promise<T>;
}

export interface GeneratedSdkRequestOptions {
  method?: string;
  params?: Record<string, string | number | boolean | undefined>;
  body?: unknown;
  contentType?: string;
  signal?: AbortSignal;
}

export interface TokenManagerAwareGeneratedSdkClient {
  http: GeneratedSdkHttpClient;
  setTokenManager(manager: DriveSessionTokenManager): unknown;
}

export interface GeneratedSdkOperation {
  method: string;
  path: string;
}

export interface GeneratedSdkErrorDetails {
  status: number;
  title?: string;
  detail?: string;
  code?: number;
  traceId?: string;
}

export function normalizeGeneratedSdkBaseUrl(
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

export function buildGeneratedSdkPath(
  pathTemplate: string,
  params: Record<string, string | number> = {},
): string {
  return pathTemplate.replace(/\{([^}]+)\}/g, (_, key: string) => {
    const value = params[key];
    if (value === undefined || value === null) {
      throw new Error(`Missing SDK path parameter: ${key}`);
    }
    return encodeURIComponent(String(value));
  });
}

export function compactQuery(
  query: GeneratedSdkRequestOptions['params'] = {},
): Record<string, string | number | boolean> | undefined {
  const compact = Object.fromEntries(
    Object.entries(query).filter(([, value]) => value !== undefined),
  ) as Record<string, string | number | boolean>;
  return Object.keys(compact).length > 0 ? compact : undefined;
}

const LEGACY_SDKWORK_PAGINATION_QUERY_KEYS = [
  'pageSize',
  'pageToken',
  'page_token',
  'limit',
  'page_no',
  'pageNo',
  'per_page',
  'size',
];

export function assertStandardSdkWorkPaginationQuery(
  query?: GeneratedSdkRequestOptions['params'],
): GeneratedSdkRequestOptions['params'] | undefined {
  if (!query) {
    return undefined;
  }
  const legacyKey = LEGACY_SDKWORK_PAGINATION_QUERY_KEYS.find(
    (key) => Object.prototype.hasOwnProperty.call(query, key) && query[key] !== undefined,
  );
  if (legacyKey) {
    throw new Error(
      `Legacy SDKWork pagination query parameter "${legacyKey}" is not allowed. Use "page_size" and "cursor".`,
    );
  }
  return query;
}

export function normalizeGeneratedSdkError(error: unknown): GeneratedSdkErrorDetails {
  if (!error || typeof error !== 'object') {
    return {
      status: 0,
      detail: String(error),
    };
  }

  const record = error as Record<string, unknown>;
  const message = typeof record.message === 'string' ? record.message : undefined;
  const name = typeof record.name === 'string' ? record.name : undefined;

  return {
    status: numberValue(record.httpStatus) ?? numberValue(record.status) ?? 0,
    title: stringValue(record.title) ?? name,
    detail: stringValue(record.detail) ?? message,
    code: numberValue(record.businessCode) ?? numberValue(record.code),
    traceId: stringValue(record.traceId) ?? stringValue(record.trace_id),
  };
}

export function isDriveRequestCancellationError(error: unknown): boolean {
  if (!error || typeof error !== 'object') {
    return false;
  }

  const record = error as Record<string, unknown>;
  return record.name === 'AbortError'
    || record.name === 'CancelledError'
    || record.code === 'CANCELLED';
}

function stringValue(value: unknown): string | undefined {
  return typeof value === 'string' && value.trim() !== '' ? value : undefined;
}

function numberValue(value: unknown): number | undefined {
  return typeof value === 'number' && Number.isFinite(value) ? value : undefined;
}
