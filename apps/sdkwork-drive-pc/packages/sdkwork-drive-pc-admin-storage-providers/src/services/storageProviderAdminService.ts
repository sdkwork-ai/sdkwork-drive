import {
  DriveAdminStorageSdkError,
  type DriveAdminStorageSdkClient,
} from 'sdkwork-drive-pc-admin-core';
import type { SessionSnapshot } from 'sdkwork-drive-pc-core';
import type {
  AbortStorageProviderMultipartUploadInput,
  CompleteStorageProviderMultipartUploadInput,
  CopyStorageProviderObjectInput,
  CreateStorageProviderAccountInput,
  CreateStorageProviderInput,
  CreateStorageProviderMultipartUploadInput,
  GetStorageOverviewInput,
  ListStorageProviderAccountsInput,
  ListStorageProviderBindingsInput,
  ListStorageProviderBindingsPageResult,
  ListStorageProviderAccountsPageResult,
  ListStorageProvidersInput,
  ListStorageProvidersPageResult,
  ListStorageProviderObjectsInput,
  ListStorageProviderObjectsResult,
  PresignStorageProviderUploadPartsInput,
  SetDefaultStorageProviderBindingInput,
  StorageOverviewView,
  StorageProviderAccountDefaultView,
  StorageProviderAccountScope,
  StorageProviderAccountView,
  StorageProviderBindingView,
  StorageProviderBucketInitializeView,
  StorageProviderBucketListItemView,
  StorageProviderBucketView,
  StorageProviderCapabilitiesView,
  StorageProviderKindView,
  StorageProviderMutationOptions,
  StorageProviderMultipartUploadView,
  StorageProviderObjectContentView,
  StorageProviderObjectMutationResult,
  StorageProviderObjectScopeOptions,
  StorageProviderObjectView,
  StorageProviderVendorCapabilityDefaults,
  StorageProviderVendorCredentialFields,
  StorageProviderView,
  StorageProviderUploadPartGrantsView,
  UpdateStorageProviderInput,
  WriteStorageProviderObjectContentInput,
} from '../types/storageProviderAdminTypes';

type JsonRecord = Record<string, unknown>;

/**
 * 服务商桶清单每页读取的窗口。
 *
 * `storageProviders.buckets.list` 的 `page_size` 声明是 1..=200（`PAGINATION_SPEC.md`
 * §3），后端把厂商 ListBuckets 返回的整份清单按 200 上限截断后再做 offset 分页——超过上限
 * 的账号直接报错，而不是给半份清单。所以按上限读一次就是「一次拿全」，续页只是兜底。
 */
const BUCKET_INVENTORY_PAGE_SIZE = 200;

export interface StorageProviderAdminService {
  /**
   * 存储中心仪表盘聚合。
   *
   * 一次请求拿全：容量、provider 用量分布、绑定健康度、目录覆盖、月度趋势。
   * 分项统计接口无法拼出同样的口径（尤其「本租户用到的 provider」这个集合），
   * 所以读 aggregate 而不是组合 list。
   */
  getStorageOverview(input?: GetStorageOverviewInput): Promise<StorageOverviewView>;
  listProviders(input?: ListStorageProvidersInput): Promise<StorageProviderView[]>;
  listProvidersPage(input?: ListStorageProvidersInput): Promise<ListStorageProvidersPageResult>;
  listKinds(input?: { signal?: AbortSignal }): Promise<StorageProviderKindView[]>;
  setKindEnabled(
    providerKind: string,
    enabled: boolean,
    options?: StorageProviderMutationOptions,
  ): Promise<StorageProviderKindView>;
  initializeKinds(options?: StorageProviderMutationOptions): Promise<StorageProviderKindView[]>;
  /**
   * 把内置服务商的账号中心账号、服务商配置与租户默认绑定一次铺齐。
   *
   * 幂等：已有的东西一概不动（尤其不会覆盖运维已填的真实密钥），所以可以重复点。
   */
  initializeProviderAccountDefaults(
    options?: StorageProviderMutationOptions,
  ): Promise<StorageProviderAccountDefaultView[]>;
  createProvider(
    input: CreateStorageProviderInput,
    options?: StorageProviderMutationOptions,
  ): Promise<StorageProviderView>;
  updateProvider(
    providerId: string,
    input: UpdateStorageProviderInput,
    options?: StorageProviderMutationOptions,
  ): Promise<StorageProviderView>;
  deleteProvider(providerId: string, options?: StorageProviderMutationOptions): Promise<boolean>;
  testProvider(providerId: string, options?: StorageProviderMutationOptions): Promise<boolean>;
  activateProvider(providerId: string, options?: StorageProviderMutationOptions): Promise<StorageProviderView>;
  deactivateProvider(providerId: string, options?: StorageProviderMutationOptions): Promise<StorageProviderView>;
  rotateCredential(
    providerId: string,
    credentialRef: string,
    options?: StorageProviderMutationOptions,
  ): Promise<StorageProviderView>;
  getCapabilities(
    providerId: string,
    options?: StorageProviderMutationOptions,
  ): Promise<StorageProviderCapabilitiesView>;
  headBucket(
    providerId: string,
    options?: StorageProviderMutationOptions,
  ): Promise<StorageProviderBucketView>;
  getDefaultBinding(
    spaceId?: string,
    options?: StorageProviderMutationOptions,
  ): Promise<StorageProviderBindingView | undefined>;
  setDefaultBinding(input: SetDefaultStorageProviderBindingInput): Promise<StorageProviderBindingView>;
  deleteDefaultBinding(
    spaceIdOrSpaceType?: string,
    options?: StorageProviderMutationOptions & { spaceType?: boolean },
  ): Promise<boolean>;
  setSpaceTypeBinding(input: SetDefaultStorageProviderBindingInput & { spaceType: string }): Promise<StorageProviderBindingView>;
  deleteSpaceTypeBinding(spaceType: string, options?: StorageProviderMutationOptions): Promise<boolean>;
  listBindings(
    input?: ListStorageProviderBindingsInput,
  ): Promise<StorageProviderBindingView[]>;
  /**
   * One page of bindings, with the continuation the caller needs to read the
   * rest.
   *
   * The console renders one section per resolution step, and steps are ranked
   * (`space` → `space_type` → `tenant`), so a caller that reads the unfiltered
   * list as "this step's rows" is reading one page of every step: enough
   * space-scoped bindings push the space-type rows out of the window and the
   * section then renders them as unbound. Pass `bindingScope` and follow
   * `nextPageToken`.
   */
  listBindingsPage(
    input?: ListStorageProviderBindingsInput,
  ): Promise<ListStorageProviderBindingsPageResult>;
  /**
   * 该服务商账号下**全部**可见的存储桶。
   *
   * 厂商 ListBuckets 本来就是整份账号清单，接口的 offset 分页只是限制单次响应大小；
   * 调用方要回答的是"这个配置下有哪些桶"，所以这里读完游标链再返回，而不是只给第一页
   * ——那会让窗口之后的桶在页面上凭空消失，而页面（含数量徽标与搜索）把它们当成全集。
   */
  listBuckets(providerId: string, options?: StorageProviderMutationOptions): Promise<StorageProviderBucketListItemView[]>;
  /** Idempotent initialization: ensures the configured bucket exists on the vendor. */
  initializeBucket(providerId: string, options?: StorageProviderMutationOptions): Promise<StorageProviderBucketInitializeView>;
  deleteBucket(providerId: string, options?: StorageProviderMutationOptions): Promise<boolean>;
  listObjects(
    providerId: string,
    input?: ListStorageProviderObjectsInput,
  ): Promise<ListStorageProviderObjectsResult>;
  deleteObject(
    providerId: string,
    objectKey: string,
    options?: StorageProviderObjectScopeOptions,
  ): Promise<boolean>;
  readObjectContent(
    providerId: string,
    objectKey: string,
    options?: StorageProviderObjectScopeOptions,
  ): Promise<StorageProviderObjectContentView>;
  writeObjectContent(
    providerId: string,
    objectKey: string,
    input: WriteStorageProviderObjectContentInput,
    options?: StorageProviderObjectScopeOptions,
  ): Promise<StorageProviderObjectView>;
  /**
   * 开启一次预签名分片上传，拿到厂商的 `uploadId`。
   *
   * 单次内容接口上限 8 MiB，而且内容以 base64 装在 JSON 里（请求体还会放大 1.37 倍）；
   * 更大的对象走这条路：开启 → 逐片签发直传 URL → 完成/中止。字节不经过本服务。
   */
  createMultipartUpload(
    providerId: string,
    input: CreateStorageProviderMultipartUploadInput,
    options?: StorageProviderObjectScopeOptions,
  ): Promise<StorageProviderMultipartUploadView>;
  /**
   * 为一批分片签发直传授权。
   *
   * 返回的 `headers` 是签名的一部分，客户端必须原样回放；`url` 指向厂商地址。
   */
  presignUploadParts(
    providerId: string,
    input: PresignStorageProviderUploadPartsInput,
    options?: StorageProviderObjectScopeOptions,
  ): Promise<StorageProviderUploadPartGrantsView>;
  /** 用已上传分片的 ETag 完成上传，返回落盘后的对象。 */
  completeMultipartUpload(
    providerId: string,
    input: CompleteStorageProviderMultipartUploadInput,
    options?: StorageProviderObjectScopeOptions,
  ): Promise<StorageProviderObjectView>;
  /** 中止上传：运营商取消或关闭时必须调用，否则已上传的分片会继续计费。 */
  abortMultipartUpload(
    providerId: string,
    input: AbortStorageProviderMultipartUploadInput,
    options?: StorageProviderObjectScopeOptions,
  ): Promise<boolean>;
  copyObject(
    providerId: string,
    input: CopyStorageProviderObjectInput,
    options?: StorageProviderMutationOptions,
  ): Promise<StorageProviderObjectMutationResult>;
  renameObject(
    providerId: string,
    sourceObjectKey: string,
    destinationObjectKey: string,
    options?: StorageProviderObjectScopeOptions,
  ): Promise<boolean>;
  /**
   * The first page of account-center accounts.
   *
   * Convenience for a caller that renders one window and never pages; anything
   * that has to *find* an account uses
   * {@link StorageProviderAdminService.listProviderAccountsPage} and follows the
   * cursor it returns.
   */
  listProviderAccounts(
    input?: ListStorageProviderAccountsInput,
  ): Promise<StorageProviderAccountView[]>;
  /**
   * One page of account-center accounts, with the continuation the caller needs
   * to read the rest.
   *
   * The account list is paginated and the console shows it as a pickable list
   * scoped to one vendor, so the window has to be selected by the server from
   * that vendor's accounts and the cursor has to travel with it: reading a
   * single page as if it were the whole set hides every account that sorts past
   * the window — a freshly registered one included — and the picker then offers
   * "register a new account" for a vendor that already has one.
   */
  listProviderAccountsPage(
    input?: ListStorageProviderAccountsInput,
  ): Promise<ListStorageProviderAccountsPageResult>;
  createProviderAccount(
    input: CreateStorageProviderAccountInput,
    options?: StorageProviderMutationOptions,
  ): Promise<StorageProviderAccountView>;
}

export interface CreateStorageProviderAdminServiceOptions {
  adminStorageSdkClient: DriveAdminStorageSdkClient;
  getSession: () => SessionSnapshot;
}

export function createStorageProviderAdminService({
  adminStorageSdkClient,
  getSession,
}: CreateStorageProviderAdminServiceOptions): StorageProviderAdminService {
  const service: StorageProviderAdminService = {
    async getStorageOverview(input = {}) {
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageOverview.retrieve',
        signal: input.signal,
        query: {
          trendMonths: input.trendMonths,
        },
      });
      return responseToStorageOverview(response);
    },
    async listProviders(input = {}) {
      const page = await service.listProvidersPage(input);
      return page.items;
    },
    async listKinds(input = {}) {
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviderKinds.list',
        signal: input.signal,
      });
      return extractItems(response).map(responseToProviderKind);
    },
    async setKindEnabled(providerKind, enabled, options) {
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviderKinds.update',
        signal: options?.signal,
        pathParams: { providerKind },
        body: { enabled },
      });
      return responseToProviderKind(response);
    },
    async initializeKinds(options) {
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviderKinds.create',
        signal: options?.signal,
        body: {},
      });
      return extractItems(response).map(responseToProviderKind);
    },
    async initializeProviderAccountDefaults(options) {
      // 写操作要有可归属的操作者，与 createProviderAccount 同一前置条件。
      assertAdminWriteSession(getSession);
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviderAccountDefaults.create',
        signal: options?.signal,
      });
      return extractItems(response).map(responseToStorageProviderAccountDefault);
    },
    async listProvidersPage(input = {}) {
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviders.list',
        signal: input.signal,
        query: {
          // The kind filter travels to the server so it is applied *before* the
          // cursor window. Filtering the returned page instead made the
          // console's provider switch report "no rows" for a kind whose row sat
          // on the next page — see `StorageProvidersAdminPage`.
          //
          // `provider_kind` is the canonical lower_snake_case wire name for a
          // multi-word query parameter (`API_SPEC.md` §13), matching `page_size`
          // and `cursor` on this same request.
          provider_kind: input.providerKind,
          status: input.status,
          page_size: input.pageSize ?? 20,
          cursor: input.pageToken,
        },
      });
      const record = recordOf(response);
      const pageInfo = isRecord(record.pageInfo) ? record.pageInfo : {};
      const nextPageToken = stringField(pageInfo, 'nextCursor');
      const hasMore = booleanField(pageInfo, 'hasMore') ?? Boolean(nextPageToken);
      return {
        items: extractItems(response).map(responseToStorageProvider),
        nextPageToken,
        hasMore,
      };
    },
    async createProvider(input, options) {
      assertAdminWriteSession(getSession);
      const body = providerCreateBody(input);
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviders.create',
        signal: options?.signal,
        body,
      });
      return responseToStorageProvider(response);
    },
    async updateProvider(providerId, input, options) {
      assertAdminWriteSession(getSession);
      const body: JsonRecord = {};
      assignDefined(body, 'name', input.name);
      assignDefined(body, 'endpointUrl', input.endpointUrl);
      assignDefined(body, 'region', input.region);
      assignDefined(body, 'bucket', input.bucket);
      assignDefined(body, 'pathStyle', input.pathStyle);
      // strictTls must be forwarded: the create path already sends it, and the
      // edit form exposes a "强制 TLS" toggle. Omitting it here silently
      // reverted the operator's change to the server-side default.
      assignDefined(body, 'strictTls', input.strictTls);
      assignDefined(body, 'credentialRef', input.credentialRef);
      assignDefined(body, 'providerAccountId', input.providerAccountId);
      assignDefined(body, 'serverSideEncryptionMode', input.serverSideEncryptionMode);
      assignDefined(body, 'defaultStorageClass', input.defaultStorageClass);
      assignDefined(body, 'status', input.status);
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviders.update',
        signal: options?.signal,
        pathParams: { providerId },
        body,
      });
      return responseToStorageProvider(response);
    },
    async deleteProvider(providerId, options) {
      assertAdminWriteSession(getSession);
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviders.delete',
        signal: options?.signal,
        pathParams: { providerId },
      });
      // 契约返回 204 无内容：请求成功即视为已删除。
      return response === undefined || response === null
        || booleanField(resourceRecord(response), 'deleted') === true;
    },
    async testProvider(providerId, options) {
      assertAdminWriteSession(getSession);
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviders.test',
        signal: options?.signal,
        pathParams: { providerId },
        body: {},
      });
      // `storageProviders.test` answers the resource envelope too, so a flat
      // read would report every provider as unreachable.
      return booleanField(resourceRecord(response), 'reachable') ?? false;
    },
    async activateProvider(providerId, options) {
      assertAdminWriteSession(getSession);
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviders.activate',
        signal: options?.signal,
        pathParams: { providerId },
        body: {},
      });
      return responseToStorageProvider(response);
    },
    async deactivateProvider(providerId, options) {
      assertAdminWriteSession(getSession);
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviders.deactivate',
        signal: options?.signal,
        pathParams: { providerId },
        body: {},
      });
      return responseToStorageProvider(response);
    },
    async rotateCredential(providerId, credentialRef, options) {
      assertAdminWriteSession(getSession);
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviders.credentials.rotate',
        signal: options?.signal,
        pathParams: { providerId },
        body: {
          credentialRef,
        },
      });
      return responseToStorageProvider(response);
    },
    async getCapabilities(providerId, options) {
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviders.capabilities.list',
        signal: options?.signal,
        pathParams: { providerId },
      });
      return responseToCapabilities(response);
    },
    async headBucket(providerId, options) {
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviders.bucket.retrieve',
        signal: options?.signal,
        pathParams: { providerId },
      });
      const record = resourceRecord(response);
      return {
        providerId: stringField(record, 'providerId') ?? providerId,
        bucket: stringField(record, 'bucket') ?? '',
        exists: booleanField(record, 'exists') ?? false,
      };
    },
    async getDefaultBinding(spaceId, options) {
      try {
        const response = await adminStorageSdkClient.request<unknown>({
          operationId: 'storageProviderBindings.default.retrieve',
          signal: options?.signal,
          query: {
            spaceId,
          },
        });
        return responseToBinding(response);
      } catch (error) {
        if (
          error instanceof DriveAdminStorageSdkError
          && error.operationId === 'storageProviderBindings.default.retrieve'
          && error.status === 404
        ) {
          return undefined;
        }
        throw error;
      }
    },
    async setDefaultBinding(input) {
      assertAdminWriteSession(getSession);
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviderBindings.default.update',
        signal: input.signal,
        body: {
          providerId: input.providerId,
          spaceId: input.spaceId,
          spaceType: input.spaceType,
          storageRootPrefix: input.storageRootPrefix,
        },
      });
      return responseToBinding(response);
    },
    async deleteDefaultBinding(spaceIdOrSpaceType, options) {
      assertAdminWriteSession(getSession);
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviderBindings.default.delete',
        signal: options?.signal,
        query: options?.spaceType
          ? {
              spaceType: spaceIdOrSpaceType,
            }
          : {
              spaceId: spaceIdOrSpaceType,
            },
      });
      return booleanField(recordOf(response), 'deleted') ?? false;
    },
    async setSpaceTypeBinding(input) {
      return service.setDefaultBinding({
        providerId: input.providerId,
        spaceType: input.spaceType,
        storageRootPrefix: input.storageRootPrefix,
        signal: input.signal,
      });
    },
    async deleteSpaceTypeBinding(spaceType, options) {
      return service.deleteDefaultBinding(spaceType, { ...options, spaceType: true });
    },
    async listBindings(input = {}) {
      const page = await service.listBindingsPage(input);
      return page.items;
    },
    async listBindingsPage(input = {}) {
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviderBindings.list',
        signal: input.signal,
        query: {
          // `binding_scope` is the canonical lower_snake_case wire name for a
          // multi-word query parameter (`API_SPEC.md` §13), matching `page_size`
          // and `cursor` on this same request.
          binding_scope: input.bindingScope,
          providerId: input.providerId,
          spaceId: input.spaceId,
          lifecycleStatus: input.lifecycleStatus,
          page_size: input.pageSize ?? 20,
          cursor: input.pageToken,
        },
      });
      const record = recordOf(response);
      const pageInfo = isRecord(record.pageInfo) ? record.pageInfo : {};
      const nextPageToken = stringField(pageInfo, 'nextCursor');
      return {
        items: extractItems(response).map(responseToBinding),
        nextPageToken,
        hasMore: booleanField(pageInfo, 'hasMore') ?? Boolean(nextPageToken),
      };
    },
    async listBuckets(providerId, options) {
      // 厂商清单本来就是全量的（S3 ListBuckets 一次返回账号下的所有桶），接口却按
      // offset 分页：只读第一页会把窗口之外的桶藏起来，而调用方把返回值当成"这个账号
      // 里所有的桶"来展示——数量徽标、客户端搜索、以及"哪个桶是配置桶"的判定都基于这
      // 份清单。所以这里按声明上限逐页读完游标链，而不是只取一页。
      const buckets: StorageProviderBucketListItemView[] = [];
      // 同名桶只可能有一个：offset 窗口在并发写入后可能重叠，去重顺带保证表格 key 唯一。
      const seenBuckets = new Set<string>();
      // 游标如果原地打转，继续跟只会空转；记下走过的游标即可判定读完。
      const visitedCursors = new Set<string>();
      let pageToken: string | undefined;
      for (;;) {
        const response = await adminStorageSdkClient.request<unknown>({
          operationId: 'storageProviders.buckets.list',
          signal: options?.signal,
          pathParams: { providerId },
          query: {
            page_size: BUCKET_INVENTORY_PAGE_SIZE,
            cursor: pageToken,
          },
        });
        for (const item of extractItems(response)) {
          const bucket = responseToBucketListItem(item);
          if (bucket.bucket === '' || seenBuckets.has(bucket.bucket)) {
            continue;
          }
          seenBuckets.add(bucket.bucket);
          buckets.push(bucket);
        }
        const record = recordOf(response);
        const pageInfo = isRecord(record.pageInfo) ? record.pageInfo : {};
        const nextPageToken = stringField(pageInfo, 'nextCursor');
        if (!nextPageToken || visitedCursors.has(nextPageToken)) {
          return buckets;
        }
        visitedCursors.add(nextPageToken);
        pageToken = nextPageToken;
      }
    },
    async initializeBucket(providerId, options) {
      // storageProviders.bucket.update is an idempotent ensure on the backend:
      // it creates the configured bucket only when missing and reports
      // changed=false when it already exists, so re-running stays safe.
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviders.bucket.update',
        signal: options?.signal,
        pathParams: { providerId },
      });
      const record = resourceRecord(response);
      return {
        providerId: stringField(record, 'providerId') ?? providerId,
        bucket: stringField(record, 'bucket') ?? '',
        changed: booleanField(record, 'changed') ?? false,
      } satisfies StorageProviderBucketInitializeView;
    },
    async deleteBucket(providerId, options) {
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviders.bucket.delete',
        signal: options?.signal,
        pathParams: { providerId },
      });
      return booleanField(resourceRecord(response), 'changed') ?? false;
    },
    async listObjects(providerId, input = {}) {
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviders.objects.list',
        signal: input.signal,
        pathParams: { providerId },
        query: {
          // 桶随请求走：管理端浏览的是厂商账号下真实存在的桶，缺省才回落到配置里的桶。
          bucket: input.bucket || undefined,
          // 地域同样随请求走：账号级桶清单跨地域，点开的桶未必在配置写的地域里。
          region: input.region || undefined,
          prefix: input.prefix || undefined,
          delimiter: '/',
          page_size: input.pageSize ?? 100,
          cursor: input.pageToken || undefined,
        },
      });
      const record = recordOf(response);
      const pageInfo = isRecord(record.pageInfo) ? record.pageInfo : {};
      const items = extractItems(record).map((item) => objectRecordToView(recordOf(item)));
      const nextPageToken = stringField(pageInfo, 'nextCursor');
      return {
        items,
        nextPageToken,
        hasMore: booleanField(pageInfo, 'hasMore') ?? Boolean(nextPageToken),
      };
    },
    async deleteObject(providerId, objectKey, options) {
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviders.objects.delete',
        signal: options?.signal,
        pathParams: { providerId, objectKey },
        query: {
          bucket: options?.bucket || undefined,
          region: options?.region || undefined,
        },
      });
      // 契约返回 204 无内容：请求成功即视为已删除。
      return response === undefined || response === null
        || booleanField(resourceRecord(response), 'deleted') === true;
    },
    async readObjectContent(providerId, objectKey, options) {
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviders.objects.content.retrieve',
        signal: options?.signal,
        pathParams: { providerId, objectKey },
        query: {
          bucket: options?.bucket || undefined,
          region: options?.region || undefined,
        },
      });
      const record = resourceRecord(response);
      return {
        providerId: stringField(record, 'providerId') ?? providerId,
        bucket: stringField(record, 'bucket') ?? '',
        objectKey: stringField(record, 'objectKey', 'object_key') ?? objectKey,
        contentType: stringField(record, 'contentType', 'content_type'),
        sizeBytes: numberField(record, 'sizeBytes', 'size_bytes') ?? 0,
        encoding: 'base64',
        content: stringField(record, 'content') ?? '',
        checksumSha256: stringField(record, 'checksumSha256') ?? '',
      };
    },
    async writeObjectContent(providerId, objectKey, input, options) {
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviders.objects.content.update',
        signal: options?.signal,
        pathParams: { providerId, objectKey },
        query: {
          bucket: options?.bucket || undefined,
          region: options?.region || undefined,
        },
        body: {
          content: input.content,
          ...(input.encoding !== undefined ? { encoding: input.encoding } : {}),
          ...(input.contentType !== undefined ? { contentType: input.contentType } : {}),
        },
      });
      return objectRecordToView(resourceRecord(response));
    },
    async createMultipartUpload(providerId, input, options) {
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviders.objects.multipartUpload.create',
        signal: options?.signal,
        pathParams: { providerId },
        query: {
          bucket: options?.bucket || undefined,
          region: options?.region || undefined,
        },
        body: {
          objectKey: input.objectKey,
          ...(input.contentType !== undefined ? { contentType: input.contentType } : {}),
          ...(input.checksumSha256Hex !== undefined
            ? { checksumSha256Hex: input.checksumSha256Hex }
            : {}),
        },
      });
      const record = resourceRecord(response);
      const uploadId = stringField(record, 'uploadId', 'upload_id');
      if (!uploadId) {
        // 没有 uploadId 就没有后续任何一步：宁可在服务边界报错，也不要把空令牌传下去，
        // 否则失败会推迟到 complete 阶段，现场只剩"分片都对但合不起来"。
        throw new Error('multipart upload response is missing uploadId');
      }
      return {
        providerId: stringField(record, 'providerId') ?? providerId,
        bucket: stringField(record, 'bucket') ?? options?.bucket ?? '',
        objectKey: stringField(record, 'objectKey', 'object_key') ?? input.objectKey,
        uploadId,
      };
    },
    async presignUploadParts(providerId, input, options) {
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviders.objects.multipartUpload.parts.presign',
        signal: options?.signal,
        pathParams: { providerId },
        query: {
          bucket: options?.bucket || undefined,
          region: options?.region || undefined,
        },
        body: {
          objectKey: input.objectKey,
          uploadId: input.uploadId,
          partNumbers: input.partNumbers,
          ...(input.expiresInSeconds !== undefined
            ? { expiresInSeconds: input.expiresInSeconds }
            : {}),
        },
      });
      const record = resourceRecord(response);
      const rawParts = Array.isArray(record.parts) ? record.parts : [];
      return {
        providerId: stringField(record, 'providerId') ?? providerId,
        bucket: stringField(record, 'bucket') ?? options?.bucket ?? '',
        objectKey: stringField(record, 'objectKey', 'object_key') ?? input.objectKey,
        uploadId: stringField(record, 'uploadId', 'upload_id') ?? input.uploadId,
        parts: rawParts.map((part) => {
          const item = recordOf(part);
          const headers = isRecord(item.headers) ? item.headers : {};
          return {
            partNumber: numberField(item, 'partNumber', 'part_number') ?? 0,
            method: stringField(item, 'method') ?? 'PUT',
            url: stringField(item, 'url') ?? '',
            headers: Object.fromEntries(
              Object.entries(headers).map(([name, value]) => [name, String(value)]),
            ),
            // 契约把它声明为 int64 字符串；解析不出来就当厂商没给（界面不展示到期时间）。
            ...(numberField(item, 'expiresAtEpochMs', 'expires_at_epoch_ms') !== undefined
              ? { expiresAtEpochMs: numberField(item, 'expiresAtEpochMs', 'expires_at_epoch_ms') }
              : {}),
          };
        }),
      };
    },
    async completeMultipartUpload(providerId, input, options) {
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviders.objects.multipartUpload.complete',
        signal: options?.signal,
        pathParams: { providerId },
        query: {
          bucket: options?.bucket || undefined,
          region: options?.region || undefined,
        },
        body: {
          objectKey: input.objectKey,
          uploadId: input.uploadId,
          parts: input.parts,
        },
      });
      return objectRecordToView(resourceRecord(response));
    },
    async abortMultipartUpload(providerId, input, options) {
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviders.objects.multipartUpload.abort',
        signal: options?.signal,
        pathParams: { providerId },
        query: {
          bucket: options?.bucket || undefined,
          region: options?.region || undefined,
        },
        body: {
          objectKey: input.objectKey,
          uploadId: input.uploadId,
        },
      });
      return booleanField(resourceRecord(response), 'changed') ?? true;
    },
    async copyObject(providerId, input, options) {
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviders.objects.copy',
        signal: options?.signal,
        pathParams: { providerId },
        body: {
          sourceObjectKey: input.sourceObjectKey,
          destinationObjectKey: input.destinationObjectKey,
          ...(input.sourceBucket !== undefined ? { sourceBucket: input.sourceBucket } : {}),
          ...(input.destinationBucket !== undefined
            ? { destinationBucket: input.destinationBucket }
            : {}),
          ...(input.region !== undefined ? { region: input.region } : {}),
        },
      });
      const record = resourceRecord(response);
      return {
        providerId: stringField(record, 'providerId') ?? providerId,
        bucket: stringField(record, 'bucket') ?? '',
        objectKey: stringField(record, 'objectKey', 'object_key') ?? '',
        changed: booleanField(record, 'changed') ?? false,
      };
    },
    /**
     * 重命名/移动对象 = 服务端 copy + 源删除，非原子：
     * - copy 失败：源对象保持原样，无副作用；
     * - delete 失败：目标已复制成功但源残留（双份），调用方可重试删除源对象。
     */
    async renameObject(providerId, sourceObjectKey, destinationObjectKey, options) {
      await service.copyObject(
        providerId,
        {
          sourceObjectKey,
          destinationObjectKey,
          // 同桶改名：源与目标都锁在调用方指定的那个桶上，否则 copy 会落到配置桶、
          // delete 却删到目标桶，留下无从解释的残影。地域同理，否则复制会去配置地域的
          // 端点找一个不在那里的桶。
          ...(options?.bucket !== undefined
            ? { sourceBucket: options.bucket, destinationBucket: options.bucket }
            : {}),
          ...(options?.region !== undefined ? { region: options.region } : {}),
        },
        options,
      );
      return service.deleteObject(providerId, sourceObjectKey, options);
    },
    async listProviderAccounts(input = {}) {
      const page = await service.listProviderAccountsPage(input);
      return page.items;
    },
    async listProviderAccountsPage(input = {}) {
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviderAccounts.list',
        signal: input.signal,
        query: {
          vendorCode: input.vendorCode,
          status: input.status,
          search: input.search,
          scopeType: input.scopeType,
          ownerUserId: input.ownerUserId,
          mine: input.mine,
          includePlatform: input.includePlatform,
          capabilityCode: input.capabilityCode,
          // The window travels on the wire together with the continuation it
          // belongs to: a caller that follows `nextPageToken` has to name the
          // same `page_size`, or the cursor's offset and the window size
          // disagree. Both stay absent when the caller reads one page only, and
          // the server then applies its own default.
          page_size: input.pageSize,
          cursor: input.pageToken,
        },
      });
      const record = recordOf(response);
      const pageInfo = isRecord(record.pageInfo) ? record.pageInfo : {};
      const nextPageToken = stringField(pageInfo, 'nextCursor');
      return {
        items: extractItems(response).map(responseToStorageProviderAccount),
        nextPageToken,
        hasMore: booleanField(pageInfo, 'hasMore') ?? Boolean(nextPageToken),
      };
    },
    async createProviderAccount(input, options) {
      assertAdminWriteSession(getSession);
      const response = await adminStorageSdkClient.request<unknown>({
        operationId: 'storageProviderAccounts.create',
        signal: options?.signal,
        body: {
          displayName: input.displayName,
          vendorCode: input.vendorCode,
          accountCode: input.accountCode,
          ...(input.accountType !== undefined ? { accountType: input.accountType } : {}),
          ...(input.environment !== undefined ? { environment: input.environment } : {}),
          ...(input.externalAccountId !== undefined
            ? { externalAccountId: input.externalAccountId }
            : {}),
          ...(input.regionCode !== undefined ? { regionCode: input.regionCode } : {}),
          ...(input.scopeType !== undefined ? { scopeType: input.scopeType } : {}),
          ...(input.ownerUserId !== undefined ? { ownerUserId: input.ownerUserId } : {}),
          ...(input.isDefault !== undefined ? { isDefault: input.isDefault } : {}),
          accessKeyId: input.accessKeyId,
          secretAccessKey: input.secretAccessKey,
          ...(input.sessionToken !== undefined ? { sessionToken: input.sessionToken } : {}),
        },
      });
      return responseToStorageProviderAccount(response);
    },
  };

  return service;
}

/**
 * Write operations require an identified operator.
 *
 * The storage backend authorises from the dual-token session rather than from
 * body fields, so this is a client-side precondition and not a request
 * projection: it fails fast when a host mounted the surface with a session that
 * was never fully hydrated, instead of sending a write the server will reject.
 *
 * It deliberately returns nothing — a projected `{ tenantId, operatorId }` used
 * to be threaded into request bodies and was removed when identity projection
 * moved into the SDK transport.
 */
function assertAdminWriteSession(getSession: () => SessionSnapshot): void {
  const context = getSession().context;
  if (!context?.tenantId || !context.actorId) {
    throw new Error('Drive admin session context is missing tenantId or operatorId.');
  }
}

function providerCreateBody(input: CreateStorageProviderInput): JsonRecord {
  const body: JsonRecord = {
    id: input.id,
    providerKind: input.providerKind,
    name: input.name,
    endpointUrl: input.endpointUrl,
    bucket: input.bucket,
  };
  assignDefined(body, 'region', input.region);
  assignDefined(body, 'pathStyle', input.pathStyle);
  assignDefined(body, 'credentialRef', input.credentialRef);
  assignDefined(body, 'providerAccountId', input.providerAccountId);
  assignDefined(body, 'serverSideEncryptionMode', input.serverSideEncryptionMode);
  assignDefined(body, 'defaultStorageClass', input.defaultStorageClass);
  assignDefined(body, 'status', input.status);
  assignDefined(body, 'strictTls', input.strictTls);
  return body;
}

function responseToStorageProvider(response: unknown): StorageProviderView {
  const record = resourceRecord(response);
  const name = stringField(record, 'name', 'displayName') ?? '';
  return {
    id: stringField(record, 'id', 'providerId') ?? '',
    providerKind: stringField(record, 'providerKind', 'kind') ?? '',
    displayName: name,
    endpointUrl: stringField(record, 'endpointUrl', 'endpoint') ?? '',
    region: stringField(record, 'region'),
    bucket: stringField(record, 'bucket') ?? '',
    pathStyle: booleanField(record, 'pathStyle') ?? false,
    credentialRef: stringField(record, 'credentialRef'),
    providerAccountId: stringField(record, 'providerAccountId'),
    credentialConfigured: booleanField(record, 'credentialConfigured') ?? false,
    serverSideEncryptionMode: stringField(record, 'serverSideEncryptionMode'),
    defaultStorageClass: stringField(record, 'defaultStorageClass'),
    status: stringField(record, 'status') ?? 'unknown',
    version: numberField(record, 'version') ?? 0,
    strictTls: booleanField(record, 'strictTls') ?? true,
  };
}

/** 账号中心账号投影：列表响应为 items 数组，创建响应为 { item }。 */
function responseToStorageProviderAccount(response: unknown): StorageProviderAccountView {
  const record = isRecord(response) && isRecord(response.item) ? response.item : recordOf(response);
  return {
    id: stringField(record, 'id') ?? '',
    scopeType: accountScopeField(record),
    ownerUserId: stringField(record, 'ownerUserId', 'owner_user_id'),
    isDefault: booleanField(record, 'isDefault', 'is_default') ?? false,
    vendorCode: stringField(record, 'vendorCode', 'vendor_code') ?? '',
    accountCode: stringField(record, 'accountCode', 'account_code') ?? '',
    displayName: stringField(record, 'displayName', 'display_name') ?? '',
    // Identity shape, not a relationship label: the account centre's vocabulary is
    // long_term_key | temporary_credential | service_account | service_linked_role |
    // federated_identity | managed_identity | api_key. The storage console only ever
    // stores an access-key pair, so `long_term_key` is the shape a missing value
    // stands for.
    accountType: stringField(record, 'accountType', 'account_type') ?? 'long_term_key',
    environment: stringField(record, 'environment') ?? 'production',
    externalAccountId: stringField(record, 'externalAccountId', 'external_account_id'),
    capabilityCodes: stringArrayField(record, 'capabilityCodes', 'capability_codes'),
    regionCode: stringField(record, 'regionCode', 'region_code'),
    status: stringField(record, 'status') ?? 'unknown',
    credentialConfigured: booleanField(record, 'credentialConfigured') ?? false,
    credentialCount: numberField(record, 'credentialCount', 'credential_count') ?? 0,
    version: numberField(record, 'version') ?? 0,
  };
}

/**
 * 作用域投影带上兜底。
 *
 * 服务端默认 `tenant`（`DEFAULT_ACCOUNT_SCOPE`），历史行也是 `tenant`；读到未知值时
 * 收敛到 `tenant`，避免视图层对一个无法解释的字符串做 switch 而走到 undefined 分支。
 */
function accountScopeField(record: Record<string, unknown>): StorageProviderAccountScope {
  const raw = stringField(record, 'scopeType', 'scope_type')?.toLowerCase();
  return raw === 'platform' || raw === 'user' ? raw : 'tenant';
}

/**
 * 内置服务商初始化结果投影。
 *
 * 两个布尔字段是这一行存在的理由：`accountCreated` 与 `credentialSeeded` 同时为
 * `false` 表示「本来就在，本run 没有再动它」——即运维已经填过的真实密钥不会被覆盖。
 */
function responseToStorageProviderAccountDefault(
  response: unknown,
): StorageProviderAccountDefaultView {
  const record = recordOf(response);
  return {
    providerKind: stringField(record, 'providerKind', 'provider_kind') ?? '',
    providerId: stringField(record, 'providerId', 'provider_id') ?? '',
    providerCreated: booleanField(record, 'providerCreated', 'provider_created') ?? false,
    vendorCode: stringField(record, 'vendorCode', 'vendor_code'),
    providerAccountId: stringField(record, 'providerAccountId', 'provider_account_id'),
    accountCode: stringField(record, 'accountCode', 'account_code'),
    accountCreated: booleanField(record, 'accountCreated', 'account_created') ?? false,
    credentialSeeded: booleanField(record, 'credentialSeeded', 'credential_seeded') ?? false,
    credentialFields: responseToVendorCredentialFields(record.credentialFields),
    vendorCapabilities: responseToVendorCapabilityDefaults(record.vendorCapabilities),
  };
}

/**
 * The vendor credential vocabulary the server wrote alongside the placeholder
 * key pair.
 *
 * Read with a tolerant shape check rather than a blind cast: the field is
 * optional in the response (a credential-free kind has none), and a version
 * skew that dropped it must degrade to `undefined` — the console then falls
 * back to its own static catalog — instead of handing the editor a half-built
 * object with empty labels.
 */
function responseToVendorCredentialFields(
  value: unknown,
): StorageProviderVendorCredentialFields | undefined {
  const record = recordOf(value);
  const accessKeyLabel = stringField(record, 'accessKeyLabel', 'access_key_label');
  const secretKeyLabel = stringField(record, 'secretKeyLabel', 'secret_key_label');
  const defaultEnvAccessKey = stringField(record, 'defaultEnvAccessKey', 'default_env_access_key');
  const defaultEnvSecretKey = stringField(record, 'defaultEnvSecretKey', 'default_env_secret_key');
  const consoleUrl = stringField(record, 'consoleUrl', 'console_url');
  if (
    !accessKeyLabel ||
    !secretKeyLabel ||
    !defaultEnvAccessKey ||
    !defaultEnvSecretKey ||
    !consoleUrl
  ) {
    return undefined;
  }
  return { accessKeyLabel, secretKeyLabel, defaultEnvAccessKey, defaultEnvSecretKey, consoleUrl };
}

/**
 * The vendor's encryption-mode / storage-class vocabulary, as returned by the
 * bootstrap.
 *
 * Unlike the credential labels these two lists may legitimately be empty (a
 * vendor that exposes no tier choice), so an absent field and an empty list are
 * different answers: absent means "no opinion, keep the console's own list",
 * empty means "this vendor offers nothing". Both are preserved rather than
 * collapsed, so the editor's `has*` flags stay honest.
 */
function responseToVendorCapabilityDefaults(
  value: unknown,
): StorageProviderVendorCapabilityDefaults | undefined {
  const record = recordOf(value);
  if (Object.keys(record).length === 0) {
    return undefined;
  }
  return {
    serverSideEncryptionModes: stringArrayField(
      record,
      'serverSideEncryptionModes',
      'server_side_encryption_modes',
    ),
    storageClasses: stringArrayField(record, 'storageClasses', 'storage_classes'),
  };
}

function responseToProviderKind(response: unknown): StorageProviderKindView {
  const record = resourceRecord(response);
  return {
    providerKind: stringField(record, 'providerKind', 'provider_kind') ?? '',
    displayName: stringField(record, 'displayName', 'display_name') ?? '',
    enabled: booleanField(record, 'enabled') ?? false,
    sortOrder: numberField(record, 'sortOrder', 'sort_order') ?? 0,
    version: numberField(record, 'version') ?? 0,
    configCount: numberField(record, 'configCount', 'config_count') ?? 0,
  };
}

/**
 * 概览响应 → 视图模型。
 *
 * 所有计数与字节数在契约里是 int64 字符串（`numberField` 会做字符串→数字
 * 兜底），所以这里的投影对两种形态都成立，不会因为后端序列化差异而变成 NaN。
 */
function responseToStorageOverview(response: unknown): StorageOverviewView {
  const record = isRecord(response) && isRecord(response.item) ? response.item : recordOf(response);
  const capacity = recordOf(record.capacity);
  const providers = recordOf(record.providers);
  const bindings = recordOf(record.bindings);
  const byScope = recordOf(bindings.byScope);
  const catalog = recordOf(record.catalog);
  const trend = Array.isArray(record.trend) ? record.trend : [];

  return {
    generatedAt: stringField(record, 'generatedAt') ?? '',
    scopeTenantId: stringField(record, 'scopeTenantId') ?? '',
    capacity: {
      totalObjectCount: numberField(capacity, 'totalObjectCount') ?? 0,
      activeObjectCount: numberField(capacity, 'activeObjectCount') ?? 0,
      deletedObjectCount: numberField(capacity, 'deletedObjectCount') ?? 0,
      usedBytes: numberField(capacity, 'usedBytes') ?? 0,
      averageObjectBytes: numberField(capacity, 'averageObjectBytes') ?? 0,
      largestObjectBytes: numberField(capacity, 'largestObjectBytes'),
      bucketCount: numberField(capacity, 'bucketCount') ?? 0,
      quotaBytes: numberField(capacity, 'quotaBytes'),
      quotaConfigured: booleanField(capacity, 'quotaConfigured') ?? false,
      quotaUsageRatio: numberField(capacity, 'quotaUsageRatio'),
    },
    providers: {
      totalCount: numberField(providers, 'totalCount') ?? 0,
      activeCount: numberField(providers, 'activeCount') ?? 0,
      disabledCount: numberField(providers, 'disabledCount') ?? 0,
      deletedCount: numberField(providers, 'deletedCount') ?? 0,
      usage: (Array.isArray(providers.usage) ? providers.usage : []).map((row) => {
        const usage = recordOf(row);
        return {
          providerId: stringField(usage, 'providerId') ?? '',
          name: stringField(usage, 'name') ?? '',
          providerKind: stringField(usage, 'providerKind') ?? '',
          status: stringField(usage, 'status') ?? 'unknown',
          bucket: stringField(usage, 'bucket') ?? '',
          objectCount: numberField(usage, 'objectCount') ?? 0,
          usedBytes: numberField(usage, 'usedBytes') ?? 0,
          bindingCount: numberField(usage, 'bindingCount') ?? 0,
          isTenantDefault: booleanField(usage, 'isTenantDefault') ?? false,
          capacityShare: numberField(usage, 'capacityShare') ?? 0,
        };
      }),
    },
    bindings: {
      totalCount: numberField(bindings, 'totalCount') ?? 0,
      activeCount: numberField(bindings, 'activeCount') ?? 0,
      inactiveCount: numberField(bindings, 'inactiveCount') ?? 0,
      byScope: {
        tenantCount: numberField(byScope, 'tenantCount') ?? 0,
        spaceCount: numberField(byScope, 'spaceCount') ?? 0,
        spaceTypeCount: numberField(byScope, 'spaceTypeCount') ?? 0,
      },
      hasTenantDefault: booleanField(bindings, 'hasTenantDefault') ?? false,
      tenantDefaultBindingId: stringField(bindings, 'tenantDefaultBindingId'),
      tenantDefaultProviderId: stringField(bindings, 'tenantDefaultProviderId'),
    },
    catalog: {
      totalCount: numberField(catalog, 'totalCount') ?? 0,
      enabledCount: numberField(catalog, 'enabledCount') ?? 0,
      disabledCount: numberField(catalog, 'disabledCount') ?? 0,
    },
    trend: trend.map((point) => {
      const item = recordOf(point);
      return {
        periodLabel: stringField(item, 'periodLabel') ?? '',
        objectCount: numberField(item, 'objectCount') ?? 0,
        bytes: numberField(item, 'bytes') ?? 0,
      };
    }),
  };
}

function responseToCapabilities(response: unknown): StorageProviderCapabilitiesView {
  const record = resourceRecord(response);
  return {
    providerId: stringField(record, 'providerId') ?? '',
    providerKind: stringField(record, 'providerKind') ?? '',
    supportsMultipartUpload: booleanField(record, 'supportsMultipartUpload') ?? false,
    supportsPresignedUploadPart: booleanField(record, 'supportsPresignedUploadPart') ?? false,
    supportsPresignedDownload: booleanField(record, 'supportsPresignedDownload') ?? false,
    supportsServerSideEncryption: booleanField(record, 'supportsServerSideEncryption') ?? false,
    supportsStorageClass: booleanField(record, 'supportsStorageClass') ?? false,
    supportsCredentialRotation: booleanField(record, 'supportsCredentialRotation') ?? false,
    supportedServerSideEncryptionModes: stringArrayField(record, 'supportedServerSideEncryptionModes'),
    supportedStorageClasses: stringArrayField(record, 'supportedStorageClasses'),
  };
}

function responseToBinding(response: unknown): StorageProviderBindingView {
  // A single binding arrives as `{ item }` (retrieve/upsert); a listed binding is
  // a bare object inside `items`. Both shapes project through the same reader.
  const record = resourceRecord(response);
  const storageProvider = record.storageProvider;
  return {
    id: stringField(record, 'id') ?? '',
    tenantId: stringField(record, 'tenantId') ?? '',
    spaceId: stringField(record, 'spaceId'),
    providerId: stringField(record, 'providerId') ?? '',
    bindingScope: stringField(record, 'bindingScope') ?? '',
    purpose: stringField(record, 'purpose') ?? '',
    lifecycleStatus: stringField(record, 'lifecycleStatus') ?? '',
    version: numberField(record, 'version') ?? 0,
    storageRootPrefix: stringField(record, 'storageRootPrefix'),
    storageProvider: isRecord(storageProvider) ? responseToStorageProvider(storageProvider) : undefined,
  };
}

/** 厂商桶清单条目（list 的一行）→ 视图模型。 */
function responseToBucketListItem(item: unknown): StorageProviderBucketListItemView {
  const record = recordOf(item);
  const creationDateEpochMs = numberField(record, 'creationDateEpochMs');
  return {
    bucket: stringField(record, 'bucket') ?? '',
    configured: booleanField(record, 'configured') ?? false,
    // 时间原样带走（ISO），由知道宿主语言的显示层格式化：服务层一旦调
    // `toLocaleDateString()`，读到的是浏览器区域而不是控制台语言。
    creationDateIso: creationDateEpochMs
      ? new Date(creationDateEpochMs).toISOString()
      : undefined,
    // 地域既是"所属地域"这一列，也是后续读写这个桶时该用哪个端点的依据。
    region: stringField(record, 'region'),
  } satisfies StorageProviderBucketListItemView;
}

/** 对象条目响应（list / content write）→ 视图模型。 */
function objectRecordToView(record: JsonRecord): StorageProviderObjectView {
  const objectKey = stringField(record, 'objectKey', 'object_key', 'key') ?? '';
  const objectKind = stringField(record, 'objectKind', 'object_kind');
  const contentLength = numberField(record, 'contentLength', 'content_length', 'sizeBytes') ?? 0;
  const lastModifiedEpochMs = numberField(record, 'lastModifiedEpochMs');
  return {
    key: objectKey,
    sizeBytes: contentLength,
    contentType: stringField(record, 'contentType', 'content_type'),
    etag: stringField(record, 'etag'),
    // 同上：ISO 交给显示层，避免"本地化字符串再被 Intl 解析一次"的双重格式化。
    lastModifiedIso: lastModifiedEpochMs
      ? new Date(lastModifiedEpochMs).toISOString()
      : undefined,
    isFolder: objectKind === 'prefix',
  };
}

function extractItems(response: unknown): unknown[] {
  if (Array.isArray(response)) {
    return response;
  }
  const record = recordOf(response);
  return Array.isArray(record.items) ? record.items : [];
}

function recordOf(value: unknown): JsonRecord {
  return isRecord(value) ? value : {};
}

/**
 * Unwrap a single-resource response body.
 *
 * The SDKWork envelope is `{ data: ... }` and the transport already returns
 * `data`, so every 2xx body for one resource is `{ item: {...} }`
 * (`API_SPEC.md` §15.1.1). A projection that reads the fields off the outer
 * object therefore reads `undefined` for all of them — silently, because the
 * types are structural. This helper accepts the item envelope and a bare object,
 * which is what nested projections (a provider inside a binding) already are.
 */
function resourceRecord(value: unknown): JsonRecord {
  const record = recordOf(value);
  return isRecord(record.item) ? record.item : record;
}

function isRecord(value: unknown): value is JsonRecord {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function stringField(source: JsonRecord, ...keys: string[]): string | undefined {
  for (const key of keys) {
    const value = source[key];
    if (typeof value === 'string' && value.trim() !== '') {
      return value;
    }
  }
  return undefined;
}

function numberField(source: JsonRecord, ...keys: string[]): number | undefined {
  for (const key of keys) {
    const value = source[key];
    if (typeof value === 'number' && Number.isFinite(value)) {
      return value;
    }
    if (typeof value === 'string' && Number.isFinite(Number(value))) {
      return Number(value);
    }
  }
  return undefined;
}

function booleanField(source: JsonRecord, ...keys: string[]): boolean | undefined {
  for (const key of keys) {
    const value = source[key];
    if (typeof value === 'boolean') {
      return value;
    }
  }
  return undefined;
}

function stringArrayField(source: JsonRecord, ...keys: string[]): string[] {
  for (const key of keys) {
    const value = source[key];
    if (Array.isArray(value)) {
      return value.filter((item): item is string => typeof item === 'string');
    }
  }
  return [];
}

function assignDefined(target: JsonRecord, key: string, value: unknown): void {
  if (value !== undefined) {
    target[key] = value;
  }
}
