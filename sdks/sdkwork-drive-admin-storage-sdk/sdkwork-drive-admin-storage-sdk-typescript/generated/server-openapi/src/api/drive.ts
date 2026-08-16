import { customApiPath } from './paths';
import type { ApiRequestOptions, HttpClient } from '../http/client';

import type { CopyProviderObjectRequest, CreateStorageProviderRequest, RotateStorageProviderCredentialRequest, SetDefaultStorageProviderBindingRequest, SetStorageProviderKindEnabledRequest, StorageProviderBindingsDefaultRetrieveResponse, StorageProviderBindingsDefaultUpdateResponse, StorageProviderBindingsListResponse, StorageProviderKindsInitializeResponse, StorageProviderKindsListResponse, StorageProviderKindsUpdateResponse, StorageProvidersActivateResponse, StorageProvidersBucketRetrieveResponse, StorageProvidersBucketsListResponse, StorageProvidersBucketUpdateResponse, StorageProvidersCapabilitiesListResponse, StorageProvidersCreateResponse201, StorageProvidersCredentialsRotateResponse, StorageProvidersDeactivateResponse, StorageProvidersListResponse, StorageProvidersObjectsContentRetrieveResponse, StorageProvidersObjectsContentUpdateResponse, StorageProvidersObjectsCopyResponse, StorageProvidersObjectsListResponse, StorageProvidersObjectsRetrieveResponse, StorageProvidersRetrieveResponse, StorageProvidersTestResponse, StorageProvidersUpdateResponse, UpdateProviderObjectContent, UpdateStorageProviderRequest } from '../types';


export class DriveStorageProviderKindsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(requestOptions?: ApiRequestOptions): Promise<StorageProviderKindsListResponse> {
    return this.client.request<StorageProviderKindsListResponse>(customApiPath(`/drive/storage/provider-kinds`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'GET' as any });
  }

async initialize(requestOptions?: ApiRequestOptions): Promise<StorageProviderKindsInitializeResponse> {
    return this.client.request<StorageProviderKindsInitializeResponse>(customApiPath(`/drive/storage/provider-kinds`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'POST' as any });
  }

async update(providerKind: string, body: SetStorageProviderKindEnabledRequest, requestOptions?: ApiRequestOptions): Promise<StorageProviderKindsUpdateResponse> {
    return this.client.request<StorageProviderKindsUpdateResponse>(customApiPath(`/drive/storage/provider-kinds/${serializePathParameter(providerKind, { name: 'providerKind', style: 'simple', explode: false })}`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'PATCH' as any, body, contentType: 'application/json' });
  }
}

export class DriveStorageProvidersObjectsContentApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Retrieve provider object content */
  async retrieve(providerId: string, objectKey: string, requestOptions?: ApiRequestOptions): Promise<StorageProvidersObjectsContentRetrieveResponse> {
    return this.client.request<StorageProvidersObjectsContentRetrieveResponse>(customApiPath(`/drive/storage/providers/${serializePathParameter(providerId, { name: 'providerId', style: 'simple', explode: false })}/object-contents/${serializePathParameter(objectKey, { name: 'objectKey', style: 'simple', explode: false })}`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'GET' as any });
  }

/** Write provider object content */
  async update(providerId: string, objectKey: string, body: UpdateProviderObjectContent, requestOptions?: ApiRequestOptions): Promise<StorageProvidersObjectsContentUpdateResponse> {
    return this.client.request<StorageProvidersObjectsContentUpdateResponse>(customApiPath(`/drive/storage/providers/${serializePathParameter(providerId, { name: 'providerId', style: 'simple', explode: false })}/object-contents/${serializePathParameter(objectKey, { name: 'objectKey', style: 'simple', explode: false })}`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'PUT' as any, body, contentType: 'application/json' });
  }
}

export interface DriveStorageProvidersObjectsListParams {
  prefix?: string;
  delimiter?: string;
  cursor?: string;
  pageSize?: number;
}

export class DriveStorageProvidersObjectsApi {
  private client: HttpClient;
  public readonly content: DriveStorageProvidersObjectsContentApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.content = new DriveStorageProvidersObjectsContentApi(client);
  }


async list(providerId: string, params?: DriveStorageProvidersObjectsListParams, requestOptions?: ApiRequestOptions): Promise<StorageProvidersObjectsListResponse> {
    const query = buildQueryString([
      { name: 'prefix', value: params?.prefix, style: 'form', explode: true, allowReserved: false },
      { name: 'delimiter', value: params?.delimiter, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<StorageProvidersObjectsListResponse>(appendQueryString(customApiPath(`/drive/storage/providers/${serializePathParameter(providerId, { name: 'providerId', style: 'simple', explode: false })}/objects`), query), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'GET' as any });
  }

async retrieve(providerId: string, objectKey: string, requestOptions?: ApiRequestOptions): Promise<StorageProvidersObjectsRetrieveResponse> {
    return this.client.request<StorageProvidersObjectsRetrieveResponse>(customApiPath(`/drive/storage/providers/${serializePathParameter(providerId, { name: 'providerId', style: 'simple', explode: false })}/objects/${serializePathParameter(objectKey, { name: 'objectKey', style: 'simple', explode: false })}`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'GET' as any });
  }

async delete(providerId: string, objectKey: string, requestOptions?: ApiRequestOptions): Promise<void> {
    return this.client.request<void>(customApiPath(`/drive/storage/providers/${serializePathParameter(providerId, { name: 'providerId', style: 'simple', explode: false })}/objects/${serializePathParameter(objectKey, { name: 'objectKey', style: 'simple', explode: false })}`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'DELETE' as any });
  }

async copy(providerId: string, body: CopyProviderObjectRequest, requestOptions?: ApiRequestOptions): Promise<StorageProvidersObjectsCopyResponse> {
    return this.client.request<StorageProvidersObjectsCopyResponse>(customApiPath(`/drive/storage/providers/${serializePathParameter(providerId, { name: 'providerId', style: 'simple', explode: false })}/objects/copy`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'POST' as any, body, contentType: 'application/json' });
  }
}

export interface DriveStorageProvidersBucketListParams {
  cursor?: string;
  pageSize?: number;
}

export class DriveStorageProvidersBucketApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(providerId: string, requestOptions?: ApiRequestOptions): Promise<StorageProvidersBucketRetrieveResponse> {
    return this.client.request<StorageProvidersBucketRetrieveResponse>(customApiPath(`/drive/storage/providers/${serializePathParameter(providerId, { name: 'providerId', style: 'simple', explode: false })}/bucket`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'GET' as any });
  }

async update(providerId: string, requestOptions?: ApiRequestOptions): Promise<StorageProvidersBucketUpdateResponse> {
    return this.client.request<StorageProvidersBucketUpdateResponse>(customApiPath(`/drive/storage/providers/${serializePathParameter(providerId, { name: 'providerId', style: 'simple', explode: false })}/bucket`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'PUT' as any });
  }

async delete(providerId: string, requestOptions?: ApiRequestOptions): Promise<void> {
    return this.client.request<void>(customApiPath(`/drive/storage/providers/${serializePathParameter(providerId, { name: 'providerId', style: 'simple', explode: false })}/bucket`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'DELETE' as any });
  }

/** List buckets visible to a Drive storage provider account */
  async list(providerId: string, params?: DriveStorageProvidersBucketListParams, requestOptions?: ApiRequestOptions): Promise<StorageProvidersBucketsListResponse> {
    const query = buildQueryString([
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<StorageProvidersBucketsListResponse>(appendQueryString(customApiPath(`/drive/storage/providers/${serializePathParameter(providerId, { name: 'providerId', style: 'simple', explode: false })}/buckets`), query), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'GET' as any });
  }
}

export class DriveStorageProvidersCredentialsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async rotate(providerId: string, body: RotateStorageProviderCredentialRequest, requestOptions?: ApiRequestOptions): Promise<StorageProvidersCredentialsRotateResponse> {
    return this.client.request<StorageProvidersCredentialsRotateResponse>(customApiPath(`/drive/storage/providers/${serializePathParameter(providerId, { name: 'providerId', style: 'simple', explode: false })}/credentials/rotate`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'POST' as any, body, contentType: 'application/json' });
  }
}

export class DriveStorageProvidersCapabilitiesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(providerId: string, requestOptions?: ApiRequestOptions): Promise<StorageProvidersCapabilitiesListResponse> {
    return this.client.request<StorageProvidersCapabilitiesListResponse>(customApiPath(`/drive/storage/providers/${serializePathParameter(providerId, { name: 'providerId', style: 'simple', explode: false })}/capabilities`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'GET' as any });
  }
}

export interface DriveStorageProvidersListParams {
  status?: string;
}

export class DriveStorageProvidersApi {
  private client: HttpClient;
  public readonly capabilities: DriveStorageProvidersCapabilitiesApi;
  public readonly credentials: DriveStorageProvidersCredentialsApi;
  public readonly bucket: DriveStorageProvidersBucketApi;
  public readonly objects: DriveStorageProvidersObjectsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.capabilities = new DriveStorageProvidersCapabilitiesApi(client);
    this.credentials = new DriveStorageProvidersCredentialsApi(client);
    this.bucket = new DriveStorageProvidersBucketApi(client);
    this.objects = new DriveStorageProvidersObjectsApi(client);
  }


async list(params?: DriveStorageProvidersListParams, requestOptions?: ApiRequestOptions): Promise<StorageProvidersListResponse> {
    const query = buildQueryString([
      { name: 'status', value: params?.status, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<StorageProvidersListResponse>(appendQueryString(customApiPath(`/drive/storage/providers`), query), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'GET' as any });
  }

async create(body: CreateStorageProviderRequest, requestOptions?: ApiRequestOptions): Promise<StorageProvidersCreateResponse201> {
    return this.client.request<StorageProvidersCreateResponse201>(customApiPath(`/drive/storage/providers`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'POST' as any, body, contentType: 'application/json' });
  }

async update(providerId: string, body: UpdateStorageProviderRequest, requestOptions?: ApiRequestOptions): Promise<StorageProvidersUpdateResponse> {
    return this.client.request<StorageProvidersUpdateResponse>(customApiPath(`/drive/storage/providers/${serializePathParameter(providerId, { name: 'providerId', style: 'simple', explode: false })}`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'PATCH' as any, body, contentType: 'application/json' });
  }

async delete(providerId: string, requestOptions?: ApiRequestOptions): Promise<void> {
    return this.client.request<void>(customApiPath(`/drive/storage/providers/${serializePathParameter(providerId, { name: 'providerId', style: 'simple', explode: false })}`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'DELETE' as any });
  }

async retrieve(providerId: string, requestOptions?: ApiRequestOptions): Promise<StorageProvidersRetrieveResponse> {
    return this.client.request<StorageProvidersRetrieveResponse>(customApiPath(`/drive/storage/providers/${serializePathParameter(providerId, { name: 'providerId', style: 'simple', explode: false })}`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'GET' as any });
  }

async activate(providerId: string, requestOptions?: ApiRequestOptions): Promise<StorageProvidersActivateResponse> {
    return this.client.request<StorageProvidersActivateResponse>(customApiPath(`/drive/storage/providers/${serializePathParameter(providerId, { name: 'providerId', style: 'simple', explode: false })}/activate`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'POST' as any });
  }

async deactivate(providerId: string, requestOptions?: ApiRequestOptions): Promise<StorageProvidersDeactivateResponse> {
    return this.client.request<StorageProvidersDeactivateResponse>(customApiPath(`/drive/storage/providers/${serializePathParameter(providerId, { name: 'providerId', style: 'simple', explode: false })}/deactivate`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'POST' as any });
  }

async test(providerId: string, requestOptions?: ApiRequestOptions): Promise<StorageProvidersTestResponse> {
    return this.client.request<StorageProvidersTestResponse>(customApiPath(`/drive/storage/providers/${serializePathParameter(providerId, { name: 'providerId', style: 'simple', explode: false })}/test`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'POST' as any });
  }
}

export interface DriveStorageProviderBindingsDefaultRetrieveParams {
  spaceId?: string;
  spaceType?: 'personal' | 'team' | 'knowledge_base' | 'ai_generated' | 'git_repository' | 'deployment' | 'app_upload' | 'im' | 'rtc' | 'notary' | 'website';
}

export interface DriveStorageProviderBindingsDefaultDeleteParams {
  spaceId?: string;
  spaceType?: 'personal' | 'team' | 'knowledge_base' | 'ai_generated' | 'git_repository' | 'deployment' | 'app_upload' | 'im' | 'rtc' | 'notary' | 'website';
}

export class DriveStorageProviderBindingsDefaultApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(params?: DriveStorageProviderBindingsDefaultRetrieveParams, requestOptions?: ApiRequestOptions): Promise<StorageProviderBindingsDefaultRetrieveResponse> {
    const query = buildQueryString([
      { name: 'spaceId', value: params?.spaceId, style: 'form', explode: true, allowReserved: false },
      { name: 'spaceType', value: params?.spaceType, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<StorageProviderBindingsDefaultRetrieveResponse>(appendQueryString(customApiPath(`/drive/storage/bindings/default`), query), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'GET' as any });
  }

async update(body: SetDefaultStorageProviderBindingRequest, requestOptions?: ApiRequestOptions): Promise<StorageProviderBindingsDefaultUpdateResponse> {
    return this.client.request<StorageProviderBindingsDefaultUpdateResponse>(customApiPath(`/drive/storage/bindings/default`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'PUT' as any, body, contentType: 'application/json' });
  }

/** Delete a Drive default storage provider binding */
  async delete(params?: DriveStorageProviderBindingsDefaultDeleteParams, requestOptions?: ApiRequestOptions): Promise<void> {
    const query = buildQueryString([
      { name: 'spaceId', value: params?.spaceId, style: 'form', explode: true, allowReserved: false },
      { name: 'spaceType', value: params?.spaceType, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<void>(appendQueryString(customApiPath(`/drive/storage/bindings/default`), query), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'DELETE' as any });
  }
}

export interface DriveStorageProviderBindingsListParams {
  spaceId?: string;
  providerId?: string;
  lifecycleStatus?: 'active' | 'disabled' | 'deleted';
}

export class DriveStorageProviderBindingsApi {
  private client: HttpClient;
  public readonly default: DriveStorageProviderBindingsDefaultApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.default = new DriveStorageProviderBindingsDefaultApi(client);
  }


/** List Drive storage provider bindings */
  async list(params?: DriveStorageProviderBindingsListParams, requestOptions?: ApiRequestOptions): Promise<StorageProviderBindingsListResponse> {
    const query = buildQueryString([
      { name: 'spaceId', value: params?.spaceId, style: 'form', explode: true, allowReserved: false },
      { name: 'providerId', value: params?.providerId, style: 'form', explode: true, allowReserved: false },
      { name: 'lifecycleStatus', value: params?.lifecycleStatus, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<StorageProviderBindingsListResponse>(appendQueryString(customApiPath(`/drive/storage/bindings`), query), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'GET' as any });
  }
}

export class DriveApi {
  private client: HttpClient;
  public readonly storageProviderBindings: DriveStorageProviderBindingsApi;
  public readonly storageProviders: DriveStorageProvidersApi;
  public readonly storageProviderKinds: DriveStorageProviderKindsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.storageProviderBindings = new DriveStorageProviderBindingsApi(client);
    this.storageProviders = new DriveStorageProvidersApi(client);
    this.storageProviderKinds = new DriveStorageProviderKindsApi(client);
  }

}

export function createDriveApi(client: HttpClient): DriveApi {
  return new DriveApi(client);
}

function appendQueryString(path: string, rawQueryString: string): string {
  const query = rawQueryString.replace(/^\?+/, '');
  if (!query) {
    return path;
  }
  return path.includes('?') ? `${path}&${query}` : `${path}?${query}`;
}

interface PathParameterSpec {
  name: string;
  style: string;
  explode: boolean;
}

function serializePathParameter(value: unknown, spec: PathParameterSpec): string {
  if (value === undefined || value === null) {
    return '';
  }

  const style = spec.style || 'simple';
  if (Array.isArray(value)) {
    return serializePathArray(spec.name, value, style, spec.explode);
  }
  if (typeof value === 'object') {
    return serializePathObject(spec.name, value as Record<string, unknown>, style, spec.explode);
  }
  return pathPrefix(spec.name, style, false) + encodePathValue(serializePathPrimitive(value));
}

function serializePathArray(name: string, values: unknown[], style: string, explode: boolean): string {
  const serialized = values
    .filter((item) => item !== undefined && item !== null)
    .map((item) => encodePathValue(serializePathPrimitive(item)));
  if (serialized.length === 0) {
    return pathPrefix(name, style, false);
  }
  if (style === 'matrix') {
    return explode
      ? serialized.map((item) => `;${name}=${item}`).join('')
      : `;${name}=${serialized.join(',')}`;
  }
  return pathPrefix(name, style, false) + serialized.join(explode ? '.' : ',');
}

function serializePathObject(name: string, value: Record<string, unknown>, style: string, explode: boolean): string {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
  if (entries.length === 0) {
    return pathPrefix(name, style, true);
  }
  if (style === 'matrix') {
    return explode
      ? entries.map(([key, entryValue]) => `;${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join('')
      : `;${name}=${entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',')}`;
  }
  const serialized = explode
    ? entries.map(([key, entryValue]) => `${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join(style === 'label' ? '.' : ',')
    : entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',');
  return pathPrefix(name, style, true) + serialized;
}

function pathPrefix(name: string, style: string, _objectValue: boolean): string {
  if (style === 'label') return '.';
  if (style === 'matrix') return `;${name}`;
  return '';
}

function encodePathValue(value: string): string {
  return encodeURIComponent(value);
}

function serializePathPrimitive(value: unknown): string {
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  return String(value);
}
interface QueryParameterSpec {
  name: string;
  value: unknown;
  style: string;
  explode: boolean;
  allowReserved: boolean;
  contentType?: string;
}

function buildQueryString(parameters: QueryParameterSpec[]): string {
  const pairs: string[] = [];
  for (const parameter of parameters) {
    appendSerializedParameter(pairs, parameter);
  }
  return pairs.join('&');
}

function appendSerializedParameter(pairs: string[], parameter: QueryParameterSpec): void {
  if (parameter.value === undefined || parameter.value === null) {
    return;
  }

  if (parameter.contentType) {
    pairs.push(`${encodeQueryComponent(parameter.name)}=${encodeQueryValue(JSON.stringify(parameter.value), parameter.allowReserved)}`);
    return;
  }

  const style = parameter.style || 'form';
  if (style === 'deepObject') {
    appendDeepObjectParameter(pairs, parameter.name, parameter.value, parameter.allowReserved);
    return;
  }

  if (Array.isArray(parameter.value)) {
    appendArrayParameter(pairs, parameter.name, parameter.value, style, parameter.explode, parameter.allowReserved);
    return;
  }

  if (typeof parameter.value === 'object') {
    appendObjectParameter(pairs, parameter.name, parameter.value as Record<string, unknown>, style, parameter.explode, parameter.allowReserved);
    return;
  }

  pairs.push(`${encodeQueryComponent(parameter.name)}=${encodeQueryValue(serializePrimitive(parameter.value), parameter.allowReserved)}`);
}

function appendArrayParameter(
  pairs: string[],
  name: string,
  value: unknown[],
  style: string,
  explode: boolean,
  allowReserved: boolean,
): void {
  const values = value
    .filter((item) => item !== undefined && item !== null)
    .map((item) => serializePrimitive(item));
  if (values.length === 0) {
    return;
  }

  if (style === 'form' && explode) {
    for (const item of values) {
      pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(item, allowReserved)}`);
    }
    return;
  }

  pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(values.join(','), allowReserved)}`);
}

function appendObjectParameter(
  pairs: string[],
  name: string,
  value: Record<string, unknown>,
  style: string,
  explode: boolean,
  allowReserved: boolean,
): void {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
  if (entries.length === 0) {
    return;
  }

  if (style === 'form' && explode) {
    for (const [key, entryValue] of entries) {
      pairs.push(`${encodeQueryComponent(key)}=${encodeQueryValue(serializePrimitive(entryValue), allowReserved)}`);
    }
    return;
  }

  const serialized = entries.flatMap(([key, entryValue]) => [key, serializePrimitive(entryValue)]).join(',');
  pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(serialized, allowReserved)}`);
}

function appendDeepObjectParameter(
  pairs: string[],
  name: string,
  value: unknown,
  allowReserved: boolean,
): void {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(serializePrimitive(value), allowReserved)}`);
    return;
  }

  for (const [key, entryValue] of Object.entries(value as Record<string, unknown>)) {
    if (entryValue === undefined || entryValue === null) {
      continue;
    }
    pairs.push(`${encodeQueryComponent(`${name}[${key}]`)}=${encodeQueryValue(serializePrimitive(entryValue), allowReserved)}`);
  }
}

function serializePrimitive(value: unknown): string {
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  return String(value);
}

function encodeQueryComponent(value: string): string {
  return encodeURIComponent(value);
}

function encodeQueryValue(value: string, allowReserved: boolean): string {
  const encoded = encodeURIComponent(value);
  if (!allowReserved) {
    return encoded;
  }
  return encoded.replace(/%3A/gi, ':')
    .replace(/%2F/gi, '/')
    .replace(/%3F/gi, '?')
    .replace(/%23/gi, '#')
    .replace(/%5B/gi, '[')
    .replace(/%5D/gi, ']')
    .replace(/%40/gi, '@')
    .replace(/%21/gi, '!')
    .replace(/%24/gi, '$')
    .replace(/%26/gi, '&')
    .replace(/%27/gi, "'")
    .replace(/%28/gi, '(')
    .replace(/%29/gi, ')')
    .replace(/%2A/gi, '*')
    .replace(/%2B/gi, '+')
    .replace(/%2C/gi, ',')
    .replace(/%3B/gi, ';')
    .replace(/%3D/gi, '=');
}
