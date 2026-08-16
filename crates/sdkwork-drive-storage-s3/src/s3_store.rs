use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::error::{ProvideErrorMetadata, SdkError};
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
use aws_sdk_s3::{Client, Config};
use aws_types::region::Region;
use sdkwork_drive_storage_contract::{
    AbortMultipartUploadRequest, CompleteMultipartUploadRequest, CompleteMultipartUploadResponse,
    CopyObjectRequest, CopyObjectResponse, CreateBucketRequest, CreateBucketResponse,
    CreateMultipartUploadRequest, CreateMultipartUploadResponse, DeleteBucketRequest,
    DeleteBucketResponse, DeleteObjectRequest, DeleteObjectResponse, DriveObjectChunkStream,
    DriveObjectHeaders, DriveObjectLocator, DriveObjectStore, DriveObjectStoreError,
    DriveObjectStoreErrorKind, DriveStorageProviderCapabilities, DriveStorageProviderKind,
    HeadBucketRequest, HeadBucketResponse, HeadObjectRequest, HeadObjectResponse,
    ListBucketsRequest, ListBucketsResponse, ListObjectsRequest, ListObjectsResponse, ListedBucket,
    ListedObject, PresignDownloadRequest, PresignUploadPartRequest, PresignedDownloadResponse,
    PresignedUploadPartResponse, PutObjectRequest, PutObjectResponse, ReadObjectRangeRequest,
    ReadObjectRangeResponse,
};
use std::collections::BTreeMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::config::S3StoreConfig;

#[derive(Debug, Clone)]
pub struct S3DriveObjectStore {
    client: Client,
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

        let credentials = Credentials::new(
            config.access_key_id.clone(),
            config.secret_access_key.clone(),
            config.session_token.clone(),
            None,
            "sdkwork-drive-storage-s3",
        );
        let mut loader = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(config.region.clone()))
            .credentials_provider(credentials);
        if let Some(endpoint) = config.endpoint.as_ref() {
            loader = loader.endpoint_url(endpoint);
        }
        let shared_config = loader.load().await;

        let mut s3_config_builder = Config::from(&shared_config)
            .to_builder()
            .force_path_style(config.force_path_style)
            .region(Region::new(config.region.clone()))
            .credentials_provider(Credentials::new(
                config.access_key_id.clone(),
                config.secret_access_key.clone(),
                config.session_token.clone(),
                None,
                "sdkwork-drive-storage-s3",
            ));

        if let Some(endpoint) = config.endpoint.clone() {
            s3_config_builder = s3_config_builder.endpoint_url(endpoint);
        }

        let client = Client::from_conf(s3_config_builder.build());
        Ok(Self { client, config })
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

    pub async fn put_object_from_path(
        &self,
        locator: DriveObjectLocator,
        content_type: Option<String>,
        metadata: BTreeMap<String, String>,
        file_path: &std::path::Path,
    ) -> Result<PutObjectResponse, DriveObjectStoreError> {
        use aws_sdk_s3::primitives::ByteStream;

        let bucket = self.resolve_bucket(&locator.bucket)?;
        Self::validate_locator(&locator)?;
        let body = ByteStream::from_path(file_path).await.map_err(|error| {
            DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::Internal,
                format!("read upload file failed: {error}"),
            )
        })?;
        let mut builder = self
            .client
            .put_object()
            .bucket(bucket)
            .key(locator.object_key.clone())
            .body(body);
        if let Some(content_type) = content_type {
            builder = builder.content_type(content_type);
        }
        for (key, value) in metadata {
            builder = builder.metadata(key, value);
        }
        let output = builder
            .send()
            .await
            .map_err(|error| Self::map_sdk_error(error, "put object from path failed"))?;
        Ok(PutObjectResponse {
            locator,
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
        self.client
            .head_bucket()
            .bucket(bucket.clone())
            .send()
            .await
            .map_err(|error| Self::map_sdk_error(error, "head bucket failed"))?;
        Ok(HeadBucketResponse {
            bucket: request.bucket,
            exists: true,
        })
    }

    async fn list_buckets(
        &self,
        _request: ListBucketsRequest,
    ) -> Result<ListBucketsResponse, DriveObjectStoreError> {
        let output = self
            .client
            .list_buckets()
            .send()
            .await
            .map_err(|error| Self::map_sdk_error(error, "list buckets failed"))?;
        let items = output
            .buckets()
            .iter()
            .filter_map(|bucket| {
                let bucket_name = bucket.name()?.to_string();
                Some(ListedBucket {
                    bucket: bucket_name,
                    creation_date_epoch_ms: bucket
                        .creation_date()
                        .and_then(|value| value.to_millis().ok()),
                })
            })
            .collect();
        Ok(ListBucketsResponse { items })
    }

    async fn create_bucket(
        &self,
        request: CreateBucketRequest,
    ) -> Result<CreateBucketResponse, DriveObjectStoreError> {
        let bucket = self.resolve_bucket(&request.bucket)?;
        self.client
            .create_bucket()
            .bucket(bucket)
            .send()
            .await
            .map_err(|error| Self::map_sdk_error(error, "create bucket failed"))?;
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
