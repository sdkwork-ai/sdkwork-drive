import { appApiPath } from './paths';
import type { ApiRequestOptions, HttpClient } from '../http/client';

import type { ActivateWebsiteGenerationRequest, ApplyNodeLabelRequest, ArchiveEntry, ChangeListData, CheckFavoriteNodesRequest, ClaimShareLinkResponse, CompleteUploadSessionRequest, CopyNodeRequest, CreateCommentReplyRequest, CreateCommentRequest, CreateDownloadGrantRequest, CreateDownloadPackageRequest, CreateDownloadUrlRequest, CreateDownloadUrlResponse, CreateDriveSandboxDirectoryRequest, CreateDriveSandboxFileRequest, CreateFileRequest, CreateFileResponse, CreateFolderRequest, CreatePermissionRequest, CreateShareLinkRequest, CreateShareLinkResponse, CreateShortcutRequest, CreateSpaceRequest, CreateUploadSessionRequest, CreateWatchChannelRequest, CreateWebsiteRootRequest, CreateWebsiteSyncRequest, DownloadPackageResponse, DriveComment, DriveCommentReply, DriveNode, DriveNodeListData, DriveNodeProperty, DrivePermission, DriveSandboxEntry, DriveSandboxEntryListData, DriveSandboxFileContent, DriveSandboxMutationCommandData, DriveSandboxVolumeListData, DriveShareLink, DriveSpace, DriveUploadSession, DriveWatchChannel, DriveWatchChannelListData, EffectivePermission, EmptyTrashRequest, EmptyTrashResponse, ExtractArchiveEntriesRequest, ExtractArchiveEntriesResponse, FavoriteNodeRequest, FavoriteNodeResponse, FileVersion, FileVersionListData, MarkUploaderPartUploadedRequest, MoveNodeRequest, NodeCapabilitiesResponse, NodeCommandRequest, NodeLabel, NodePathResponse, PageInfo, PositiveInt64String, PrepareUploaderUploadRequest, PrepareUploaderUploadResponse, PresignedUploadPart, PresignUploadPartRequest, PurgeDriveSandboxEntryRequest, QuotaSummary, SetNodePropertyRequest, StartPageTokenResponse, StopWatchChannelRequest, StopWatchChannelResponse, UpdateCommentReplyRequest, UpdateCommentRequest, UpdateDriveSandboxEntryRequest, UpdateDriveSandboxFileContentRequest, UpdateNodeRequest, UpdatePermissionRequest, UpdateShareLinkRequest, UpdateSpaceRequest, UploaderUploadPart, WebsiteGenerationActivation, WebsiteRoot, WebsiteRootPageData, WebsiteSync, WebsiteSyncActivation, WebsiteSyncVersionRequest } from '../types';


export class DriveUploaderUploadsPartsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async update(uploadItemId: string, partNo: number, body: MarkUploaderPartUploadedRequest, requestOptions?: ApiRequestOptions): Promise<UploaderUploadPart> {
    return this.client.request<UploaderUploadPart>(appApiPath(`/drive/uploader/uploads/${serializePathParameter(uploadItemId, { name: 'uploadItemId', style: 'simple', explode: false })}/parts/${serializePathParameter(partNo, { name: 'partNo', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PUT' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class DriveUploaderUploadsApi {
  private client: HttpClient;
  public readonly parts: DriveUploaderUploadsPartsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.parts = new DriveUploaderUploadsPartsApi(client);
  }


async create(body: PrepareUploaderUploadRequest, requestOptions?: ApiRequestOptions): Promise<PrepareUploaderUploadResponse> {
    return this.client.request<PrepareUploaderUploadResponse>(appApiPath(`/drive/uploader/uploads`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }
}

export class DriveUploaderApi {
  public readonly uploads: DriveUploaderUploadsApi;

  constructor(client: HttpClient) {
    this.uploads = new DriveUploaderUploadsApi(client);
  }

}

export class DriveArchiveEntriesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(nodeId: string, requestOptions?: ApiRequestOptions): Promise<{ items: ArchiveEntry[]; pageInfo: PageInfo; }> {
    return this.client.request<{ items: ArchiveEntry[]; pageInfo: PageInfo; }>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/archive_entries`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

async extract(nodeId: string, body: ExtractArchiveEntriesRequest, requestOptions?: ApiRequestOptions): Promise<ExtractArchiveEntriesResponse> {
    return this.client.request<ExtractArchiveEntriesResponse>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/archive_entries/extract`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }
}

export class DriveDownloadPackagesDownloadUrlsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(packageId: string, requestOptions?: ApiRequestOptions): Promise<DownloadPackageResponse> {
    return this.client.request<DownloadPackageResponse>(appApiPath(`/drive/download_packages/${serializePathParameter(packageId, { name: 'packageId', style: 'simple', explode: false })}/download_url`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'data' });
  }
}

export class DriveDownloadPackagesApi {
  private client: HttpClient;
  public readonly downloadUrls: DriveDownloadPackagesDownloadUrlsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.downloadUrls = new DriveDownloadPackagesDownloadUrlsApi(client);
  }


async create(body: CreateDownloadPackageRequest, requestOptions?: ApiRequestOptions): Promise<DownloadPackageResponse> {
    return this.client.request<DownloadPackageResponse>(appApiPath(`/drive/download_packages`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }
}

export interface DriveWatchChannelsListParams {
  resourceType?: 'changes' | 'node';
  lifecycleStatus?: 'active' | 'stopped' | 'expired';
  pageSize?: number;
  cursor?: string;
}

export class DriveWatchChannelsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List Drive watch channels */
  async list(params?: DriveWatchChannelsListParams, requestOptions?: ApiRequestOptions): Promise<DriveWatchChannelListData> {
    const query = buildQueryString([
      { name: 'resourceType', value: params?.resourceType, style: 'form', explode: true, allowReserved: false },
      { name: 'lifecycleStatus', value: params?.lifecycleStatus, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<DriveWatchChannelListData>(appendQueryString(appApiPath(`/drive/watch_channels`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Get a Drive watch channel */
  async retrieve(channelId: string, requestOptions?: ApiRequestOptions): Promise<DriveWatchChannel> {
    return this.client.request<DriveWatchChannel>(appApiPath(`/drive/watch_channels/${serializePathParameter(channelId, { name: 'channelId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Stop a Drive watch channel */
  async stop(channelId: string, body: StopWatchChannelRequest, requestOptions?: ApiRequestOptions): Promise<StopWatchChannelResponse> {
    return this.client.request<StopWatchChannelResponse>(appApiPath(`/drive/watch_channels/${serializePathParameter(channelId, { name: 'channelId', style: 'simple', explode: false })}/stop`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }
}

export class DriveUploadSessionsPartsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async update(uploadSessionId: string, partNo: number, body: PresignUploadPartRequest, requestOptions?: ApiRequestOptions): Promise<PresignedUploadPart> {
    return this.client.request<PresignedUploadPart>(appApiPath(`/drive/upload_sessions/${serializePathParameter(uploadSessionId, { name: 'uploadSessionId', style: 'simple', explode: false })}/parts/${serializePathParameter(partNo, { name: 'partNo', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PUT' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }
}

export class DriveUploadSessionsApi {
  private client: HttpClient;
  public readonly parts: DriveUploadSessionsPartsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.parts = new DriveUploadSessionsPartsApi(client);
  }


async create(body: CreateUploadSessionRequest, requestOptions?: ApiRequestOptions): Promise<DriveUploadSession> {
    return this.client.request<DriveUploadSession>(appApiPath(`/drive/upload_sessions`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

async retrieve(uploadSessionId: string, requestOptions?: ApiRequestOptions): Promise<DriveUploadSession> {
    return this.client.request<DriveUploadSession>(appApiPath(`/drive/upload_sessions/${serializePathParameter(uploadSessionId, { name: 'uploadSessionId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

async abort(uploadSessionId: string, body: NodeCommandRequest, requestOptions?: ApiRequestOptions): Promise<DriveUploadSession> {
    return this.client.request<DriveUploadSession>(appApiPath(`/drive/upload_sessions/${serializePathParameter(uploadSessionId, { name: 'uploadSessionId', style: 'simple', explode: false })}/abort`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

async complete(uploadSessionId: string, body: CompleteUploadSessionRequest, requestOptions?: ApiRequestOptions): Promise<DriveUploadSession> {
    return this.client.request<DriveUploadSession>(appApiPath(`/drive/upload_sessions/${serializePathParameter(uploadSessionId, { name: 'uploadSessionId', style: 'simple', explode: false })}/complete`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export interface DriveMoveDestinationsListParams {
  excludeNodeIds?: string;
  pageSize?: string;
  cursor?: string;
}

export class DriveMoveDestinationsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(spaceId: string, params?: DriveMoveDestinationsListParams, requestOptions?: ApiRequestOptions): Promise<DriveNodeListData> {
    const query = buildQueryString([
      { name: 'excludeNodeIds', value: params?.excludeNodeIds, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<DriveNodeListData>(appendQueryString(appApiPath(`/drive/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}/move_destinations`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }
}

export class DriveWebsiteRootsGenerationsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Activate a retained website generation as a new logical generation */
  async activate(rootUuid: string, generation: PositiveInt64String, body: ActivateWebsiteGenerationRequest, requestOptions?: ApiRequestOptions): Promise<WebsiteGenerationActivation> {
    return this.client.request<WebsiteGenerationActivation>(appApiPath(`/drive/website_roots/${serializePathParameter(rootUuid, { name: 'rootUuid', style: 'simple', explode: false })}/generations/${serializePathParameter(generation, { name: 'generation', style: 'simple', explode: false })}/activate`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export interface DriveWebsiteRootsSyncsCreateParams {
  idempotencyKey: string;
}

export class DriveWebsiteRootsSyncsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create an isolated atomic website synchronization */
  async create(rootUuid: string, body: CreateWebsiteSyncRequest, params: DriveWebsiteRootsSyncsCreateParams, requestOptions?: ApiRequestOptions): Promise<WebsiteSync> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.request<WebsiteSync>(appApiPath(`/drive/website_roots/${serializePathParameter(rootUuid, { name: 'rootUuid', style: 'simple', explode: false })}/syncs`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', ...(requestHeaders !== undefined ? { headers: requestHeaders } : {}), sdkworkUnwrapKind: 'item' });
  }

/** Retrieve an atomic website synchronization */
  async retrieve(rootUuid: string, syncId: string, requestOptions?: ApiRequestOptions): Promise<WebsiteSync> {
    return this.client.request<WebsiteSync>(appApiPath(`/drive/website_roots/${serializePathParameter(rootUuid, { name: 'rootUuid', style: 'simple', explode: false })}/syncs/${serializePathParameter(syncId, { name: 'syncId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Validate and atomically activate a complete website tree */
  async finalize(rootUuid: string, syncId: string, body: WebsiteSyncVersionRequest, requestOptions?: ApiRequestOptions): Promise<WebsiteSyncActivation> {
    return this.client.request<WebsiteSyncActivation>(appApiPath(`/drive/website_roots/${serializePathParameter(rootUuid, { name: 'rootUuid', style: 'simple', explode: false })}/syncs/${serializePathParameter(syncId, { name: 'syncId', style: 'simple', explode: false })}/finalize`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Abort an unactivated website synchronization */
  async abort(rootUuid: string, syncId: string, body: WebsiteSyncVersionRequest, requestOptions?: ApiRequestOptions): Promise<WebsiteSync> {
    return this.client.request<WebsiteSync>(appApiPath(`/drive/website_roots/${serializePathParameter(rootUuid, { name: 'rootUuid', style: 'simple', explode: false })}/syncs/${serializePathParameter(syncId, { name: 'syncId', style: 'simple', explode: false })}/abort`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export interface DriveWebsiteRootsListParams {
  pageSize?: number;
  cursor?: string;
}

export class DriveWebsiteRootsApi {
  private client: HttpClient;
  public readonly syncs: DriveWebsiteRootsSyncsApi;
  public readonly generations: DriveWebsiteRootsGenerationsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.syncs = new DriveWebsiteRootsSyncsApi(client);
    this.generations = new DriveWebsiteRootsGenerationsApi(client);
  }


async list(spaceId: string, params?: DriveWebsiteRootsListParams, requestOptions?: ApiRequestOptions): Promise<WebsiteRootPageData> {
    const query = buildQueryString([
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<WebsiteRootPageData>(appendQueryString(appApiPath(`/drive/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}/website_roots`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

async create(spaceId: string, body: CreateWebsiteRootRequest, requestOptions?: ApiRequestOptions): Promise<WebsiteRoot> {
    return this.client.request<WebsiteRoot>(appApiPath(`/drive/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}/website_roots`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

async retrieve(rootUuid: string, requestOptions?: ApiRequestOptions): Promise<WebsiteRoot> {
    return this.client.request<WebsiteRoot>(appApiPath(`/drive/website_roots/${serializePathParameter(rootUuid, { name: 'rootUuid', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }
}

export interface DriveSpacesListParams {
  ownerSubjectType?: string;
  ownerSubjectId?: string;
  spaceType?: 'personal' | 'team' | 'knowledge_base' | 'ai_generated' | 'git_repository' | 'deployment' | 'app_upload' | 'im' | 'rtc' | 'notary' | 'website';
  pageSize?: number;
  cursor?: string;
}

export class DriveSpacesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: DriveSpacesListParams, requestOptions?: ApiRequestOptions): Promise<{ items: DriveSpace[]; pageInfo: PageInfo; }> {
    const query = buildQueryString([
      { name: 'ownerSubjectType', value: params?.ownerSubjectType, style: 'form', explode: true, allowReserved: false },
      { name: 'ownerSubjectId', value: params?.ownerSubjectId, style: 'form', explode: true, allowReserved: false },
      { name: 'spaceType', value: params?.spaceType, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<{ items: DriveSpace[]; pageInfo: PageInfo; }>(appendQueryString(appApiPath(`/drive/spaces`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

async create(body: CreateSpaceRequest, requestOptions?: ApiRequestOptions): Promise<DriveSpace> {
    return this.client.request<DriveSpace>(appApiPath(`/drive/spaces`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

async retrieve(spaceId: string, requestOptions?: ApiRequestOptions): Promise<DriveSpace> {
    return this.client.request<DriveSpace>(appApiPath(`/drive/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

async update(spaceId: string, body: UpdateSpaceRequest, requestOptions?: ApiRequestOptions): Promise<DriveSpace> {
    return this.client.request<DriveSpace>(appApiPath(`/drive/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PATCH' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

async delete(spaceId: string, requestOptions?: ApiRequestOptions): Promise<void> {
    return this.client.request<void>(appApiPath(`/drive/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'DELETE' as any });
  }
}

export interface DriveSandboxFileContentsRetrieveParams {
  logicalPath: string;
  encoding?: 'utf8' | 'base64';
}

export interface DriveSandboxFileContentsUpdateParams {
  ifMatch: string;
  idempotencyKey: string;
}

export class DriveSandboxFileContentsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(sandboxId: string, entryId: string, params: DriveSandboxFileContentsRetrieveParams, requestOptions?: ApiRequestOptions): Promise<DriveSandboxFileContent> {
    const query = buildQueryString([
      { name: 'logical_path', value: params.logicalPath, style: 'form', explode: true, allowReserved: false },
      { name: 'encoding', value: params.encoding, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<DriveSandboxFileContent>(appendQueryString(appApiPath(`/drive/sandboxes/${serializePathParameter(sandboxId, { name: 'sandboxId', style: 'simple', explode: false })}/files/${serializePathParameter(entryId, { name: 'entryId', style: 'simple', explode: false })}/content`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

async update(sandboxId: string, entryId: string, body: UpdateDriveSandboxFileContentRequest, params: DriveSandboxFileContentsUpdateParams, requestOptions?: ApiRequestOptions): Promise<DriveSandboxEntry> {
    const requestHeaders = buildRequestHeaders(
      {
        'If-Match': { value: params.ifMatch, style: 'simple', explode: false },
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.request<DriveSandboxEntry>(appApiPath(`/drive/sandboxes/${serializePathParameter(sandboxId, { name: 'sandboxId', style: 'simple', explode: false })}/files/${serializePathParameter(entryId, { name: 'entryId', style: 'simple', explode: false })}/content`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PUT' as any, body, contentType: 'application/json', ...(requestHeaders !== undefined ? { headers: requestHeaders } : {}), sdkworkUnwrapKind: 'item' });
  }
}

export interface DriveSandboxFilesCreateParams {
  idempotencyKey: string;
}

export class DriveSandboxFilesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(sandboxId: string, body: CreateDriveSandboxFileRequest, params: DriveSandboxFilesCreateParams, requestOptions?: ApiRequestOptions): Promise<DriveSandboxEntry> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.request<DriveSandboxEntry>(appApiPath(`/drive/sandboxes/${serializePathParameter(sandboxId, { name: 'sandboxId', style: 'simple', explode: false })}/files`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', ...(requestHeaders !== undefined ? { headers: requestHeaders } : {}), sdkworkUnwrapKind: 'item' });
  }
}

export interface DriveSandboxDirectoriesCreateParams {
  idempotencyKey: string;
}

export class DriveSandboxDirectoriesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(sandboxId: string, body: CreateDriveSandboxDirectoryRequest, params: DriveSandboxDirectoriesCreateParams, requestOptions?: ApiRequestOptions): Promise<DriveSandboxEntry> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.request<DriveSandboxEntry>(appApiPath(`/drive/sandboxes/${serializePathParameter(sandboxId, { name: 'sandboxId', style: 'simple', explode: false })}/directories`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', ...(requestHeaders !== undefined ? { headers: requestHeaders } : {}), sdkworkUnwrapKind: 'item' });
  }
}

export interface DriveSandboxEntriesListParams {
  parentPath?: string;
  cursor?: string;
  pageSize?: number;
}

export interface DriveSandboxEntriesUpdateParams {
  ifMatch: string;
  idempotencyKey: string;
}

export interface DriveSandboxEntriesPurgeParams {
  ifMatch: string;
  idempotencyKey: string;
}

export class DriveSandboxEntriesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(sandboxId: string, params?: DriveSandboxEntriesListParams, requestOptions?: ApiRequestOptions): Promise<DriveSandboxEntryListData> {
    const query = buildQueryString([
      { name: 'parent_path', value: params?.parentPath, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<DriveSandboxEntryListData>(appendQueryString(appApiPath(`/drive/sandboxes/${serializePathParameter(sandboxId, { name: 'sandboxId', style: 'simple', explode: false })}/entries`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

async update(sandboxId: string, entryId: string, body: UpdateDriveSandboxEntryRequest, params: DriveSandboxEntriesUpdateParams, requestOptions?: ApiRequestOptions): Promise<DriveSandboxEntry> {
    const requestHeaders = buildRequestHeaders(
      {
        'If-Match': { value: params.ifMatch, style: 'simple', explode: false },
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.request<DriveSandboxEntry>(appApiPath(`/drive/sandboxes/${serializePathParameter(sandboxId, { name: 'sandboxId', style: 'simple', explode: false })}/entries/${serializePathParameter(entryId, { name: 'entryId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PATCH' as any, body, contentType: 'application/json', ...(requestHeaders !== undefined ? { headers: requestHeaders } : {}), sdkworkUnwrapKind: 'item' });
  }

async purge(sandboxId: string, entryId: string, body: PurgeDriveSandboxEntryRequest, params: DriveSandboxEntriesPurgeParams, requestOptions?: ApiRequestOptions): Promise<DriveSandboxMutationCommandData> {
    const requestHeaders = buildRequestHeaders(
      {
        'If-Match': { value: params.ifMatch, style: 'simple', explode: false },
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.request<DriveSandboxMutationCommandData>(appApiPath(`/drive/sandboxes/${serializePathParameter(sandboxId, { name: 'sandboxId', style: 'simple', explode: false })}/entries/${serializePathParameter(entryId, { name: 'entryId', style: 'simple', explode: false })}/purge`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', ...(requestHeaders !== undefined ? { headers: requestHeaders } : {}), sdkworkUnwrapKind: 'command' });
  }
}

export interface DriveSandboxesListParams {
  page?: number;
  pageSize?: number;
}

export class DriveSandboxesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: DriveSandboxesListParams, requestOptions?: ApiRequestOptions): Promise<DriveSandboxVolumeListData> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<DriveSandboxVolumeListData>(appendQueryString(appApiPath(`/drive/sandboxes`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }
}

export interface DriveSharedWithMeListParams {
  spaceId?: string;
  pageSize?: string;
  cursor?: string;
  sortBy?: 'name' | 'owner' | 'lastModified' | 'contentLength' | 'type';
  sortOrder?: 'asc' | 'desc';
}

export class DriveSharedWithMeApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: DriveSharedWithMeListParams, requestOptions?: ApiRequestOptions): Promise<DriveNodeListData> {
    const query = buildQueryString([
      { name: 'spaceId', value: params?.spaceId, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'sortBy', value: params?.sortBy, style: 'form', explode: true, allowReserved: false },
      { name: 'sortOrder', value: params?.sortOrder, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<DriveNodeListData>(appendQueryString(appApiPath(`/drive/shared_with_me`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }
}

export interface DriveSearchListParams {
  q?: string;
  spaceId?: string;
  pageSize?: string;
  cursor?: string;
}

export class DriveSearchApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: DriveSearchListParams, requestOptions?: ApiRequestOptions): Promise<DriveNodeListData> {
    const query = buildQueryString([
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'spaceId', value: params?.spaceId, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<DriveNodeListData>(appendQueryString(appApiPath(`/drive/search`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }
}

export interface DriveRecentListParams {
  spaceId?: string;
  pageSize?: string;
  cursor?: string;
  sortBy?: 'name' | 'owner' | 'lastModified' | 'contentLength' | 'type';
  sortOrder?: 'asc' | 'desc';
}

export class DriveRecentApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: DriveRecentListParams, requestOptions?: ApiRequestOptions): Promise<DriveNodeListData> {
    const query = buildQueryString([
      { name: 'spaceId', value: params?.spaceId, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'sortBy', value: params?.sortBy, style: 'form', explode: true, allowReserved: false },
      { name: 'sortOrder', value: params?.sortOrder, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<DriveNodeListData>(appendQueryString(appApiPath(`/drive/recent`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }
}

export interface DrivePropertyNodesListParams {
  pageSize?: string;
  cursor?: string;
}

export class DrivePropertyNodesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List nodes carrying an app_public property */
  async list(propertyKey: string, params?: DrivePropertyNodesListParams, requestOptions?: ApiRequestOptions): Promise<DriveNodeListData> {
    const query = buildQueryString([
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<DriveNodeListData>(appendQueryString(appApiPath(`/drive/properties/${serializePathParameter(propertyKey, { name: 'propertyKey', style: 'simple', explode: false })}/nodes`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }
}

export interface DriveVersionsListParams {
  pageSize?: string;
  cursor?: string;
}

export class DriveVersionsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(nodeId: string, params?: DriveVersionsListParams, requestOptions?: ApiRequestOptions): Promise<FileVersionListData> {
    const query = buildQueryString([
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<FileVersionListData>(appendQueryString(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/versions`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

async delete(nodeId: string, versionId: string, requestOptions?: ApiRequestOptions): Promise<void> {
    return this.client.request<void>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/versions/${serializePathParameter(versionId, { name: 'versionId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'DELETE' as any });
  }

async retrieve(nodeId: string, versionId: string, requestOptions?: ApiRequestOptions): Promise<FileVersion> {
    return this.client.request<FileVersion>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/versions/${serializePathParameter(versionId, { name: 'versionId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

async restore(nodeId: string, versionId: string, body: NodeCommandRequest, requestOptions?: ApiRequestOptions): Promise<DriveNode> {
    return this.client.request<DriveNode>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/versions/${serializePathParameter(versionId, { name: 'versionId', style: 'simple', explode: false })}/restore`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export interface DriveTrashListParams {
  spaceId?: string;
  pageSize?: string;
  cursor?: string;
  parentNodeId?: string;
  sortBy?: 'name' | 'owner' | 'lastModified' | 'contentLength' | 'type';
  sortOrder?: 'asc' | 'desc';
}

export class DriveTrashApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(nodeId: string, body: NodeCommandRequest, requestOptions?: ApiRequestOptions): Promise<DriveNode> {
    return this.client.request<DriveNode>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/trash`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

async list(params?: DriveTrashListParams, requestOptions?: ApiRequestOptions): Promise<DriveNodeListData> {
    const query = buildQueryString([
      { name: 'spaceId', value: params?.spaceId, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'parentNodeId', value: params?.parentNodeId, style: 'form', explode: true, allowReserved: false },
      { name: 'sortBy', value: params?.sortBy, style: 'form', explode: true, allowReserved: false },
      { name: 'sortOrder', value: params?.sortOrder, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<DriveNodeListData>(appendQueryString(appApiPath(`/drive/trash`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

async restore(nodeId: string, body: NodeCommandRequest, requestOptions?: ApiRequestOptions): Promise<DriveNode> {
    return this.client.request<DriveNode>(appApiPath(`/drive/trash/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/restore`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

async empty(body: EmptyTrashRequest, requestOptions?: ApiRequestOptions): Promise<EmptyTrashResponse> {
    return this.client.request<EmptyTrashResponse>(appApiPath(`/drive/trash/empty`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }
}

export interface DriveShareLinksListParams {
  pageSize?: string;
  cursor?: string;
}

export class DriveShareLinksApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(nodeId: string, body: CreateShareLinkRequest, requestOptions?: ApiRequestOptions): Promise<CreateShareLinkResponse> {
    return this.client.request<CreateShareLinkResponse>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/share_links`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }

async list(nodeId: string, params?: DriveShareLinksListParams, requestOptions?: ApiRequestOptions): Promise<{ items: DriveShareLink[]; pageInfo: PageInfo; }> {
    const query = buildQueryString([
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<{ items: DriveShareLink[]; pageInfo: PageInfo; }>(appendQueryString(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/share_links`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

async claim(token: string, requestOptions?: ApiRequestOptions): Promise<ClaimShareLinkResponse> {
    return this.client.request<ClaimShareLinkResponse>(appApiPath(`/drive/share_links/${serializePathParameter(token, { name: 'token', style: 'simple', explode: false })}/claim`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, sdkworkUnwrapKind: 'data' });
  }

async delete(shareLinkId: string, requestOptions?: ApiRequestOptions): Promise<void> {
    return this.client.request<void>(appApiPath(`/drive/share_links/${serializePathParameter(shareLinkId, { name: 'shareLinkId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'DELETE' as any });
  }

async update(shareLinkId: string, body: UpdateShareLinkRequest, requestOptions?: ApiRequestOptions): Promise<DriveShareLink> {
    return this.client.request<DriveShareLink>(appApiPath(`/drive/share_links/${serializePathParameter(shareLinkId, { name: 'shareLinkId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PATCH' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

async retrieve(shareLinkId: string, requestOptions?: ApiRequestOptions): Promise<DriveShareLink> {
    return this.client.request<DriveShareLink>(appApiPath(`/drive/share_links/${serializePathParameter(shareLinkId, { name: 'shareLinkId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }
}

export interface DriveNodePropertiesListParams {
  visibility?: 'private' | 'app_public';
  pageSize?: number;
  cursor?: string;
}

export interface DriveNodePropertiesDeleteParams {
  visibility?: 'private' | 'app_public';
}

export class DriveNodePropertiesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List node custom properties */
  async list(nodeId: string, params?: DriveNodePropertiesListParams, requestOptions?: ApiRequestOptions): Promise<{ items: DriveNodeProperty[]; pageInfo: PageInfo; }> {
    const query = buildQueryString([
      { name: 'visibility', value: params?.visibility, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<{ items: DriveNodeProperty[]; pageInfo: PageInfo; }>(appendQueryString(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/properties`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create or update a node custom property */
  async update(nodeId: string, propertyKey: string, body: SetNodePropertyRequest, requestOptions?: ApiRequestOptions): Promise<DriveNodeProperty> {
    return this.client.request<DriveNodeProperty>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/properties/${serializePathParameter(propertyKey, { name: 'propertyKey', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PUT' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Delete a node custom property */
  async delete(nodeId: string, propertyKey: string, params?: DriveNodePropertiesDeleteParams, requestOptions?: ApiRequestOptions): Promise<void> {
    const query = buildQueryString([
      { name: 'visibility', value: params?.visibility, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<void>(appendQueryString(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/properties/${serializePathParameter(propertyKey, { name: 'propertyKey', style: 'simple', explode: false })}`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'DELETE' as any });
  }
}

export interface DrivePermissionsEffectiveListParams {
  pageSize?: string;
  cursor?: string;
}

export class DrivePermissionsEffectiveApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(nodeId: string, params?: DrivePermissionsEffectiveListParams, requestOptions?: ApiRequestOptions): Promise<{ items: EffectivePermission[]; pageInfo: PageInfo; }> {
    const query = buildQueryString([
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<{ items: EffectivePermission[]; pageInfo: PageInfo; }>(appendQueryString(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/permissions/effective`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }
}

export interface DrivePermissionsListParams {
  pageSize?: string;
  cursor?: string;
}

export class DrivePermissionsApi {
  private client: HttpClient;
  public readonly effective: DrivePermissionsEffectiveApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.effective = new DrivePermissionsEffectiveApi(client);
  }


async list(nodeId: string, params?: DrivePermissionsListParams, requestOptions?: ApiRequestOptions): Promise<{ items: DrivePermission[]; pageInfo: PageInfo; }> {
    const query = buildQueryString([
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<{ items: DrivePermission[]; pageInfo: PageInfo; }>(appendQueryString(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/permissions`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

async create(nodeId: string, body: CreatePermissionRequest, requestOptions?: ApiRequestOptions): Promise<DrivePermission> {
    return this.client.request<DrivePermission>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/permissions`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

async delete(nodeId: string, permissionId: string, requestOptions?: ApiRequestOptions): Promise<void> {
    return this.client.request<void>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/permissions/${serializePathParameter(permissionId, { name: 'permissionId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'DELETE' as any });
  }

async update(nodeId: string, permissionId: string, body: UpdatePermissionRequest, requestOptions?: ApiRequestOptions): Promise<DrivePermission> {
    return this.client.request<DrivePermission>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/permissions/${serializePathParameter(permissionId, { name: 'permissionId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PATCH' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

async retrieve(nodeId: string, permissionId: string, requestOptions?: ApiRequestOptions): Promise<DrivePermission> {
    return this.client.request<DrivePermission>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/permissions/${serializePathParameter(permissionId, { name: 'permissionId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }
}

export interface DriveNodeLabelsListParams {
  labelKey?: string;
  pageSize?: number;
  cursor?: string;
}

export class DriveNodeLabelsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List labels applied to a node */
  async list(nodeId: string, params?: DriveNodeLabelsListParams, requestOptions?: ApiRequestOptions): Promise<{ items: NodeLabel[]; pageInfo: PageInfo; }> {
    const query = buildQueryString([
      { name: 'labelKey', value: params?.labelKey, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<{ items: NodeLabel[]; pageInfo: PageInfo; }>(appendQueryString(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/labels`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Apply a label to a node */
  async update(nodeId: string, labelId: string, body: ApplyNodeLabelRequest, requestOptions?: ApiRequestOptions): Promise<NodeLabel> {
    return this.client.request<NodeLabel>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/labels/${serializePathParameter(labelId, { name: 'labelId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PUT' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Remove a label from a node */
  async delete(nodeId: string, labelId: string, requestOptions?: ApiRequestOptions): Promise<void> {
    return this.client.request<void>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/labels/${serializePathParameter(labelId, { name: 'labelId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'DELETE' as any });
  }
}

export class DriveDownloadGrantsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(nodeId: string, body?: CreateDownloadGrantRequest, requestOptions?: ApiRequestOptions): Promise<CreateDownloadUrlResponse> {
    return this.client.request<CreateDownloadUrlResponse>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/download_grants`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, ...(body !== undefined ? { body, contentType: 'application/json' } : {}), sdkworkUnwrapKind: 'data' });
  }
}

export interface DriveCommentRepliesListParams {
  pageSize?: string;
  cursor?: string;
}

export class DriveCommentRepliesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(nodeId: string, commentId: string, params?: DriveCommentRepliesListParams, requestOptions?: ApiRequestOptions): Promise<{ items: DriveCommentReply[]; pageInfo: PageInfo; }> {
    const query = buildQueryString([
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<{ items: DriveCommentReply[]; pageInfo: PageInfo; }>(appendQueryString(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/comments/${serializePathParameter(commentId, { name: 'commentId', style: 'simple', explode: false })}/replies`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

async create(nodeId: string, commentId: string, body: CreateCommentReplyRequest, requestOptions?: ApiRequestOptions): Promise<DriveCommentReply> {
    return this.client.request<DriveCommentReply>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/comments/${serializePathParameter(commentId, { name: 'commentId', style: 'simple', explode: false })}/replies`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

async retrieve(nodeId: string, commentId: string, replyId: string, requestOptions?: ApiRequestOptions): Promise<DriveCommentReply> {
    return this.client.request<DriveCommentReply>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/comments/${serializePathParameter(commentId, { name: 'commentId', style: 'simple', explode: false })}/replies/${serializePathParameter(replyId, { name: 'replyId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

async update(nodeId: string, commentId: string, replyId: string, body: UpdateCommentReplyRequest, requestOptions?: ApiRequestOptions): Promise<DriveCommentReply> {
    return this.client.request<DriveCommentReply>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/comments/${serializePathParameter(commentId, { name: 'commentId', style: 'simple', explode: false })}/replies/${serializePathParameter(replyId, { name: 'replyId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PATCH' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

async delete(nodeId: string, commentId: string, replyId: string, requestOptions?: ApiRequestOptions): Promise<void> {
    return this.client.request<void>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/comments/${serializePathParameter(commentId, { name: 'commentId', style: 'simple', explode: false })}/replies/${serializePathParameter(replyId, { name: 'replyId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'DELETE' as any });
  }
}

export interface DriveCommentsListParams {
  pageSize?: string;
  cursor?: string;
}

export class DriveCommentsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(nodeId: string, params?: DriveCommentsListParams, requestOptions?: ApiRequestOptions): Promise<{ items: DriveComment[]; pageInfo: PageInfo; }> {
    const query = buildQueryString([
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<{ items: DriveComment[]; pageInfo: PageInfo; }>(appendQueryString(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/comments`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

async create(nodeId: string, body: CreateCommentRequest, requestOptions?: ApiRequestOptions): Promise<DriveComment> {
    return this.client.request<DriveComment>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/comments`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

async retrieve(nodeId: string, commentId: string, requestOptions?: ApiRequestOptions): Promise<DriveComment> {
    return this.client.request<DriveComment>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/comments/${serializePathParameter(commentId, { name: 'commentId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

async update(nodeId: string, commentId: string, body: UpdateCommentRequest, requestOptions?: ApiRequestOptions): Promise<DriveComment> {
    return this.client.request<DriveComment>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/comments/${serializePathParameter(commentId, { name: 'commentId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PATCH' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

async delete(nodeId: string, commentId: string, requestOptions?: ApiRequestOptions): Promise<void> {
    return this.client.request<void>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/comments/${serializePathParameter(commentId, { name: 'commentId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'DELETE' as any });
  }
}

export class DriveNodesShortcutsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a shortcut node */
  async create(body: CreateShortcutRequest, requestOptions?: ApiRequestOptions): Promise<DriveNode> {
    return this.client.request<DriveNode>(appApiPath(`/drive/nodes/shortcuts`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class DriveNodesFoldersApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateFolderRequest, requestOptions?: ApiRequestOptions): Promise<DriveNode> {
    return this.client.request<DriveNode>(appApiPath(`/drive/nodes/folders`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class DriveNodesFilesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateFileRequest, requestOptions?: ApiRequestOptions): Promise<CreateFileResponse> {
    return this.client.request<CreateFileResponse>(appApiPath(`/drive/nodes/files`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }
}

export class DriveNodesPathApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(nodeId: string, requestOptions?: ApiRequestOptions): Promise<NodePathResponse> {
    return this.client.request<NodePathResponse>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/path`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'data' });
  }
}

export interface DriveNodesDownloadUrlsRetrieveParams {
  requestedTtlSeconds?: number;
}

export class DriveNodesDownloadUrlsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(nodeId: string, params?: DriveNodesDownloadUrlsRetrieveParams, requestOptions?: ApiRequestOptions): Promise<CreateDownloadUrlResponse> {
    const query = buildQueryString([
      { name: 'requestedTtlSeconds', value: params?.requestedTtlSeconds, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<CreateDownloadUrlResponse>(appendQueryString(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/download_url`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'data' });
  }
}

export class DriveNodesCapabilitiesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(nodeId: string, requestOptions?: ApiRequestOptions): Promise<NodeCapabilitiesResponse> {
    return this.client.request<NodeCapabilitiesResponse>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/capabilities`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }
}

export interface DriveNodesListParams {
  parentNodeId?: string;
  pageSize?: string;
  cursor?: string;
  sortBy?: 'name' | 'owner' | 'lastModified' | 'contentLength' | 'type';
  sortOrder?: 'asc' | 'desc';
}

export class DriveNodesApi {
  private client: HttpClient;
  public readonly capabilities: DriveNodesCapabilitiesApi;
  public readonly downloadUrls: DriveNodesDownloadUrlsApi;
  public readonly path: DriveNodesPathApi;
  public readonly files: DriveNodesFilesApi;
  public readonly folders: DriveNodesFoldersApi;
  public readonly shortcuts: DriveNodesShortcutsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.capabilities = new DriveNodesCapabilitiesApi(client);
    this.downloadUrls = new DriveNodesDownloadUrlsApi(client);
    this.path = new DriveNodesPathApi(client);
    this.files = new DriveNodesFilesApi(client);
    this.folders = new DriveNodesFoldersApi(client);
    this.shortcuts = new DriveNodesShortcutsApi(client);
  }


async update(nodeId: string, body: UpdateNodeRequest, requestOptions?: ApiRequestOptions): Promise<DriveNode> {
    return this.client.request<DriveNode>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PATCH' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

async retrieve(nodeId: string, requestOptions?: ApiRequestOptions): Promise<DriveNode> {
    return this.client.request<DriveNode>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

async delete(nodeId: string, requestOptions?: ApiRequestOptions): Promise<void> {
    return this.client.request<void>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'DELETE' as any });
  }

async copy(nodeId: string, body: CopyNodeRequest, requestOptions?: ApiRequestOptions): Promise<DriveNode> {
    return this.client.request<DriveNode>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/copy`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

async move(nodeId: string, body: MoveNodeRequest, requestOptions?: ApiRequestOptions): Promise<DriveNode> {
    return this.client.request<DriveNode>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/move`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

async list(spaceId: string, params?: DriveNodesListParams, requestOptions?: ApiRequestOptions): Promise<DriveNodeListData> {
    const query = buildQueryString([
      { name: 'parentNodeId', value: params?.parentNodeId, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'sortBy', value: params?.sortBy, style: 'form', explode: true, allowReserved: false },
      { name: 'sortOrder', value: params?.sortOrder, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<DriveNodeListData>(appendQueryString(appApiPath(`/drive/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}/nodes`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create a push notification channel for a Drive node */
  async watch(nodeId: string, body: CreateWatchChannelRequest, requestOptions?: ApiRequestOptions): Promise<DriveWatchChannel> {
    return this.client.request<DriveWatchChannel>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/watch`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class DriveQuotasApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(requestOptions?: ApiRequestOptions): Promise<QuotaSummary> {
    return this.client.request<QuotaSummary>(appApiPath(`/drive/quotas/summary`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }
}

export interface DriveFavoritesListParams {
  spaceId?: string;
  pageSize?: string;
  cursor?: string;
  sortBy?: 'name' | 'owner' | 'lastModified' | 'contentLength' | 'type';
  sortOrder?: 'asc' | 'desc';
}

export class DriveFavoritesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: DriveFavoritesListParams, requestOptions?: ApiRequestOptions): Promise<DriveNodeListData> {
    const query = buildQueryString([
      { name: 'spaceId', value: params?.spaceId, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'sortBy', value: params?.sortBy, style: 'form', explode: true, allowReserved: false },
      { name: 'sortOrder', value: params?.sortOrder, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<DriveNodeListData>(appendQueryString(appApiPath(`/drive/favorites`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

async check(body: CheckFavoriteNodesRequest, requestOptions?: ApiRequestOptions): Promise<unknown> {
    return this.client.request<unknown>(appApiPath(`/drive/favorites/check`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }

async update(nodeId: string, body: FavoriteNodeRequest, requestOptions?: ApiRequestOptions): Promise<FavoriteNodeResponse> {
    return this.client.request<FavoriteNodeResponse>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/favorite`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PUT' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }

async delete(nodeId: string, requestOptions?: ApiRequestOptions): Promise<void> {
    return this.client.request<void>(appApiPath(`/drive/nodes/${serializePathParameter(nodeId, { name: 'nodeId', style: 'simple', explode: false })}/favorite`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'DELETE' as any });
  }
}

export class DriveDownloadUrlsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateDownloadUrlRequest, requestOptions?: ApiRequestOptions): Promise<CreateDownloadUrlResponse> {
    return this.client.request<CreateDownloadUrlResponse>(appApiPath(`/drive/download_urls`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }
}

export class DriveDownloadTokensApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(token: string, requestOptions?: ApiRequestOptions): Promise<CreateDownloadUrlResponse> {
    return this.client.request<CreateDownloadUrlResponse>(appApiPath(`/drive/download_tokens/${serializePathParameter(token, { name: 'token', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'data' });
  }
}

export interface DriveChangesStartPageTokenRetrieveParams {
  spaceId: string;
}

export class DriveChangesStartPageTokenApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(params: DriveChangesStartPageTokenRetrieveParams, requestOptions?: ApiRequestOptions): Promise<StartPageTokenResponse> {
    const query = buildQueryString([
      { name: 'spaceId', value: params.spaceId, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<StartPageTokenResponse>(appendQueryString(appApiPath(`/drive/changes/start_page_token`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'data' });
  }
}

export interface DriveChangesListParams {
  spaceId: string;
  cursor?: string;
  pageSize?: string;
}

export class DriveChangesApi {
  private client: HttpClient;
  public readonly startPageToken: DriveChangesStartPageTokenApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.startPageToken = new DriveChangesStartPageTokenApi(client);
  }


async list(params: DriveChangesListParams, requestOptions?: ApiRequestOptions): Promise<ChangeListData> {
    const query = buildQueryString([
      { name: 'spaceId', value: params.spaceId, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<ChangeListData>(appendQueryString(appApiPath(`/drive/changes`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create a push notification channel for Drive changes */
  async watch(body: CreateWatchChannelRequest, requestOptions?: ApiRequestOptions): Promise<DriveWatchChannel> {
    return this.client.request<DriveWatchChannel>(appApiPath(`/drive/changes/watch`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class DriveApi {
  public readonly changes: DriveChangesApi;
  public readonly downloadTokens: DriveDownloadTokensApi;
  public readonly downloadUrls: DriveDownloadUrlsApi;
  public readonly favorites: DriveFavoritesApi;
  public readonly quotas: DriveQuotasApi;
  public readonly nodes: DriveNodesApi;
  public readonly comments: DriveCommentsApi;
  public readonly commentReplies: DriveCommentRepliesApi;
  public readonly downloadGrants: DriveDownloadGrantsApi;
  public readonly nodeLabels: DriveNodeLabelsApi;
  public readonly permissions: DrivePermissionsApi;
  public readonly nodeProperties: DriveNodePropertiesApi;
  public readonly shareLinks: DriveShareLinksApi;
  public readonly trash: DriveTrashApi;
  public readonly versions: DriveVersionsApi;
  public readonly propertyNodes: DrivePropertyNodesApi;
  public readonly recent: DriveRecentApi;
  public readonly search: DriveSearchApi;
  public readonly sharedWithMe: DriveSharedWithMeApi;
  public readonly sandboxes: DriveSandboxesApi;
  public readonly sandboxEntries: DriveSandboxEntriesApi;
  public readonly sandboxDirectories: DriveSandboxDirectoriesApi;
  public readonly sandboxFiles: DriveSandboxFilesApi;
  public readonly sandboxFileContents: DriveSandboxFileContentsApi;
  public readonly spaces: DriveSpacesApi;
  public readonly websiteRoots: DriveWebsiteRootsApi;
  public readonly moveDestinations: DriveMoveDestinationsApi;
  public readonly uploadSessions: DriveUploadSessionsApi;
  public readonly watchChannels: DriveWatchChannelsApi;
  public readonly downloadPackages: DriveDownloadPackagesApi;
  public readonly archiveEntries: DriveArchiveEntriesApi;
  public readonly uploader: DriveUploaderApi;

  constructor(client: HttpClient) {
    this.changes = new DriveChangesApi(client);
    this.downloadTokens = new DriveDownloadTokensApi(client);
    this.downloadUrls = new DriveDownloadUrlsApi(client);
    this.favorites = new DriveFavoritesApi(client);
    this.quotas = new DriveQuotasApi(client);
    this.nodes = new DriveNodesApi(client);
    this.comments = new DriveCommentsApi(client);
    this.commentReplies = new DriveCommentRepliesApi(client);
    this.downloadGrants = new DriveDownloadGrantsApi(client);
    this.nodeLabels = new DriveNodeLabelsApi(client);
    this.permissions = new DrivePermissionsApi(client);
    this.nodeProperties = new DriveNodePropertiesApi(client);
    this.shareLinks = new DriveShareLinksApi(client);
    this.trash = new DriveTrashApi(client);
    this.versions = new DriveVersionsApi(client);
    this.propertyNodes = new DrivePropertyNodesApi(client);
    this.recent = new DriveRecentApi(client);
    this.search = new DriveSearchApi(client);
    this.sharedWithMe = new DriveSharedWithMeApi(client);
    this.sandboxes = new DriveSandboxesApi(client);
    this.sandboxEntries = new DriveSandboxEntriesApi(client);
    this.sandboxDirectories = new DriveSandboxDirectoriesApi(client);
    this.sandboxFiles = new DriveSandboxFilesApi(client);
    this.sandboxFileContents = new DriveSandboxFileContentsApi(client);
    this.spaces = new DriveSpacesApi(client);
    this.websiteRoots = new DriveWebsiteRootsApi(client);
    this.moveDestinations = new DriveMoveDestinationsApi(client);
    this.uploadSessions = new DriveUploadSessionsApi(client);
    this.watchChannels = new DriveWatchChannelsApi(client);
    this.downloadPackages = new DriveDownloadPackagesApi(client);
    this.archiveEntries = new DriveArchiveEntriesApi(client);
    this.uploader = new DriveUploaderApi(client);
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
