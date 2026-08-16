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

    async fn list_buckets(
        &self,
        request: ListBucketsRequest,
    ) -> Result<ListBucketsResponse, DriveObjectStoreError>;

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
