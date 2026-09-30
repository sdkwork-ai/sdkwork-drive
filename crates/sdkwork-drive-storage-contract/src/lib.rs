use async_trait::async_trait;

mod types;

pub use types::*;

#[async_trait]
pub trait DriveObjectChunkStream: Send + Sync {
    async fn next_chunk(&mut self) -> Result<Option<Vec<u8>>, DriveObjectStoreError>;
}

#[async_trait]
pub trait DriveObjectStore: Send + Sync {
    fn provider_kind(&self) -> DriveStorageProviderKind;

    fn capabilities(&self) -> DriveStorageProviderCapabilities;

    async fn put_object(
        &self,
        request: PutObjectRequest,
    ) -> Result<PutObjectResponse, DriveObjectStoreError>;

    /// Store an object whose bytes already live in a local file.
    ///
    /// Separate from [`Self::put_object`] because the size is unbounded: a
    /// download package archive is staged on disk precisely so it never has to
    /// be held in memory. An adapter that can stream from a path should; the
    /// contract default reads the file once and delegates to `put_object`, which
    /// keeps every provider correct while letting the S3 adapter avoid a
    /// needless full-buffer copy.
    async fn put_object_from_path(
        &self,
        request: PutObjectFromPathRequest,
    ) -> Result<PutObjectResponse, DriveObjectStoreError> {
        let body = tokio::fs::read(&request.source_path)
            .await
            .map_err(|error| {
                DriveObjectStoreError::new(
                    DriveObjectStoreErrorKind::Internal,
                    format!("read object source file failed: {error}"),
                )
            })?;
        self.put_object(PutObjectRequest {
            locator: request.locator,
            content_type: request.content_type,
            metadata: request.metadata,
            body,
            checksum_sha256_hex: request.checksum_sha256_hex,
        })
        .await
    }

    async fn head_object(
        &self,
        request: HeadObjectRequest,
    ) -> Result<HeadObjectResponse, DriveObjectStoreError>;

    async fn delete_object(
        &self,
        request: DeleteObjectRequest,
    ) -> Result<DeleteObjectResponse, DriveObjectStoreError>;

    async fn head_bucket(
        &self,
        request: HeadBucketRequest,
    ) -> Result<HeadBucketResponse, DriveObjectStoreError>;

    /// Ensure the bucket exists. Implementations MUST be idempotent: calling
    /// this for a bucket that already exists (and is owned by the caller)
    /// succeeds with `created: false` instead of failing, so re-running
    /// bucket initialization converges instead of erroring on vendors that
    /// reject re-creation.
    async fn list_buckets(
        &self,
        request: ListBucketsRequest,
    ) -> Result<ListBucketsResponse, DriveObjectStoreError>;

    /// Ensure the bucket exists; `created` reports whether this call made the
    /// change. Idempotent per the contract-wide initialization semantics.
    async fn create_bucket(
        &self,
        request: CreateBucketRequest,
    ) -> Result<CreateBucketResponse, DriveObjectStoreError>;

    async fn delete_bucket(
        &self,
        request: DeleteBucketRequest,
    ) -> Result<DeleteBucketResponse, DriveObjectStoreError>;

    async fn list_objects(
        &self,
        request: ListObjectsRequest,
    ) -> Result<ListObjectsResponse, DriveObjectStoreError>;

    async fn copy_object(
        &self,
        request: CopyObjectRequest,
    ) -> Result<CopyObjectResponse, DriveObjectStoreError>;

    async fn create_multipart_upload(
        &self,
        request: CreateMultipartUploadRequest,
    ) -> Result<CreateMultipartUploadResponse, DriveObjectStoreError>;

    async fn presign_upload_part(
        &self,
        request: PresignUploadPartRequest,
    ) -> Result<PresignedUploadPartResponse, DriveObjectStoreError>;

    async fn complete_multipart_upload(
        &self,
        request: CompleteMultipartUploadRequest,
    ) -> Result<CompleteMultipartUploadResponse, DriveObjectStoreError>;

    async fn abort_multipart_upload(
        &self,
        request: AbortMultipartUploadRequest,
    ) -> Result<(), DriveObjectStoreError>;

    async fn presign_download(
        &self,
        request: PresignDownloadRequest,
    ) -> Result<PresignedDownloadResponse, DriveObjectStoreError>;

    async fn read_object_range(
        &self,
        request: ReadObjectRangeRequest,
    ) -> Result<(ReadObjectRangeResponse, Box<dyn DriveObjectChunkStream>), DriveObjectStoreError>;
}
