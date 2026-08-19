import { backendApiPath } from './paths';
import type { ApiRequestOptions, HttpClient } from '../http/client';

import type { CreateLabelRequest, CreateSandboxGrantRequest, CreateSandboxVolumeRequest, DriveLabel, SandboxLifecycleStatus, SandboxProviderKind, SweepObjectStoreRequest, SweepUploadSessionsRequest, UpdateLabelRequest, UpdateQuotaPolicyRequest, UpdateSandboxGrantRequest, UpdateSandboxVolumeRequest } from '../types';


export interface DriveSandboxGrantsListParams {
  page?: string;
  pageSize?: string;
}

export class DriveSandboxGrantsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List explicit sandbox grants */
  async list(sandboxId: string, params?: DriveSandboxGrantsListParams, requestOptions?: ApiRequestOptions): Promise<unknown> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<unknown>(appendQueryString(backendApiPath(`/drive/sandbox_volumes/${serializePathParameter(sandboxId, { name: 'sandboxId', style: 'simple', explode: false })}/grants`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'data' });
  }

/** Create an explicit sandbox grant */
  async create(sandboxId: string, body: CreateSandboxGrantRequest, requestOptions?: ApiRequestOptions): Promise<unknown> {
    return this.client.request<unknown>(backendApiPath(`/drive/sandbox_volumes/${serializePathParameter(sandboxId, { name: 'sandboxId', style: 'simple', explode: false })}/grants`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }

/** Retrieve a sandbox grant */
  async retrieve(sandboxId: string, grantId: string, requestOptions?: ApiRequestOptions): Promise<unknown> {
    return this.client.request<unknown>(backendApiPath(`/drive/sandbox_volumes/${serializePathParameter(sandboxId, { name: 'sandboxId', style: 'simple', explode: false })}/grants/${serializePathParameter(grantId, { name: 'grantId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'data' });
  }

/** Update a sandbox grant */
  async update(sandboxId: string, grantId: string, body: UpdateSandboxGrantRequest, requestOptions?: ApiRequestOptions): Promise<unknown> {
    return this.client.request<unknown>(backendApiPath(`/drive/sandbox_volumes/${serializePathParameter(sandboxId, { name: 'sandboxId', style: 'simple', explode: false })}/grants/${serializePathParameter(grantId, { name: 'grantId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PATCH' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }

/** Delete a sandbox grant */
  async delete(sandboxId: string, grantId: string, requestOptions?: ApiRequestOptions): Promise<void> {
    return this.client.request<void>(backendApiPath(`/drive/sandbox_volumes/${serializePathParameter(sandboxId, { name: 'sandboxId', style: 'simple', explode: false })}/grants/${serializePathParameter(grantId, { name: 'grantId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'DELETE' as any });
  }
}

export interface DriveSandboxVolumesListParams {
  lifecycleStatus?: SandboxLifecycleStatus;
  providerKind?: SandboxProviderKind;
  page?: string;
  pageSize?: string;
}

export class DriveSandboxVolumesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List server sandbox volumes */
  async list(params?: DriveSandboxVolumesListParams, requestOptions?: ApiRequestOptions): Promise<unknown> {
    const query = buildQueryString([
      { name: 'lifecycle_status', value: params?.lifecycleStatus, style: 'form', explode: true, allowReserved: false },
      { name: 'provider_kind', value: params?.providerKind, style: 'form', explode: true, allowReserved: false },
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<unknown>(appendQueryString(backendApiPath(`/drive/sandbox_volumes`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'data' });
  }

/** Create a server sandbox volume */
  async create(body: CreateSandboxVolumeRequest, requestOptions?: ApiRequestOptions): Promise<unknown> {
    return this.client.request<unknown>(backendApiPath(`/drive/sandbox_volumes`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }

/** Retrieve a server sandbox volume */
  async retrieve(sandboxId: string, requestOptions?: ApiRequestOptions): Promise<unknown> {
    return this.client.request<unknown>(backendApiPath(`/drive/sandbox_volumes/${serializePathParameter(sandboxId, { name: 'sandboxId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'data' });
  }

/** Update a server sandbox volume */
  async update(sandboxId: string, body: UpdateSandboxVolumeRequest, requestOptions?: ApiRequestOptions): Promise<unknown> {
    return this.client.request<unknown>(backendApiPath(`/drive/sandbox_volumes/${serializePathParameter(sandboxId, { name: 'sandboxId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PATCH' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }

/** Delete a server sandbox volume */
  async delete(sandboxId: string, requestOptions?: ApiRequestOptions): Promise<void> {
    return this.client.request<void>(backendApiPath(`/drive/sandbox_volumes/${serializePathParameter(sandboxId, { name: 'sandboxId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'DELETE' as any });
  }
}

export interface DriveDownloadPackagesListParams {
  state?: 'creating' | 'ready' | 'failed' | 'expired';
  page?: number;
  pageSize?: number;
}

export class DriveDownloadPackagesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: DriveDownloadPackagesListParams, requestOptions?: ApiRequestOptions): Promise<unknown> {
    const query = buildQueryString([
      { name: 'state', value: params?.state, style: 'form', explode: true, allowReserved: false },
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<unknown>(appendQueryString(backendApiPath(`/drive/download_packages`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'data' });
  }
}

export interface DriveSpacesAdminListParams {
  ownerSubjectType?: string;
  ownerSubjectId?: string;
  pageSize?: number;
  cursor?: string;
}

export class DriveSpacesAdminApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: DriveSpacesAdminListParams, requestOptions?: ApiRequestOptions): Promise<unknown> {
    const query = buildQueryString([
      { name: 'ownerSubjectType', value: params?.ownerSubjectType, style: 'form', explode: true, allowReserved: false },
      { name: 'ownerSubjectId', value: params?.ownerSubjectId, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<unknown>(appendQueryString(backendApiPath(`/drive/spaces`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'data' });
  }
}

export class DriveSpacesApi {
  public readonly admin: DriveSpacesAdminApi;

  constructor(client: HttpClient) {
    this.admin = new DriveSpacesAdminApi(client);
  }

}

export class DriveQuotasApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(requestOptions?: ApiRequestOptions): Promise<unknown> {
    return this.client.request<unknown>(backendApiPath(`/drive/quotas`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'data' });
  }

/** Update tenant quota policy */
  async update(body: UpdateQuotaPolicyRequest, requestOptions?: ApiRequestOptions): Promise<unknown> {
    return this.client.request<unknown>(backendApiPath(`/drive/quotas`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PUT' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }
}

export interface DriveMaintenanceJobsListParams {
  jobType?: 'object_sweep' | 'upload_session_sweep' | 'expired_upload_content_sweep' | 'abandoned_upload_task_sweep';
  status?: 'completed' | 'failed';
  operatorId?: string;
  page?: number;
  pageSize?: number;
}

export class DriveMaintenanceJobsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: DriveMaintenanceJobsListParams, requestOptions?: ApiRequestOptions): Promise<unknown> {
    const query = buildQueryString([
      { name: 'jobType', value: params?.jobType, style: 'form', explode: true, allowReserved: false },
      { name: 'status', value: params?.status, style: 'form', explode: true, allowReserved: false },
      { name: 'operatorId', value: params?.operatorId, style: 'form', explode: true, allowReserved: false },
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<unknown>(appendQueryString(backendApiPath(`/drive/maintenance/jobs`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'data' });
  }
}

export class DriveMaintenanceApi {
  private client: HttpClient;
  public readonly jobs: DriveMaintenanceJobsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.jobs = new DriveMaintenanceJobsApi(client);
  }


async objectSweep(body: SweepObjectStoreRequest, requestOptions?: ApiRequestOptions): Promise<unknown> {
    return this.client.request<unknown>(backendApiPath(`/drive/maintenance/object_sweep`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }

async uploadSessionSweep(body: SweepUploadSessionsRequest, requestOptions?: ApiRequestOptions): Promise<unknown> {
    return this.client.request<unknown>(backendApiPath(`/drive/maintenance/upload_session_sweep`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }

async expiredUploadContentSweep(body: SweepUploadSessionsRequest, requestOptions?: ApiRequestOptions): Promise<unknown> {
    return this.client.request<unknown>(backendApiPath(`/drive/maintenance/expired_upload_content_sweep`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }

async abandonedUploadTaskSweep(body: SweepUploadSessionsRequest, requestOptions?: ApiRequestOptions): Promise<unknown> {
    return this.client.request<unknown>(backendApiPath(`/drive/maintenance/abandoned_upload_task_sweep`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }
}

export interface DriveLabelsListParams {
  lifecycleStatus?: 'active' | 'deleted';
  pageSize?: number;
  cursor?: string;
}

export class DriveLabelsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List Drive label definitions */
  async list(params?: DriveLabelsListParams, requestOptions?: ApiRequestOptions): Promise<unknown> {
    const query = buildQueryString([
      { name: 'lifecycleStatus', value: params?.lifecycleStatus, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<unknown>(appendQueryString(backendApiPath(`/drive/labels`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'data' });
  }

/** Create a Drive label definition */
  async create(body: CreateLabelRequest, requestOptions?: ApiRequestOptions): Promise<DriveLabel> {
    return this.client.request<DriveLabel>(backendApiPath(`/drive/labels`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Get a Drive label definition */
  async retrieve(labelId: string, requestOptions?: ApiRequestOptions): Promise<unknown> {
    return this.client.request<unknown>(backendApiPath(`/drive/labels/${serializePathParameter(labelId, { name: 'labelId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'data' });
  }

/** Update a Drive label definition */
  async update(labelId: string, body: UpdateLabelRequest, requestOptions?: ApiRequestOptions): Promise<unknown> {
    return this.client.request<unknown>(backendApiPath(`/drive/labels/${serializePathParameter(labelId, { name: 'labelId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PATCH' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }

/** Delete a Drive label definition */
  async delete(labelId: string, requestOptions?: ApiRequestOptions): Promise<void> {
    return this.client.request<void>(backendApiPath(`/drive/labels/${serializePathParameter(labelId, { name: 'labelId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'DELETE' as any });
  }
}

export interface DriveAuditEventsListParams {
  action?: string;
  resourceType?: string;
  resourceId?: string;
  correlationId?: string;
  traceId?: string;
  page?: number;
  pageSize?: number;
}

export class DriveAuditEventsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: DriveAuditEventsListParams, requestOptions?: ApiRequestOptions): Promise<unknown> {
    const query = buildQueryString([
      { name: 'action', value: params?.action, style: 'form', explode: true, allowReserved: false },
      { name: 'resourceType', value: params?.resourceType, style: 'form', explode: true, allowReserved: false },
      { name: 'resourceId', value: params?.resourceId, style: 'form', explode: true, allowReserved: false },
      { name: 'correlationId', value: params?.correlationId, style: 'form', explode: true, allowReserved: false },
      { name: 'traceId', value: params?.traceId, style: 'form', explode: true, allowReserved: false },
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<unknown>(appendQueryString(backendApiPath(`/drive/audit_events`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'data' });
  }
}

export class DriveApi {
  public readonly auditEvents: DriveAuditEventsApi;
  public readonly labels: DriveLabelsApi;
  public readonly maintenance: DriveMaintenanceApi;
  public readonly quotas: DriveQuotasApi;
  public readonly spaces: DriveSpacesApi;
  public readonly downloadPackages: DriveDownloadPackagesApi;
  public readonly sandboxVolumes: DriveSandboxVolumesApi;
  public readonly sandboxGrants: DriveSandboxGrantsApi;

  constructor(client: HttpClient) {
    this.auditEvents = new DriveAuditEventsApi(client);
    this.labels = new DriveLabelsApi(client);
    this.maintenance = new DriveMaintenanceApi(client);
    this.quotas = new DriveQuotasApi(client);
    this.spaces = new DriveSpacesApi(client);
    this.downloadPackages = new DriveDownloadPackagesApi(client);
    this.sandboxVolumes = new DriveSandboxVolumesApi(client);
    this.sandboxGrants = new DriveSandboxGrantsApi(client);
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
