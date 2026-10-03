use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::error::{ProvideErrorMetadata, SdkError};
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
use aws_sdk_s3::{Client, Config};
use aws_types::region::Region;
use futures_util::stream::{self, StreamExt};
use sdkwork_drive_storage_contract::{
    AbortMultipartUploadRequest, CompleteMultipartUploadRequest, CompleteMultipartUploadResponse,
    CopyObjectRequest, CopyObjectResponse, CreateBucketRequest, CreateBucketResponse,
    CreateMultipartUploadRequest, CreateMultipartUploadResponse, DeleteBucketRequest,
    DeleteBucketResponse, DeleteObjectRequest, DeleteObjectResponse, DriveObjectChunkStream,
    DriveObjectHeaders, DriveObjectStore, DriveObjectStoreError, DriveObjectStoreErrorKind,
    DriveStorageProviderCapabilities, DriveStorageProviderKind, HeadBucketRequest,
    HeadBucketResponse, HeadObjectRequest, HeadObjectResponse, ListBucketsRequest,
    ListBucketsResponse, ListObjectsRequest, ListObjectsResponse, ListedBucket, ListedObject,
    PresignDownloadRequest, PresignUploadPartRequest, PresignedDownloadResponse,
    PresignedUploadPartResponse, PutObjectFromPathRequest, PutObjectRequest, PutObjectResponse,
    ReadObjectRangeRequest, ReadObjectRangeResponse,
};
use std::collections::BTreeMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::config::S3StoreConfig;
use crate::inventory_region::{InventoryHttpClient, InventoryRegionCapture};

/// How many buckets one inventory read may ask about at the same time.
///
/// Vendors that leave the region out of the account inventory are answered one
/// bucket at a time (see [`S3DriveObjectStore::lookup_bucket_region`]), so the
/// fan-out is bounded: an account with hundreds of buckets must not turn one
/// list into hundreds of simultaneous calls.
const BUCKET_REGION_LOOKUP_CONCURRENCY: usize = 8;

/// Ceiling for one bucket's region lookup.
///
/// The region is enrichment and the row is not: a vendor that stalls here loses
/// its region for that row instead of holding the whole inventory.
const BUCKET_REGION_LOOKUP_TIMEOUT: Duration = Duration::from_secs(3);

/// Normalize one vendor's `GetBucketLocation` answer into a region code.
///
/// S3 answers `us-east-1` with an *empty* `LocationConstraint`, and a vendor
/// that cannot answer may send the literal `null`. Neither is a region name, so
/// neither is written into a row: an empty 所属地域 column is honest, a row
/// claiming the bucket lives in a region named `""` or `null` is not.
#[must_use]
fn normalize_bucket_location(reported: Option<&str>) -> Option<String> {
    let reported = reported?.trim();
    if reported.is_empty() || reported.eq_ignore_ascii_case("null") {
        return None;
    }
    Some(reported.to_string())
}

/// Credentials every client of one store shares.
fn provider_credentials(config: &S3StoreConfig) -> Credentials {
    Credentials::new(
        config.access_key_id.clone(),
        config.secret_access_key.clone(),
        config.session_token.clone(),
        None,
        "sdkwork-drive-storage-s3",
    )
}

/// Build a client for one endpoint, keeping region, addressing style, and
/// credentials identical to the store's primary client.
fn build_client(
    shared_config: &aws_config::SdkConfig,
    config: &S3StoreConfig,
    endpoint: Option<&str>,
) -> Client {
    Client::from_conf(build_client_config(shared_config, config, endpoint))
}

/// The same client configuration, before it is turned into a client.
///
/// Split out so the inventory client can be that configuration plus the
/// body-capturing HTTP layer: one description of a client, two clients.
fn build_client_config(
    shared_config: &aws_config::SdkConfig,
    config: &S3StoreConfig,
    endpoint: Option<&str>,
) -> Config {
    let mut builder = Config::from(shared_config)
        .to_builder()
        .force_path_style(config.force_path_style)
        .region(Region::new(config.region.clone()))
        .credentials_provider(provider_credentials(config));
    if let Some(endpoint) = endpoint {
        builder = builder.endpoint_url(endpoint);
    }
    builder.build()
}

#[derive(Debug, Clone)]
pub struct S3DriveObjectStore {
    client: Client,
    /// Second client addressed to the vendor's account-level service host, for
    /// the vendors that answer the bucket inventory on a host other than the
    /// regional object endpoint. `None` for every provider whose configured
    /// endpoint already serves the inventory.
    service_client: Option<Client>,
    /// The two clients the account inventory is read through. They differ from
    /// the object clients by one thing: their HTTP layer keeps a copy of the
    /// response body, which is where a vendor that does not use AWS's
    /// `<BucketRegion>` element publishes each bucket's region. See
    /// [`crate::inventory_region`].
    inventory_service_client: Option<Client>,
    inventory_client: Client,
    inventory_capture: InventoryRegionCapture,
    config: S3StoreConfig,
}

#[derive(Debug)]
struct SingleChunkStream {
    body: Option<Vec<u8>>,
}

#[async_trait]
impl DriveObjectChunkStream for SingleChunkStream {
    async fn next_chunk(&mut self) -> Result<Option<Vec<u8>>, DriveObjectStoreError> {
        Ok(self.body.take())
    }
}

impl S3DriveObjectStore {
    pub async fn new(config: S3StoreConfig) -> Result<Self, DriveObjectStoreError> {
        config.validate()?;

        let mut loader = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(config.region.clone()))
            .credentials_provider(provider_credentials(&config));
        if let Some(endpoint) = config.endpoint.as_ref() {
            loader = loader.endpoint_url(endpoint);
        }
        let shared_config = loader.load().await;

        let client = build_client(&shared_config, &config, config.endpoint.as_deref());
        // 账号级清单（`ListBuckets`）在部分厂商走服务级域名，而配置里存的是对象操作要用的
        // 地域域名：用地域域名读清单，回答的是该地域的桶，于是一个跨地域的账号在管理端只
        // 剩一个桶。见 `S3ProviderProfile::service_endpoint`。
        let inventory_endpoint = config.bucket_inventory_endpoint();
        let inventory_service_endpoint = inventory_endpoint
            .filter(|endpoint| Some(*endpoint) != config.endpoint.as_deref());
        let service_client = inventory_service_endpoint
            .map(|endpoint| build_client(&shared_config, &config, Some(endpoint)));

        /*
         * 清单客户端与对象客户端分开：清单客户端的 HTTP 层多包一层，把响应体留一份给自己解析
         * 厂商写在 `<Location>` 里的地域（AWS 的模型里没有这个元素，SDK 会丢掉它）。底座用的是
         * SDK 自己那套默认连接器（hyper 1.x + rustls 0.23），对象读写走 `client`，从不为这个
         * 副本付出代价。见 `crate::inventory_region`。
         */
        let inventory_capture = InventoryRegionCapture::new();
        let inventory_config = |endpoint: Option<&str>| {
            build_client_config(&shared_config, &config, endpoint)
                .to_builder()
                .http_client(InventoryHttpClient::new(inventory_capture.clone()))
                .build()
        };
        let inventory_client = Client::from_conf(inventory_config(config.endpoint.as_deref()));
        let inventory_service_client = inventory_service_endpoint
            .map(|endpoint| Client::from_conf(inventory_config(Some(endpoint))));

        Ok(Self {
            client,
            service_client,
            inventory_service_client,
            inventory_client,
            inventory_capture,
            config,
        })
    }

    /// Endpoint the account bucket inventory is read through.
    ///
    /// Exposed so the vendor split this type relies on — regional endpoint for
    /// objects, service endpoint for the inventory — stays assertable without a
    /// network round trip.
    #[must_use]
    pub fn bucket_inventory_endpoint(&self) -> Option<&str> {
        if self.service_client.is_some() {
            self.config.provider_profile.service_endpoint()
        } else {
            self.config.endpoint.as_deref()
        }
    }

    /// Read the account bucket inventory from the configured endpoint.
    ///
    /// The fallback path for [`S3DriveObjectStore::list_buckets`]: either the
    /// store has no separate service host, or the service host did not answer
    /// with an inventory a private network / a differently scoped account can
    /// use.
    async fn list_buckets_from_configured_endpoint(
        &self,
    ) -> Result<aws_sdk_s3::operation::list_buckets::ListBucketsOutput, DriveObjectStoreError> {
        self.inventory_client
            .list_buckets()
            .send()
            .await
            .map_err(|error| Self::map_sdk_error(error, "list buckets failed"))
    }

    /// Ask the vendor where each of these buckets lives.
    ///
    /// The *last* of the three channels, after the SDK's own `<BucketRegion>` and
    /// the inventory body's `<Location>` (see [`crate::inventory_region`]). It
    /// only resolves a bucket that lives in the region its endpoint belongs to:
    /// the operation is bucket-scoped, and a cross-region ask is answered
    /// `404 NoSuchBucket` — probed against `cos.ap-guangzhou.myqcloud.com` for an
    /// `ap-beijing` bucket — with no redirect and no region hint to follow. It
    /// therefore exists for vendors that publish nothing on the inventory at all;
    /// a bucket it cannot reach keeps `None`, which the console renders as an
    /// empty cell rather than a guessed region.
    ///
    /// Best effort by design: bounded, timed, and never fatal. A bucket whose
    /// region cannot be read keeps `None`.
    async fn lookup_bucket_regions(&self, buckets: &[String]) -> Vec<(String, Option<String>)> {
        stream::iter(buckets.iter().cloned())
            .map(|bucket| async move {
                let region = self.lookup_bucket_region(&bucket).await;
                (bucket, region)
            })
            .buffer_unordered(BUCKET_REGION_LOOKUP_CONCURRENCY)
            .collect()
            .await
    }

    /// One bucket's region, or `None` when the vendor will not say.
    async fn lookup_bucket_region(&self, bucket: &str) -> Option<String> {
        let request = self.client.get_bucket_location().bucket(bucket).send();
        let output = match tokio::time::timeout(BUCKET_REGION_LOOKUP_TIMEOUT, request).await {
            Ok(Ok(output)) => output,
            // Denied, unsupported, or slower than the budget: the row keeps no
            // region rather than a guess, and the inventory still returns.
            Ok(Err(_)) | Err(_) => return None,
        };
        normalize_bucket_location(output.location_constraint().map(|value| value.as_str()))
    }

    fn resolve_bucket(&self, requested_bucket: &str) -> Result<String, DriveObjectStoreError> {
        self.config.resolve_bucket(requested_bucket)
    }

    fn validate_object_key(object_key: &str) -> Result<(), DriveObjectStoreError> {
        if object_key != object_key.trim() {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "object_key must be trimmed",
            ));
        }
        if object_key.is_empty() {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "object_key must not be empty",
            ));
        }
        if object_key.len() > 1024 {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "object_key must be at most 1024 UTF-8 bytes",
            ));
        }
        if object_key.as_bytes().contains(&0) {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "object_key must not contain NUL bytes",
            ));
        }
        if object_key.starts_with('/') || object_key.ends_with('/') {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "object_key must be a relative key without leading or trailing slash",
            ));
        }
        for segment in object_key.split('/') {
            if segment.is_empty() || segment == "." || segment == ".." {
                return Err(DriveObjectStoreError::new(
                    DriveObjectStoreErrorKind::InvalidRequest,
                    "object_key must not contain empty or period-only path segments",
                ));
            }
        }
        Ok(())
    }

    fn validate_locator(
        locator: &sdkwork_drive_storage_contract::DriveObjectLocator,
    ) -> Result<(), DriveObjectStoreError> {
        Self::validate_object_key(&locator.object_key)
    }

    fn validate_presign_expiry(expires_in_seconds: u32) -> Result<(), DriveObjectStoreError> {
        if expires_in_seconds == 0 {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "expires_in_seconds must be greater than zero",
            ));
        }
        Ok(())
    }

    fn normalize_list_prefix(
        prefix: Option<String>,
    ) -> Result<Option<String>, DriveObjectStoreError> {
        let Some(prefix) = prefix else {
            return Ok(None);
        };
        if prefix.trim().is_empty() {
            return Ok(None);
        }
        if prefix != prefix.trim() {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "prefix must be trimmed",
            ));
        }
        if prefix.len() > 1024 {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "prefix must be at most 1024 UTF-8 bytes",
            ));
        }
        if prefix.as_bytes().contains(&0) {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "prefix must not contain NUL bytes",
            ));
        }
        if prefix.starts_with('/') {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "prefix must not start with slash",
            ));
        }
        let path_prefix = prefix.strip_suffix('/').unwrap_or(prefix.as_str());
        for segment in path_prefix.split('/') {
            if segment.is_empty() || segment == "." || segment == ".." {
                return Err(DriveObjectStoreError::new(
                    DriveObjectStoreErrorKind::InvalidRequest,
                    "prefix must not contain empty or period-only path segments",
                ));
            }
        }
        Ok(Some(prefix))
    }

    fn normalize_list_delimiter(
        delimiter: Option<String>,
    ) -> Result<Option<String>, DriveObjectStoreError> {
        let Some(delimiter) = delimiter else {
            return Ok(None);
        };
        if delimiter.trim().is_empty() {
            return Ok(None);
        }
        if delimiter != delimiter.trim() {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "delimiter must be trimmed",
            ));
        }
        if delimiter != "/" {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "delimiter must be '/' when provided",
            ));
        }
        Ok(Some(delimiter))
    }

    fn metadata_to_btree(
        source: Option<&std::collections::HashMap<String, String>>,
    ) -> BTreeMap<String, String> {
        match source {
            Some(values) => values
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            None => BTreeMap::new(),
        }
    }

    fn headers_from_presigned(
        request: &aws_sdk_s3::presigning::PresignedRequest,
    ) -> DriveObjectHeaders {
        request
            .headers()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }

    fn expires_at_epoch_ms(expires_in_seconds: u32) -> i64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis() as i64)
            .unwrap_or(0);
        now + i64::from(expires_in_seconds) * 1000
    }

    fn map_sdk_error<E>(error: SdkError<E>, default_message: &str) -> DriveObjectStoreError
    where
        E: ProvideErrorMetadata,
    {
        let code = error
            .as_service_error()
            .and_then(ProvideErrorMetadata::code)
            .unwrap_or_default()
            .to_ascii_lowercase();

        let kind = if code.contains("notfound")
            || code.contains("nosuchkey")
            || code.contains("nosuchupload")
        {
            DriveObjectStoreErrorKind::NotFound
        } else if code.contains("invalid") || code.contains("malformed") {
            DriveObjectStoreErrorKind::InvalidRequest
        } else if code.contains("conflict")
            || code.contains("preconditionfailed")
            || code.contains("alreadyexists")
        {
            DriveObjectStoreErrorKind::Conflict
        } else if code.contains("slowdown")
            || code.contains("toomanyrequests")
            || code.contains("throttl")
        {
            DriveObjectStoreErrorKind::RateLimited
        } else if code.contains("accessdenied") || code.contains("forbidden") {
            DriveObjectStoreErrorKind::PermissionDenied
        } else if code.contains("timeout") || code.contains("requesttimeout") {
            DriveObjectStoreErrorKind::Timeout
        } else if code.contains("unavailable")
            || code.contains("serviceunavailable")
            || code.contains("temporarilyunavailable")
        {
            DriveObjectStoreErrorKind::Unavailable
        } else {
            match &error {
                SdkError::ServiceError(_) => DriveObjectStoreErrorKind::UpstreamError,
                SdkError::DispatchFailure(_) | SdkError::TimeoutError(_) => {
                    DriveObjectStoreErrorKind::Unavailable
                }
                _ => DriveObjectStoreErrorKind::Internal,
            }
        };

        let message = error
            .as_service_error()
            .and_then(ProvideErrorMetadata::message)
            .map(str::to_string)
            .unwrap_or_else(|| format!("{default_message}: {error}"));

        DriveObjectStoreError::new(kind, message)
    }

    /// Stream a local file straight into S3 without buffering it.
    ///
    /// `ByteStream::from_path` hands the SDK the path, so the SDK reads and
    /// chunks the file itself. That matters for download-package archives,
    /// which are staged on disk precisely because they can be larger than the
    /// process should ever hold in memory.
    async fn put_object_from_path_inner(
        &self,
        request: PutObjectFromPathRequest,
    ) -> Result<PutObjectResponse, DriveObjectStoreError> {
        use aws_sdk_s3::primitives::ByteStream;

        let bucket = self.resolve_bucket(&request.locator.bucket)?;
        Self::validate_locator(&request.locator)?;
        let body = ByteStream::from_path(&request.source_path)
            .await
            .map_err(|error| {
                DriveObjectStoreError::new(
                    DriveObjectStoreErrorKind::Internal,
                    format!("read upload file failed: {error}"),
                )
            })?;
        let mut builder = self
            .client
            .put_object()
            .bucket(bucket)
            .key(request.locator.object_key.clone())
            .body(body);
        if let Some(content_type) = request.content_type {
            builder = builder.content_type(content_type);
        }
        if let Some(checksum) = request.checksum_sha256_hex {
            builder = builder.checksum_sha256(checksum);
        }
        for (key, value) in request.metadata {
            builder = builder.metadata(key, value);
        }
        let output = builder
            .send()
            .await
            .map_err(|error| Self::map_sdk_error(error, "put object from path failed"))?;
        Ok(PutObjectResponse {
            locator: request.locator,
            etag: output.e_tag().map(str::to_string),
            version_id: output.version_id().map(str::to_string),
        })
    }
}

#[async_trait]
impl DriveObjectStore for S3DriveObjectStore {
    fn provider_kind(&self) -> DriveStorageProviderKind {
        self.config.provider_kind.clone()
    }

    fn capabilities(&self) -> DriveStorageProviderCapabilities {
        DriveStorageProviderCapabilities::default_s3_compatible()
    }

    /// Overrides the contract default with a true streaming upload: the SDK
    /// reads and chunks `source_path` itself instead of the adapter buffering
    /// the whole file into a `Vec<u8>` first.
    async fn put_object_from_path(
        &self,
        request: PutObjectFromPathRequest,
    ) -> Result<PutObjectResponse, DriveObjectStoreError> {
        self.put_object_from_path_inner(request).await
    }

    async fn put_object(
        &self,
        request: PutObjectRequest,
    ) -> Result<PutObjectResponse, DriveObjectStoreError> {
        let bucket = self.resolve_bucket(&request.locator.bucket)?;
        Self::validate_locator(&request.locator)?;
        let mut builder = self
            .client
            .put_object()
            .bucket(bucket)
            .key(request.locator.object_key.clone())
            .body(request.body.into());

        if let Some(content_type) = request.content_type {
            builder = builder.content_type(content_type);
        }
        if let Some(checksum) = request.checksum_sha256_hex {
            builder = builder.checksum_sha256(checksum);
        }
        for (key, value) in request.metadata {
            builder = builder.metadata(key, value);
        }

        let output = builder
            .send()
            .await
            .map_err(|error| Self::map_sdk_error(error, "put object failed"))?;

        Ok(PutObjectResponse {
            locator: request.locator,
            etag: output.e_tag().map(str::to_string),
            version_id: output.version_id().map(str::to_string),
        })
    }

    async fn head_object(
        &self,
        request: HeadObjectRequest,
    ) -> Result<HeadObjectResponse, DriveObjectStoreError> {
        let bucket = self.resolve_bucket(&request.locator.bucket)?;
        Self::validate_locator(&request.locator)?;
        let output = self
            .client
            .head_object()
            .bucket(bucket)
            .key(request.locator.object_key.clone())
            .send()
            .await
            .map_err(|error| Self::map_sdk_error(error, "head object failed"))?;

        let content_length = output
            .content_length()
            .unwrap_or_default()
            .try_into()
            .unwrap_or(0_u64);
        Ok(HeadObjectResponse {
            locator: request.locator,
            content_length,
            content_type: output.content_type().map(str::to_string),
            etag: output.e_tag().map(str::to_string),
            version_id: output.version_id().map(str::to_string),
            checksum_sha256_hex: output.checksum_sha256().map(str::to_string),
            metadata: Self::metadata_to_btree(output.metadata()),
        })
    }

    async fn delete_object(
        &self,
        request: DeleteObjectRequest,
    ) -> Result<DeleteObjectResponse, DriveObjectStoreError> {
        let bucket = self.resolve_bucket(&request.locator.bucket)?;
        Self::validate_locator(&request.locator)?;
        self.client
            .delete_object()
            .bucket(bucket)
            .key(request.locator.object_key.clone())
            .send()
            .await
            .map_err(|error| Self::map_sdk_error(error, "delete object failed"))?;

        Ok(DeleteObjectResponse {
            locator: request.locator,
            deleted: true,
        })
    }

    async fn head_bucket(
        &self,
        request: HeadBucketRequest,
    ) -> Result<HeadBucketResponse, DriveObjectStoreError> {
        let bucket = self.resolve_bucket(&request.bucket)?;
        let outcome = self.client.head_bucket().bucket(bucket).send().await;
        match outcome {
            Ok(_) => Ok(HeadBucketResponse {
                bucket: request.bucket,
                exists: true,
            }),
            Err(error) => {
                // HeadBucket responses carry no body, so the SDK cannot give
                // us an error code; the raw 404 status is the canonical
                // "bucket does not exist" signal and must surface as
                // `exists: false` instead of an error.
                let missing = error
                    .raw_response()
                    .is_some_and(|response| response.status().as_u16() == 404);
                if missing {
                    Ok(HeadBucketResponse {
                        bucket: request.bucket,
                        exists: false,
                    })
                } else {
                    Err(Self::map_sdk_error(error, "head bucket failed"))
                }
            }
        }
    }

    async fn list_buckets(
        &self,
        _request: ListBucketsRequest,
    ) -> Result<ListBucketsResponse, DriveObjectStoreError> {
        // 账号清单优先问厂商的服务级域名；它读不出来时退回配置端点，这样只放行地域域名的
        // 私有网络仍然能列出该地域的桶，而不是整个清单报错。
        let output = match self.inventory_service_client.as_ref() {
            Some(service_client) => match service_client.list_buckets().send().await {
                Ok(output) if !output.buckets().is_empty() => output,
                // 空清单也可能是"问错了域名"而不是"账号没有桶"，配置端点的答案更全时以它
                // 为准；两边都空时下面照样会返回空清单。
                _ => self.list_buckets_from_configured_endpoint().await?,
            },
            None => self.list_buckets_from_configured_endpoint().await?,
        };
        // 厂商写在 `<Location>` 里的地域（SDK 的类型化桶里没有这个元素）。
        let published_regions = self.inventory_capture.take_bucket_locations();

        let mut items: Vec<ListedBucket> = output
            .buckets()
            .iter()
            .filter_map(|bucket| {
                let bucket_name = bucket.name()?.to_string();
                Some(ListedBucket {
                    // AWS 回 `<BucketRegion>`；COS / OSS 回 `<Location>`，由清单响应体的
                    // 副本补上；两者都没有的桶留给下面的桶级查询（只有同地域的桶能答）。
                    region: bucket
                        .bucket_region()
                        .map(str::to_string)
                        .or_else(|| published_regions.get(&bucket_name).cloned()),
                    bucket: bucket_name,
                    creation_date_epoch_ms: bucket
                        .creation_date()
                        .and_then(|value| value.to_millis().ok()),
                })
            })
            .collect();

        let unresolved: Vec<String> = items
            .iter()
            .filter(|item| item.region.is_none())
            .map(|item| item.bucket.clone())
            .collect();
        if !unresolved.is_empty() {
            let resolved: BTreeMap<String, String> = self
                .lookup_bucket_regions(&unresolved)
                .await
                .into_iter()
                .filter_map(|(bucket, region)| region.map(|region| (bucket, region)))
                .collect();
            for item in &mut items {
                if item.region.is_none() {
                    if let Some(region) = resolved.get(&item.bucket) {
                        item.region = Some(region.clone());
                    }
                }
            }
        }

        Ok(ListBucketsResponse { items })
    }

    async fn create_bucket(
        &self,
        request: CreateBucketRequest,
    ) -> Result<CreateBucketResponse, DriveObjectStoreError> {
        let bucket = self.resolve_bucket(&request.bucket)?;
        if self
            .head_bucket(HeadBucketRequest {
                bucket: request.bucket.clone(),
            })
            .await?
            .exists
        {
            return Ok(CreateBucketResponse {
                bucket: request.bucket,
                created: false,
            });
        }
        let created = self.client.create_bucket().bucket(bucket).send().await;
        if let Err(error) = created {
            // Race window: another actor may have created the bucket between
            // HEAD and CREATE. S3-compatible vendors disagree on re-create
            // (200 OK vs 409), so converge through a second HEAD instead of
            // failing the initialization.
            let converged = matches!(
                self.head_bucket(HeadBucketRequest {
                    bucket: request.bucket.clone(),
                })
                .await,
                Ok(head) if head.exists
            );
            if !converged {
                return Err(Self::map_sdk_error(error, "create bucket failed"));
            }
        }
        Ok(CreateBucketResponse {
            bucket: request.bucket,
            created: true,
        })
    }

    async fn delete_bucket(
        &self,
        request: DeleteBucketRequest,
    ) -> Result<DeleteBucketResponse, DriveObjectStoreError> {
        let bucket = self.resolve_bucket(&request.bucket)?;
        self.client
            .delete_bucket()
            .bucket(bucket)
            .send()
            .await
            .map_err(|error| Self::map_sdk_error(error, "delete bucket failed"))?;
        Ok(DeleteBucketResponse {
            bucket: request.bucket,
            deleted: true,
        })
    }

    async fn list_objects(
        &self,
        request: ListObjectsRequest,
    ) -> Result<ListObjectsResponse, DriveObjectStoreError> {
        let bucket = self.resolve_bucket(&request.bucket)?;
        let prefix = Self::normalize_list_prefix(request.prefix)?;
        let delimiter = Self::normalize_list_delimiter(request.delimiter)?;
        if request.max_keys == 0 {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "max_keys must be greater than zero",
            ));
        }
        let mut builder = self
            .client
            .list_objects_v2()
            .bucket(bucket)
            .max_keys(i32::from(request.max_keys));
        if let Some(prefix) = prefix.as_deref() {
            builder = builder.prefix(prefix);
        }
        if let Some(delimiter) = delimiter.as_deref() {
            builder = builder.delimiter(delimiter);
        }
        if let Some(token) = request.continuation_token.as_deref() {
            if !token.trim().is_empty() {
                builder = builder.continuation_token(token.trim());
            }
        }
        let output = builder
            .send()
            .await
            .map_err(|error| Self::map_sdk_error(error, "list objects failed"))?;
        let items = output
            .contents()
            .iter()
            .filter_map(|object| {
                let object_key = object.key()?.to_string();
                let content_length = object
                    .size()
                    .unwrap_or_default()
                    .try_into()
                    .unwrap_or(0_u64);
                let last_modified_epoch_ms = object
                    .last_modified()
                    .and_then(|value| value.to_millis().ok());
                Some(ListedObject {
                    object_key,
                    content_length,
                    etag: object.e_tag().map(str::to_string),
                    storage_class: object
                        .storage_class()
                        .map(|value| value.as_str().to_string()),
                    last_modified_epoch_ms,
                })
            })
            .collect();
        let prefixes = output
            .common_prefixes()
            .iter()
            .filter_map(|prefix| prefix.prefix().map(str::to_string))
            .collect();
        Ok(ListObjectsResponse {
            bucket: request.bucket,
            prefix,
            items,
            prefixes,
            next_continuation_token: output.next_continuation_token().map(str::to_string),
            is_truncated: output.is_truncated().unwrap_or(false),
        })
    }

    async fn copy_object(
        &self,
        request: CopyObjectRequest,
    ) -> Result<CopyObjectResponse, DriveObjectStoreError> {
        let destination_bucket = self.resolve_bucket(&request.destination.bucket)?;
        let source_bucket = self.resolve_bucket(&request.source.bucket)?;
        Self::validate_locator(&request.source)?;
        Self::validate_locator(&request.destination)?;
        let copy_source = format!(
            "{}/{}",
            source_bucket,
            request.source.object_key.trim_start_matches('/')
        );
        let mut builder = self
            .client
            .copy_object()
            .bucket(destination_bucket)
            .key(request.destination.object_key.clone())
            .copy_source(copy_source);
        if let Some(metadata_directive) = request.metadata_directive.as_deref() {
            if !metadata_directive.trim().is_empty() {
                builder = builder.metadata_directive(aws_sdk_s3::types::MetadataDirective::from(
                    metadata_directive.trim(),
                ));
            }
        }
        let output = builder
            .send()
            .await
            .map_err(|error| Self::map_sdk_error(error, "copy object failed"))?;
        Ok(CopyObjectResponse {
            locator: request.destination,
            etag: output
                .copy_object_result()
                .and_then(|result| result.e_tag())
                .map(str::to_string),
            version_id: output.version_id().map(str::to_string),
        })
    }

    async fn create_multipart_upload(
        &self,
        request: CreateMultipartUploadRequest,
    ) -> Result<CreateMultipartUploadResponse, DriveObjectStoreError> {
        let bucket = self.resolve_bucket(&request.locator.bucket)?;
        Self::validate_locator(&request.locator)?;
        let mut builder = self
            .client
            .create_multipart_upload()
            .bucket(bucket)
            .key(request.locator.object_key.clone());

        if let Some(content_type) = request.content_type {
            builder = builder.content_type(content_type);
        }
        for (key, value) in request.metadata {
            builder = builder.metadata(key, value);
        }

        let output = builder
            .send()
            .await
            .map_err(|error| Self::map_sdk_error(error, "create multipart upload failed"))?;
        let upload_id = output.upload_id().ok_or_else(|| {
            DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::UpstreamError,
                "create multipart upload returned no upload_id",
            )
        })?;

        Ok(CreateMultipartUploadResponse {
            locator: request.locator,
            upload_id: upload_id.to_string(),
        })
    }

    async fn presign_upload_part(
        &self,
        request: PresignUploadPartRequest,
    ) -> Result<PresignedUploadPartResponse, DriveObjectStoreError> {
        let bucket = self.resolve_bucket(&request.locator.bucket)?;
        Self::validate_locator(&request.locator)?;
        Self::validate_presign_expiry(request.expires_in_seconds)?;
        let presigning_config = PresigningConfig::expires_in(Duration::from_secs(u64::from(
            request.expires_in_seconds,
        )))
        .map_err(|error| {
            DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                format!("invalid presign expiry: {error}"),
            )
        })?;
        let presigned = self
            .client
            .upload_part()
            .bucket(bucket)
            .key(request.locator.object_key.clone())
            .upload_id(request.upload_id)
            .part_number(i32::from(request.part_number))
            .presigned(presigning_config)
            .await
            .map_err(|error| Self::map_sdk_error(error, "presign upload part failed"))?;

        Ok(PresignedUploadPartResponse {
            method: presigned.method().to_string(),
            url: presigned.uri().to_string(),
            headers: Self::headers_from_presigned(&presigned),
            expires_at_epoch_ms: Self::expires_at_epoch_ms(request.expires_in_seconds),
        })
    }

    async fn complete_multipart_upload(
        &self,
        request: CompleteMultipartUploadRequest,
    ) -> Result<CompleteMultipartUploadResponse, DriveObjectStoreError> {
        let bucket = self.resolve_bucket(&request.locator.bucket)?;
        Self::validate_locator(&request.locator)?;
        let completed_parts: Vec<CompletedPart> = request
            .parts
            .into_iter()
            .map(|part| {
                CompletedPart::builder()
                    .part_number(i32::from(part.part_number))
                    .e_tag(part.etag)
                    .build()
            })
            .collect();
        let completed_upload = CompletedMultipartUpload::builder()
            .set_parts(Some(completed_parts))
            .build();

        let output = self
            .client
            .complete_multipart_upload()
            .bucket(bucket)
            .key(request.locator.object_key.clone())
            .upload_id(request.upload_id)
            .multipart_upload(completed_upload)
            .send()
            .await
            .map_err(|error| Self::map_sdk_error(error, "complete multipart upload failed"))?;

        Ok(CompleteMultipartUploadResponse {
            locator: request.locator,
            etag: output.e_tag().map(str::to_string),
            version_id: output.version_id().map(str::to_string),
        })
    }

    async fn abort_multipart_upload(
        &self,
        request: AbortMultipartUploadRequest,
    ) -> Result<(), DriveObjectStoreError> {
        let bucket = self.resolve_bucket(&request.locator.bucket)?;
        Self::validate_locator(&request.locator)?;
        self.client
            .abort_multipart_upload()
            .bucket(bucket)
            .key(request.locator.object_key.clone())
            .upload_id(request.upload_id)
            .send()
            .await
            .map_err(|error| Self::map_sdk_error(error, "abort multipart upload failed"))?;
        Ok(())
    }

    async fn presign_download(
        &self,
        request: PresignDownloadRequest,
    ) -> Result<PresignedDownloadResponse, DriveObjectStoreError> {
        let bucket = self.resolve_bucket(&request.locator.bucket)?;
        Self::validate_locator(&request.locator)?;
        Self::validate_presign_expiry(request.expires_in_seconds)?;
        let presigning_config = PresigningConfig::expires_in(Duration::from_secs(u64::from(
            request.expires_in_seconds,
        )))
        .map_err(|error| {
            DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                format!("invalid presign expiry: {error}"),
            )
        })?;
        let presigned = self
            .client
            .get_object()
            .bucket(bucket)
            .key(request.locator.object_key.clone())
            .presigned(presigning_config)
            .await
            .map_err(|error| Self::map_sdk_error(error, "presign download failed"))?;

        Ok(PresignedDownloadResponse {
            method: presigned.method().to_string(),
            url: presigned.uri().to_string(),
            headers: Self::headers_from_presigned(&presigned),
            expires_at_epoch_ms: Self::expires_at_epoch_ms(request.expires_in_seconds),
        })
    }

    async fn read_object_range(
        &self,
        request: ReadObjectRangeRequest,
    ) -> Result<(ReadObjectRangeResponse, Box<dyn DriveObjectChunkStream>), DriveObjectStoreError>
    {
        let bucket = self.resolve_bucket(&request.locator.bucket)?;
        Self::validate_locator(&request.locator)?;
        let range_value = format!(
            "bytes={}-{}",
            request.range.start_inclusive, request.range.end_inclusive
        );
        let output = self
            .client
            .get_object()
            .bucket(bucket)
            .key(request.locator.object_key.clone())
            .range(range_value)
            .send()
            .await
            .map_err(|error| Self::map_sdk_error(error, "read object range failed"))?;

        let content_type = output.content_type().map(str::to_string);
        let etag = output.e_tag().map(str::to_string);
        let bytes = output
            .body
            .collect()
            .await
            .map_err(|error| {
                DriveObjectStoreError::new(
                    DriveObjectStoreErrorKind::UpstreamError,
                    format!("read object body failed: {error}"),
                )
            })?
            .into_bytes()
            .to_vec();
        let content_length = bytes.len() as u64;
        let stream: Box<dyn DriveObjectChunkStream> =
            Box::new(SingleChunkStream { body: Some(bytes) });

        Ok((
            ReadObjectRangeResponse {
                locator: request.locator,
                content_type,
                etag,
                content_length,
            },
            stream,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_bucket_location;

    #[test]
    fn keeps_the_region_a_vendor_reports() {
        assert_eq!(
            normalize_bucket_location(Some("ap-guangzhou")),
            Some("ap-guangzhou".to_string())
        );
        assert_eq!(
            normalize_bucket_location(Some("  oss-cn-hangzhou  ")),
            Some("oss-cn-hangzhou".to_string())
        );
    }

    #[test]
    fn an_empty_or_null_answer_is_not_a_region() {
        // S3 answers `us-east-1` with an empty constraint, and a vendor that
        // cannot answer may send the literal `null`. Writing either into a row
        // would put a region on screen that no bucket lives in.
        assert_eq!(normalize_bucket_location(Some("")), None);
        assert_eq!(normalize_bucket_location(Some("   ")), None);
        assert_eq!(normalize_bucket_location(Some("null")), None);
        assert_eq!(normalize_bucket_location(Some("NULL")), None);
        assert_eq!(normalize_bucket_location(None), None);
    }
}
