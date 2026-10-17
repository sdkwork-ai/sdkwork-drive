import { customApiPath } from './paths';
import type { ApiRequestOptions, HttpClient } from '../http/client';

import type { CreateRootScopeSubscriptionRequest, DriveResourceResolution, DriveResourceScopeType, EnsureRootScopeEventDeliveryRequest, EnsureWebsiteRootEventDeliveryRequest, NormalizedRelativePath, PositiveInt64String, ResolveDriveResourceRequest, RootScopeEventDelivery, RootScopeSubscription, WebsiteRoot, WebsiteRootEventDelivery } from '../types';


export interface DriveInternalPublishingDriveResourceContentRetrieveParams {
  scopeType: DriveResourceScopeType;
  scopeUuid: string;
  relativePath: NormalizedRelativePath;
  pinnedGeneration?: PositiveInt64String;
  range?: string;
  ifMatch?: string;
  ifNoneMatch?: string;
  ifRange?: string;
  ifModifiedSince?: string;
  ifUnmodifiedSince?: string;
}

export class DriveInternalPublishingDriveResourceContentApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Stream root-qualified immutable Drive content */
  async retrieve(nodeVersionId: string, params: DriveInternalPublishingDriveResourceContentRetrieveParams, requestOptions?: ApiRequestOptions): Promise<Blob> {
    const query = buildQueryString([
      { name: 'scopeType', value: params.scopeType, style: 'form', explode: true, allowReserved: false },
      { name: 'scopeUuid', value: params.scopeUuid, style: 'form', explode: true, allowReserved: false },
      { name: 'relativePath', value: params.relativePath, style: 'form', explode: true, allowReserved: false },
      { name: 'pinnedGeneration', value: params.pinnedGeneration, style: 'form', explode: true, allowReserved: false },
    ]);
    const requestHeaders = buildRequestHeaders(
      {
        Range: { value: params.range, style: 'simple', explode: false },
        'If-Match': { value: params.ifMatch, style: 'simple', explode: false },
        'If-None-Match': { value: params.ifNoneMatch, style: 'simple', explode: false },
        'If-Range': { value: params.ifRange, style: 'simple', explode: false },
        'If-Modified-Since': { value: params.ifModifiedSince, style: 'simple', explode: false },
        'If-Unmodified-Since': { value: params.ifUnmodifiedSince, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.request<Blob>(appendQueryString(customApiPath(`/drive/node_versions/${serializePathParameter(nodeVersionId, { name: 'nodeVersionId', style: 'simple', explode: false })}/content`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, ...(requestHeaders !== undefined ? { headers: requestHeaders } : {}), sdkworkUnwrapKind: 'data' });
  }
}

export class DriveInternalPublishingDriveResourcesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Resolve a root-qualified Drive resource */
  async resolve(body: ResolveDriveResourceRequest, requestOptions?: ApiRequestOptions): Promise<DriveResourceResolution> {
    return this.client.request<DriveResourceResolution>(customApiPath(`/drive/resource_resolutions`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class DriveInternalPublishingWebsiteRootEventDeliveriesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Ensure a signed WebsiteRoot event delivery channel */
  async replace(websiteRootUuid: string, channelId: string, body: EnsureWebsiteRootEventDeliveryRequest, requestOptions?: ApiRequestOptions): Promise<WebsiteRootEventDelivery> {
    return this.client.request<WebsiteRootEventDelivery>(customApiPath(`/drive/website_roots/${serializePathParameter(websiteRootUuid, { name: 'websiteRootUuid', style: 'simple', explode: false })}/event_deliveries/${serializePathParameter(channelId, { name: 'channelId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PUT' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class DriveInternalPublishingWebsiteRootsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Retrieve a website delivery root */
  async retrieve(websiteRootUuid: string, requestOptions?: ApiRequestOptions): Promise<WebsiteRoot> {
    return this.client.request<WebsiteRoot>(customApiPath(`/drive/website_roots/${serializePathParameter(websiteRootUuid, { name: 'websiteRootUuid', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }
}

export class DriveInternalPublishingRootScopeEventDeliveriesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Ensure the signed Knowledgebase event delivery channel for a root scope */
  async replace(subscriptionUuid: string, body: EnsureRootScopeEventDeliveryRequest, requestOptions?: ApiRequestOptions): Promise<RootScopeEventDelivery> {
    return this.client.request<RootScopeEventDelivery>(customApiPath(`/drive/root_scope_subscriptions/${serializePathParameter(subscriptionUuid, { name: 'subscriptionUuid', style: 'simple', explode: false })}/event_delivery`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PUT' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class DriveInternalPublishingRootScopeSubscriptionsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Register a Knowledgebase sources/raw root scope */
  async create(body: CreateRootScopeSubscriptionRequest, requestOptions?: ApiRequestOptions): Promise<RootScopeSubscription> {
    return this.client.request<RootScopeSubscription>(customApiPath(`/drive/root_scope_subscriptions`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Retrieve a root scope subscription */
  async retrieve(subscriptionUuid: string, requestOptions?: ApiRequestOptions): Promise<RootScopeSubscription> {
    return this.client.request<RootScopeSubscription>(customApiPath(`/drive/root_scope_subscriptions/${serializePathParameter(subscriptionUuid, { name: 'subscriptionUuid', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }
}

export class DriveInternalPublishingApi {
  public readonly rootScopeSubscriptions: DriveInternalPublishingRootScopeSubscriptionsApi;
  public readonly rootScopeEventDeliveries: DriveInternalPublishingRootScopeEventDeliveriesApi;
  public readonly websiteRoots: DriveInternalPublishingWebsiteRootsApi;
  public readonly websiteRootEventDeliveries: DriveInternalPublishingWebsiteRootEventDeliveriesApi;
  public readonly driveResources: DriveInternalPublishingDriveResourcesApi;
  public readonly driveResourceContent: DriveInternalPublishingDriveResourceContentApi;

  constructor(client: HttpClient) {
    this.rootScopeSubscriptions = new DriveInternalPublishingRootScopeSubscriptionsApi(client);
    this.rootScopeEventDeliveries = new DriveInternalPublishingRootScopeEventDeliveriesApi(client);
    this.websiteRoots = new DriveInternalPublishingWebsiteRootsApi(client);
    this.websiteRootEventDeliveries = new DriveInternalPublishingWebsiteRootEventDeliveriesApi(client);
    this.driveResources = new DriveInternalPublishingDriveResourcesApi(client);
    this.driveResourceContent = new DriveInternalPublishingDriveResourceContentApi(client);
  }

}

export function createDriveInternalPublishingApi(client: HttpClient): DriveInternalPublishingApi {
  return new DriveInternalPublishingApi(client);
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
function buildRequestHeaders(
  headers: Record<string, HeaderParameterSpec | undefined>,
  cookies: Record<string, HeaderParameterSpec | undefined> = {},
): Record<string, string> | undefined {
  const requestHeaders: Record<string, string> = {};

  for (const [name, parameter] of Object.entries(headers)) {
    const serialized = serializeParameterValue(parameter);
    if (serialized !== undefined) {
      requestHeaders[name] = serialized;
    }
  }

  const cookieHeader = buildCookieHeader(cookies);
  if (cookieHeader) {
    requestHeaders.Cookie = requestHeaders.Cookie
      ? `${requestHeaders.Cookie}; ${cookieHeader}`
      : cookieHeader;
  }

  return Object.keys(requestHeaders).length > 0 ? requestHeaders : undefined;
}

interface HeaderParameterSpec {
  value: unknown;
  style: string;
  explode: boolean;
  contentType?: string;
}

function buildCookieHeader(cookies: Record<string, HeaderParameterSpec | undefined>): string | undefined {
  const pairs: string[] = [];
  for (const [name, parameter] of Object.entries(cookies)) {
    const serialized = serializeParameterValue(parameter);
    if (serialized !== undefined) {
      pairs.push(`${encodeURIComponent(name)}=${encodeURIComponent(serialized)}`);
    }
  }
  return pairs.length > 0 ? pairs.join('; ') : undefined;
}

function serializeParameterValue(parameter: HeaderParameterSpec | undefined): string | undefined {
  const value = parameter?.value;
  if (value === undefined || value === null) {
    return undefined;
  }
  if (parameter?.contentType) {
    return JSON.stringify(value);
  }
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (Array.isArray(value)) {
    return value.map((item) => serializeHeaderPrimitive(item)).join(',');
  }
  if (typeof value === 'object' && value !== null) {
    return serializeHeaderObject(value as Record<string, unknown>, parameter?.explode === true);
  }
  return serializeHeaderPrimitive(value);
}

function serializeHeaderObject(value: Record<string, unknown>, explode: boolean): string {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
  if (explode) {
    return entries.map(([key, entryValue]) => `${key}=${serializeHeaderPrimitive(entryValue)}`).join(',');
  }
  return entries.flatMap(([key, entryValue]) => [key, serializeHeaderPrimitive(entryValue)]).join(',');
}

function serializeHeaderPrimitive(value: unknown): string {
  if (value instanceof Date) {
    return value.toISOString();
  }
  return String(value);
}
