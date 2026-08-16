use async_trait::async_trait;
use opendal::services::S3;
use opendal::{ErrorKind, Metadata, Operator};
use sdkwork_drive_storage_contract::{
    AbortMultipartUploadRequest, CompleteMultipartUploadRequest, CompleteMultipartUploadResponse,
    CopyObjectRequest, CopyObjectResponse, CreateBucketRequest, CreateBucketResponse,
    CreateMultipartUploadRequest, CreateMultipartUploadResponse, DeleteBucketRequest,
    DeleteBucketResponse, DeleteObjectRequest, DeleteObjectResponse, DriveObjectChunkStream,
    DriveObjectHeaders, DriveObjectLocator, DriveObjectMetadata, DriveObjectStore,
    DriveObjectStoreError, DriveObjectStoreErrorKind, DriveStorageProviderCapabilities,
    DriveStorageProviderKind, HeadBucketRequest, HeadBucketResponse, HeadObjectRequest,
    HeadObjectResponse, ListBucketsRequest, ListBucketsResponse, ListObjectsRequest,
    ListObjectsResponse, ListedObject, PresignDownloadRequest, PresignUploadPartRequest,
    PresignedDownloadResponse, PresignedUploadPartResponse, PutObjectRequest, PutObjectResponse,
    ReadObjectRangeRequest, ReadObjectRangeResponse,
};
use std::collections::BTreeMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::config::{normalize_list_prefix, validate_object_key, OpendalS3StoreConfig};

#[derive(Debug, Clone)]
pub struct OpendalS3DriveObjectStore {
    operator: Operator,
    config: OpendalS3StoreConfig,
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

impl OpendalS3DriveObjectStore {
    pub fn new(config: OpendalS3StoreConfig) -> Result<Self, DriveObjectStoreError> {
        config.validate()?;
        let mut builder = S3::default()
            .bucket(&config.default_bucket)
            .region(&config.region)
            .access_key_id(&config.access_key_id)
            .secret_access_key(&config.secret_access_key);

        if let Some(endpoint) = config.endpoint.as_deref() {
            builder = builder.endpoint(endpoint);
        }
        if let Some(session_token) = config.session_token.as_deref() {
            builder = builder.session_token(session_token);
        }
        if let Some(root) = config.root.as_deref() {
            builder = builder.root(root);
        }
        if config.disable_config_load {
            builder = builder.disable_config_load();
        }
        if !config.force_path_style {
            builder = builder.enable_virtual_host_style();
        }
        if let Some(default_storage_class) = config.default_storage_class.as_deref() {
            builder = builder.default_storage_class(default_storage_class);
        }
        if let Some(server_side_encryption) = config.server_side_encryption.as_deref() {
            builder = builder.server_side_encryption(server_side_encryption);
        }

        let operator = Operator::new(builder).map_err(map_opendal_error)?.finish();
        Ok(Self { operator, config })
    }

    pub fn config(&self) -> &OpendalS3StoreConfig {
        &self.config
    }

    fn validate_default_bucket_only(&self, bucket: &str) -> Result<(), DriveObjectStoreError> {
        let bucket = self.config.resolve_bucket(bucket)?;
        if bucket != self.config.default_bucket {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::NotSupported,
                "OpenDAL S3 plugin operator is bound to one bucket; use one provider per bucket or the AWS S3 adapter for cross-bucket operations",
            ));
        }
        Ok(())
    }

    fn validate_locator(&self, locator: &DriveObjectLocator) -> Result<(), DriveObjectStoreError> {
        self.validate_default_bucket_only(&locator.bucket)?;
        validate_object_key(&locator.object_key)
    }

    fn not_supported(operation: &str) -> DriveObjectStoreError {
        DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::NotSupported,
            format!("{operation} is not supported by the OpenDAL S3 Drive plugin"),
        )
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

    fn expires_at_epoch_ms(expires_in_seconds: u32) -> i64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis() as i64)
            .unwrap_or(0);
        now + i64::from(expires_in_seconds) * 1000
    }
}

#[async_trait]
impl DriveObjectStore for OpendalS3DriveObjectStore {
    fn provider_kind(&self) -> DriveStorageProviderKind {
        self.config.provider_kind.clone()
    }

    fn capabilities(&self) -> DriveStorageProviderCapabilities {
        DriveStorageProviderCapabilities {
            supports_multipart_upload: false,
            supports_presigned_upload_part: false,
            supports_presigned_download: true,
            supports_range_read: true,
            supports_server_side_copy: true,
            supports_versioning: false,
        }
    }

    async fn put_object(
        &self,
        request: PutObjectRequest,
    ) -> Result<PutObjectResponse, DriveObjectStoreError> {
        self.validate_locator(&request.locator)?;
        let mut writer = self
            .operator
            .write_with(&request.locator.object_key, request.body);
        if let Some(content_type) = request.content_type.as_deref() {
            writer = writer.content_type(content_type);
        }
        if !request.metadata.is_empty() {
            writer = writer.user_metadata(metadata_to_hash_map(request.metadata));
        }
        let metadata = writer.await.map_err(map_opendal_error)?;
        Ok(PutObjectResponse {
            locator: request.locator,
            etag: metadata.etag().map(str::to_string),
            version_id: metadata.version().map(str::to_string),
        })
    }

    async fn head_object(
        &self,
        request: HeadObjectRequest,
    ) -> Result<HeadObjectResponse, DriveObjectStoreError> {
        self.validate_locator(&request.locator)?;
        let metadata = self
            .operator
            .stat(&request.locator.object_key)
            .await
            .map_err(map_opendal_error)?;
        Ok(map_metadata_to_head_response(request.locator, metadata))
    }

    async fn delete_object(
        &self,
        request: DeleteObjectRequest,
    ) -> Result<DeleteObjectResponse, DriveObjectStoreError> {
        self.validate_locator(&request.locator)?;
        self.operator
            .delete(&request.locator.object_key)
            .await
            .map_err(map_opendal_error)?;
        Ok(DeleteObjectResponse {
            locator: request.locator,
            deleted: true,
        })
    }

    async fn head_bucket(
        &self,
        request: HeadBucketRequest,
    ) -> Result<HeadBucketResponse, DriveObjectStoreError> {
        self.validate_default_bucket_only(&request.bucket)?;
        Err(Self::not_supported("head_bucket"))
    }

    async fn list_buckets(
        &self,
        _request: ListBucketsRequest,
    ) -> Result<ListBucketsResponse, DriveObjectStoreError> {
        Err(Self::not_supported("list_buckets"))
    }

    async fn create_bucket(
        &self,
        request: CreateBucketRequest,
    ) -> Result<CreateBucketResponse, DriveObjectStoreError> {
        self.validate_default_bucket_only(&request.bucket)?;
        Err(Self::not_supported("create_bucket"))
    }

    async fn delete_bucket(
        &self,
        request: DeleteBucketRequest,
    ) -> Result<DeleteBucketResponse, DriveObjectStoreError> {
        self.validate_default_bucket_only(&request.bucket)?;
        Err(Self::not_supported("delete_bucket"))
    }

    async fn list_objects(
        &self,
        request: ListObjectsRequest,
    ) -> Result<ListObjectsResponse, DriveObjectStoreError> {
        self.validate_default_bucket_only(&request.bucket)?;
        if request.max_keys == 0 {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "max_keys must be greater than zero",
            ));
        }
        if let Some(delimiter) = request.delimiter.as_deref() {
            if !delimiter.trim().is_empty() && delimiter.trim() != "/" {
                return Err(DriveObjectStoreError::new(
                    DriveObjectStoreErrorKind::InvalidRequest,
                    "delimiter must be '/' when provided",
                ));
            }
        }
        let prefix = normalize_list_prefix(request.prefix)?;
        let mut list = self
            .operator
            .list_with(prefix.as_deref().unwrap_or(""))
            .limit(usize::from(request.max_keys))
            .recursive(request.delimiter.as_deref().map(str::trim) != Some("/"));
        if let Some(token) = request.continuation_token.as_deref() {
            if !token.trim().is_empty() {
                list = list.start_after(token.trim());
            }
        }
        let entries = list.await.map_err(map_opendal_error)?;
        let mut items = Vec::new();
        let mut prefixes = Vec::new();
        let delimiter_mode = request.delimiter.as_deref().map(str::trim) == Some("/");
        for entry in entries {
            let metadata = entry.metadata();
            if metadata.is_file() {
                items.push(ListedObject {
                    object_key: entry.path().trim_end_matches('/').to_string(),
                    content_length: metadata.content_length(),
                    etag: metadata.etag().map(str::to_string),
                    storage_class: None,
                    last_modified_epoch_ms: None,
                });
                continue;
            }
            if delimiter_mode {
                let path = entry.path().trim_end_matches('/');
                if !path.is_empty() {
                    prefixes.push(format!("{path}/"));
                }
            }
        }
        prefixes.sort();
        prefixes.dedup();
        let mut continuation_keys = items
            .iter()
            .map(|item| item.object_key.as_str())
            .chain(prefixes.iter().map(String::as_str))
            .collect::<Vec<_>>();
        continuation_keys.sort_unstable();
        let is_truncated = continuation_keys.len() >= usize::from(request.max_keys);
        let next_continuation_token = if is_truncated {
            continuation_keys.last().map(|key| (*key).to_string())
        } else {
            None
        };
        Ok(ListObjectsResponse {
            bucket: request.bucket,
            prefix,
            items,
            prefixes,
            next_continuation_token,
            is_truncated,
        })
    }

    async fn copy_object(
        &self,
        request: CopyObjectRequest,
    ) -> Result<CopyObjectResponse, DriveObjectStoreError> {
        self.validate_locator(&request.source)?;
        self.validate_locator(&request.destination)?;
        if request.source.bucket != request.destination.bucket {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::NotSupported,
                "OpenDAL S3 plugin only supports same-bucket copy for the provider default bucket",
            ));
        }
        let metadata = self
            .operator
            .copy(&request.source.object_key, &request.destination.object_key)
            .await
            .map_err(map_opendal_error)?;
        Ok(CopyObjectResponse {
            locator: request.destination,
            etag: metadata.etag().map(str::to_string),
            version_id: metadata.version().map(str::to_string),
        })
    }

    async fn create_multipart_upload(
        &self,
        request: CreateMultipartUploadRequest,
    ) -> Result<CreateMultipartUploadResponse, DriveObjectStoreError> {
        self.validate_locator(&request.locator)?;
        Err(Self::not_supported("create_multipart_upload"))
    }

    async fn presign_upload_part(
        &self,
        request: PresignUploadPartRequest,
    ) -> Result<PresignedUploadPartResponse, DriveObjectStoreError> {
        self.validate_locator(&request.locator)?;
        Self::validate_presign_expiry(request.expires_in_seconds)?;
        Err(Self::not_supported("presign_upload_part"))
    }

    async fn complete_multipart_upload(
        &self,
        request: CompleteMultipartUploadRequest,
    ) -> Result<CompleteMultipartUploadResponse, DriveObjectStoreError> {
        self.validate_locator(&request.locator)?;
        Err(Self::not_supported("complete_multipart_upload"))
    }

    async fn abort_multipart_upload(
        &self,
        request: AbortMultipartUploadRequest,
    ) -> Result<(), DriveObjectStoreError> {
        self.validate_locator(&request.locator)?;
        Err(Self::not_supported("abort_multipart_upload"))
    }

    async fn presign_download(
        &self,
        request: PresignDownloadRequest,
    ) -> Result<PresignedDownloadResponse, DriveObjectStoreError> {
        self.validate_locator(&request.locator)?;
        Self::validate_presign_expiry(request.expires_in_seconds)?;
        let presigned = self
            .operator
            .presign_read(
                &request.locator.object_key,
                Duration::from_secs(u64::from(request.expires_in_seconds)),
            )
            .await
            .map_err(map_opendal_error)?;
        Ok(PresignedDownloadResponse {
            method: presigned.method().to_string(),
            url: presigned.uri().to_string(),
            headers: headers_from_presigned(presigned.header()),
            expires_at_epoch_ms: Self::expires_at_epoch_ms(request.expires_in_seconds),
        })
    }

    async fn read_object_range(
        &self,
        request: ReadObjectRangeRequest,
    ) -> Result<(ReadObjectRangeResponse, Box<dyn DriveObjectChunkStream>), DriveObjectStoreError>
    {
        self.validate_locator(&request.locator)?;
        if request.range.end_inclusive < request.range.start_inclusive {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "range end_inclusive must be greater than or equal to start_inclusive",
            ));
        }
        let end_exclusive = request.range.end_inclusive.checked_add(1).ok_or_else(|| {
            DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "range end_inclusive is too large",
            )
        })?;
        let buffer = self
            .operator
            .read_with(&request.locator.object_key)
            .range(request.range.start_inclusive..end_exclusive)
            .await
            .map_err(map_opendal_error)?;
        let bytes = buffer.to_vec();
        let response = ReadObjectRangeResponse {
            locator: request.locator,
            content_type: None,
            etag: None,
            content_length: bytes.len() as u64,
        };
        Ok((response, Box::new(SingleChunkStream { body: Some(bytes) })))
    }
}

fn metadata_to_hash_map(
    metadata: DriveObjectMetadata,
) -> std::collections::HashMap<String, String> {
    metadata.into_iter().collect()
}

fn map_metadata_to_head_response(
    locator: DriveObjectLocator,
    metadata: Metadata,
) -> HeadObjectResponse {
    HeadObjectResponse {
        locator,
        content_length: metadata.content_length(),
        content_type: metadata.content_type().map(str::to_string),
        etag: metadata.etag().map(str::to_string),
        version_id: metadata.version().map(str::to_string),
        checksum_sha256_hex: None,
        metadata: metadata
            .user_metadata()
            .map(|items| {
                items
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect::<BTreeMap<String, String>>()
            })
            .unwrap_or_default(),
    }
}

fn headers_from_presigned(headers: &http::HeaderMap) -> DriveObjectHeaders {
    headers
        .iter()
        .filter_map(|(key, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (key.to_string(), value.to_string()))
        })
        .collect()
}

fn map_opendal_error(error: opendal::Error) -> DriveObjectStoreError {
    let kind = match error.kind() {
        ErrorKind::NotFound => DriveObjectStoreErrorKind::NotFound,
        ErrorKind::PermissionDenied => DriveObjectStoreErrorKind::PermissionDenied,
        ErrorKind::AlreadyExists | ErrorKind::IsSameFile | ErrorKind::ConditionNotMatch => {
            DriveObjectStoreErrorKind::Conflict
        }
        ErrorKind::RateLimited => DriveObjectStoreErrorKind::RateLimited,
        ErrorKind::Unsupported => DriveObjectStoreErrorKind::NotSupported,
        ErrorKind::ConfigInvalid
        | ErrorKind::IsADirectory
        | ErrorKind::NotADirectory
        | ErrorKind::RangeNotSatisfied => DriveObjectStoreErrorKind::InvalidRequest,
        _ => DriveObjectStoreErrorKind::UpstreamError,
    };
    DriveObjectStoreError::new(kind, error.to_string())
}
