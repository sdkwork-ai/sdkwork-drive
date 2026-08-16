use async_trait::async_trait;
use sdkwork_drive_storage_contract::{
    AbortMultipartUploadRequest, CompleteMultipartUploadRequest, CompleteMultipartUploadResponse,
    CopyObjectRequest, CopyObjectResponse, CreateBucketRequest, CreateBucketResponse,
    CreateMultipartUploadRequest, CreateMultipartUploadResponse, DeleteBucketRequest,
    DeleteBucketResponse, DeleteObjectRequest, DeleteObjectResponse, DriveObjectChunkStream,
    DriveObjectLocator, DriveObjectStore, DriveObjectStoreError, DriveObjectStoreErrorKind,
    DriveStorageProviderCapabilities, DriveStorageProviderKind, HeadBucketRequest,
    HeadBucketResponse, HeadObjectRequest, HeadObjectResponse, ListBucketsRequest,
    ListBucketsResponse, ListObjectsRequest, ListObjectsResponse, ListedBucket, ListedObject,
    PresignDownloadRequest, PresignUploadPartRequest, PresignedDownloadResponse,
    PresignedUploadPartResponse, PutObjectRequest, PutObjectResponse, ReadObjectRangeRequest,
    ReadObjectRangeResponse,
};
use sdkwork_utils_rust::sha256_hash;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct LocalDriveObjectStore {
    root_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalObjectMetadata {
    content_type: Option<String>,
    metadata: std::collections::BTreeMap<String, String>,
    checksum_sha256_hex: Option<String>,
}

#[derive(Debug)]
struct ListedObjectPageCandidate {
    sort_key: String,
    entry: ListedObjectPageEntry,
}

#[derive(Debug)]
enum ListedObjectPageEntry {
    Object(ListedObject),
    Prefix(String),
}

struct CollectedObjectPage {
    items: Vec<ListedObject>,
    prefixes: Vec<String>,
    next_continuation_token: Option<String>,
}

impl Eq for ListedObjectPageCandidate {}

impl PartialEq for ListedObjectPageCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.sort_key == other.sort_key
    }
}

impl Ord for ListedObjectPageCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sort_key.cmp(&other.sort_key)
    }
}

impl PartialOrd for ListedObjectPageCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
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

impl LocalDriveObjectStore {
    pub fn new(root_dir: impl AsRef<Path>) -> Self {
        Self {
            root_dir: root_dir.as_ref().to_path_buf(),
        }
    }

    fn validate_relative_segment(
        value: &str,
        segment_name: &str,
    ) -> Result<(), DriveObjectStoreError> {
        if value.trim().is_empty() {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                format!("{segment_name} must not be empty"),
            ));
        }

        let path = Path::new(value);
        if path.is_absolute() {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                format!("{segment_name} must be a relative path"),
            ));
        }

        for component in path.components() {
            match component {
                Component::Normal(_) => {}
                _ => {
                    return Err(DriveObjectStoreError::new(
                        DriveObjectStoreErrorKind::InvalidRequest,
                        format!("{segment_name} contains invalid component"),
                    ));
                }
            }
        }

        Ok(())
    }

    fn object_path(&self, locator: &DriveObjectLocator) -> Result<PathBuf, DriveObjectStoreError> {
        Self::validate_relative_segment(&locator.bucket, "bucket")?;
        Self::validate_relative_segment(&locator.object_key, "object_key")?;
        Ok(self
            .root_dir
            .join(&locator.bucket)
            .join(&locator.object_key))
    }

    fn bucket_path(&self, bucket: &str) -> Result<PathBuf, DriveObjectStoreError> {
        Self::validate_relative_segment(bucket, "bucket")?;
        Ok(self.root_dir.join(bucket))
    }

    fn metadata_path(object_path: &Path) -> PathBuf {
        PathBuf::from(format!("{}.meta.json", object_path.display()))
    }

    fn ensure_parent_dir(path: &Path) -> Result<(), DriveObjectStoreError> {
        let parent = path.parent().ok_or_else(|| {
            DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::Internal,
                "object path parent is missing",
            )
        })?;
        fs::create_dir_all(parent).map_err(|error| {
            DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::Internal,
                format!("create object parent dir failed: {error}"),
            )
        })
    }

    fn calculate_sha256_hex(body: &[u8]) -> String {
        sha256_hash(body)
    }

    fn now_epoch_ms() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis() as i64)
            .unwrap_or(0)
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

    fn normalize_continuation_token(
        token: Option<String>,
    ) -> Result<Option<String>, DriveObjectStoreError> {
        let Some(token) = token else {
            return Ok(None);
        };
        if token.trim().is_empty() {
            return Ok(None);
        }
        if token != token.trim() {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "continuation_token must be trimmed",
            ));
        }
        Self::validate_relative_segment(&token, "continuation_token")?;
        Ok(Some(token))
    }

    fn push_page_candidate(
        candidates: &mut BinaryHeap<ListedObjectPageCandidate>,
        candidate_limit: usize,
        sort_key: String,
        entry: ListedObjectPageEntry,
    ) {
        candidates.push(ListedObjectPageCandidate { sort_key, entry });
        if candidates.len() > candidate_limit {
            candidates.pop();
        }
    }

    fn push_prefix_page_candidate(
        candidates: &mut BinaryHeap<ListedObjectPageCandidate>,
        candidate_limit: usize,
        prefix: String,
    ) {
        if candidates
            .iter()
            .any(|candidate| matches!(&candidate.entry, ListedObjectPageEntry::Prefix(value) if value == &prefix))
        {
            return;
        }
        Self::push_page_candidate(
            candidates,
            candidate_limit,
            prefix.clone(),
            ListedObjectPageEntry::Prefix(prefix),
        );
    }

    fn common_prefix_for_object_key(
        object_key: &str,
        prefix: Option<&str>,
        delimiter: Option<&str>,
    ) -> Option<String> {
        let delimiter = delimiter?;
        let remainder = match prefix {
            Some(prefix) => object_key.strip_prefix(prefix).unwrap_or(object_key),
            None => object_key,
        };
        let delimiter_index = remainder.find(delimiter)?;
        let prefix_root = prefix.unwrap_or("");
        Some(format!(
            "{}{}{}",
            prefix_root,
            &remainder[..delimiter_index],
            delimiter,
        ))
    }

    fn collect_object_page(
        bucket_root: &Path,
        prefix: Option<&str>,
        delimiter: Option<&str>,
        continuation_token: Option<&str>,
        max_keys: usize,
    ) -> Result<CollectedObjectPage, DriveObjectStoreError> {
        let candidate_limit = max_keys.saturating_add(1);
        let mut candidates = BinaryHeap::<ListedObjectPageCandidate>::new();
        let mut pending_dirs = vec![bucket_root.to_path_buf()];

        while let Some(current) = pending_dirs.pop() {
            for entry in fs::read_dir(&current).map_err(|error| {
                DriveObjectStoreError::new(
                    DriveObjectStoreErrorKind::Internal,
                    format!("read bucket dir failed: {error}"),
                )
            })? {
                let entry = entry.map_err(|error| {
                    DriveObjectStoreError::new(
                        DriveObjectStoreErrorKind::Internal,
                        format!("read bucket entry failed: {error}"),
                    )
                })?;
                let file_type = entry.file_type().map_err(|error| {
                    DriveObjectStoreError::new(
                        DriveObjectStoreErrorKind::Internal,
                        format!("read bucket entry type failed: {error}"),
                    )
                })?;
                let path = entry.path();
                if file_type.is_dir() {
                    pending_dirs.push(path);
                    continue;
                }
                if !file_type.is_file() {
                    continue;
                }
                let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
                    continue;
                };
                if file_name.ends_with(".meta.json") {
                    continue;
                }
                let object_key = path
                    .strip_prefix(bucket_root)
                    .map_err(|error| {
                        DriveObjectStoreError::new(
                            DriveObjectStoreErrorKind::Internal,
                            format!("strip bucket prefix failed: {error}"),
                        )
                    })?
                    .to_string_lossy()
                    .replace('\\', "/");
                if prefix.is_some_and(|value| !object_key.starts_with(value)) {
                    continue;
                }
                if continuation_token.is_some_and(|value| object_key.as_str() <= value) {
                    continue;
                }
                if let Some(common_prefix) =
                    Self::common_prefix_for_object_key(&object_key, prefix, delimiter)
                {
                    if continuation_token.is_some_and(|value| common_prefix.as_str() <= value) {
                        continue;
                    }
                    Self::push_prefix_page_candidate(
                        &mut candidates,
                        candidate_limit,
                        common_prefix,
                    );
                    continue;
                }
                let stat = fs::metadata(&path).map_err(|error| {
                    DriveObjectStoreError::new(
                        DriveObjectStoreErrorKind::Internal,
                        format!("stat object failed: {error}"),
                    )
                })?;
                let object = ListedObject {
                    object_key,
                    content_length: stat.len(),
                    etag: None,
                    storage_class: None,
                    last_modified_epoch_ms: None,
                };
                Self::push_page_candidate(
                    &mut candidates,
                    candidate_limit,
                    object.object_key.clone(),
                    ListedObjectPageEntry::Object(object),
                );
            }
        }

        let mut candidates = candidates.into_vec().into_iter().collect::<Vec<_>>();
        candidates.sort_by(|left, right| left.sort_key.cmp(&right.sort_key));
        let next_continuation_token = if candidates.len() > max_keys {
            candidates.truncate(max_keys);
            candidates
                .last()
                .map(|candidate| candidate.sort_key.clone())
        } else {
            None
        };
        let mut items = Vec::new();
        let mut prefixes = Vec::new();
        for candidate in candidates {
            match candidate.entry {
                ListedObjectPageEntry::Object(object) => items.push(object),
                ListedObjectPageEntry::Prefix(prefix) => prefixes.push(prefix),
            }
        }
        Ok(CollectedObjectPage {
            items,
            prefixes,
            next_continuation_token,
        })
    }
}

#[async_trait]
impl DriveObjectStore for LocalDriveObjectStore {
    fn provider_kind(&self) -> DriveStorageProviderKind {
        DriveStorageProviderKind::LocalFilesystem
    }

    fn capabilities(&self) -> DriveStorageProviderCapabilities {
        DriveStorageProviderCapabilities::default_local_filesystem()
    }

    async fn put_object(
        &self,
        request: PutObjectRequest,
    ) -> Result<PutObjectResponse, DriveObjectStoreError> {
        let object_path = self.object_path(&request.locator)?;
        Self::ensure_parent_dir(&object_path)?;

        fs::write(&object_path, &request.body).map_err(|error| {
            DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::Internal,
                format!("write object failed: {error}"),
            )
        })?;

        let checksum = request
            .checksum_sha256_hex
            .or_else(|| Some(Self::calculate_sha256_hex(&request.body)));
        let metadata_payload = LocalObjectMetadata {
            content_type: request.content_type,
            metadata: request.metadata,
            checksum_sha256_hex: checksum,
        };
        let metadata_text = serde_json::to_string_pretty(&metadata_payload).map_err(|error| {
            DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::Internal,
                format!("serialize metadata failed: {error}"),
            )
        })?;
        let metadata_path = Self::metadata_path(&object_path);
        fs::write(metadata_path, metadata_text).map_err(|error| {
            DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::Internal,
                format!("write metadata failed: {error}"),
            )
        })?;

        Ok(PutObjectResponse {
            locator: request.locator,
            etag: None,
            version_id: None,
        })
    }

    async fn head_object(
        &self,
        request: HeadObjectRequest,
    ) -> Result<HeadObjectResponse, DriveObjectStoreError> {
        let object_path = self.object_path(&request.locator)?;
        let stat = fs::metadata(&object_path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                return DriveObjectStoreError::new(
                    DriveObjectStoreErrorKind::NotFound,
                    "object not found",
                );
            }
            DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::Internal,
                format!("stat object failed: {error}"),
            )
        })?;

        let metadata_path = Self::metadata_path(&object_path);
        let metadata = if metadata_path.exists() {
            let metadata_text = fs::read_to_string(&metadata_path).map_err(|error| {
                DriveObjectStoreError::new(
                    DriveObjectStoreErrorKind::Internal,
                    format!("read metadata failed: {error}"),
                )
            })?;
            serde_json::from_str::<LocalObjectMetadata>(&metadata_text).map_err(|error| {
                DriveObjectStoreError::new(
                    DriveObjectStoreErrorKind::Internal,
                    format!("parse metadata failed: {error}"),
                )
            })?
        } else {
            LocalObjectMetadata {
                content_type: None,
                metadata: std::collections::BTreeMap::new(),
                checksum_sha256_hex: None,
            }
        };

        Ok(HeadObjectResponse {
            locator: request.locator,
            content_length: stat.len(),
            content_type: metadata.content_type,
            etag: None,
            version_id: None,
            checksum_sha256_hex: metadata.checksum_sha256_hex,
            metadata: metadata.metadata,
        })
    }

    async fn delete_object(
        &self,
        request: DeleteObjectRequest,
    ) -> Result<DeleteObjectResponse, DriveObjectStoreError> {
        let object_path = self.object_path(&request.locator)?;
        let metadata_path = Self::metadata_path(&object_path);

        let deleted = if object_path.exists() {
            fs::remove_file(&object_path).map_err(|error| {
                DriveObjectStoreError::new(
                    DriveObjectStoreErrorKind::Internal,
                    format!("delete object failed: {error}"),
                )
            })?;
            true
        } else {
            false
        };

        if metadata_path.exists() {
            fs::remove_file(metadata_path).map_err(|error| {
                DriveObjectStoreError::new(
                    DriveObjectStoreErrorKind::Internal,
                    format!("delete metadata failed: {error}"),
                )
            })?;
        }

        Ok(DeleteObjectResponse {
            locator: request.locator,
            deleted,
        })
    }

    async fn head_bucket(
        &self,
        request: HeadBucketRequest,
    ) -> Result<HeadBucketResponse, DriveObjectStoreError> {
        let bucket_path = self.bucket_path(&request.bucket)?;
        Ok(HeadBucketResponse {
            bucket: request.bucket,
            exists: bucket_path.is_dir(),
        })
    }

    async fn list_buckets(
        &self,
        _request: ListBucketsRequest,
    ) -> Result<ListBucketsResponse, DriveObjectStoreError> {
        if !self.root_dir.exists() {
            return Ok(ListBucketsResponse { items: Vec::new() });
        }
        let mut items = Vec::new();
        for entry in fs::read_dir(&self.root_dir).map_err(|error| {
            DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::Internal,
                format!("read storage root failed: {error}"),
            )
        })? {
            let entry = entry.map_err(|error| {
                DriveObjectStoreError::new(
                    DriveObjectStoreErrorKind::Internal,
                    format!("read storage root entry failed: {error}"),
                )
            })?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let Some(bucket) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            items.push(ListedBucket {
                bucket,
                creation_date_epoch_ms: None,
            });
        }
        items.sort_by(|left, right| left.bucket.cmp(&right.bucket));
        Ok(ListBucketsResponse { items })
    }

    async fn create_bucket(
        &self,
        request: CreateBucketRequest,
    ) -> Result<CreateBucketResponse, DriveObjectStoreError> {
        let bucket_path = self.bucket_path(&request.bucket)?;
        fs::create_dir_all(&bucket_path).map_err(|error| {
            DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::Internal,
                format!("create bucket dir failed: {error}"),
            )
        })?;
        Ok(CreateBucketResponse {
            bucket: request.bucket,
            created: true,
        })
    }

    async fn delete_bucket(
        &self,
        request: DeleteBucketRequest,
    ) -> Result<DeleteBucketResponse, DriveObjectStoreError> {
        let bucket_path = self.bucket_path(&request.bucket)?;
        let deleted = if bucket_path.exists() {
            fs::remove_dir_all(&bucket_path).map_err(|error| {
                DriveObjectStoreError::new(
                    DriveObjectStoreErrorKind::Internal,
                    format!("delete bucket dir failed: {error}"),
                )
            })?;
            true
        } else {
            false
        };
        Ok(DeleteBucketResponse {
            bucket: request.bucket,
            deleted,
        })
    }

    async fn list_objects(
        &self,
        request: ListObjectsRequest,
    ) -> Result<ListObjectsResponse, DriveObjectStoreError> {
        let bucket_path = self.bucket_path(&request.bucket)?;
        let prefix = Self::normalize_list_prefix(request.prefix)?;
        if !bucket_path.exists() {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::NotFound,
                "bucket not found",
            ));
        }
        if request.max_keys == 0 {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "max_keys must be greater than zero",
            ));
        }
        let max_keys = usize::from(request.max_keys);
        let continuation_token = Self::normalize_continuation_token(request.continuation_token)?;
        let delimiter = Self::normalize_list_delimiter(request.delimiter)?;
        let CollectedObjectPage {
            items,
            prefixes,
            next_continuation_token,
        } = Self::collect_object_page(
            &bucket_path,
            prefix.as_deref(),
            delimiter.as_deref(),
            continuation_token.as_deref(),
            max_keys,
        )?;
        let is_truncated = next_continuation_token.is_some();
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
        let source_path = self.object_path(&request.source)?;
        let destination_path = self.object_path(&request.destination)?;
        Self::ensure_parent_dir(&destination_path)?;
        fs::copy(&source_path, &destination_path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                return DriveObjectStoreError::new(
                    DriveObjectStoreErrorKind::NotFound,
                    "source object not found",
                );
            }
            DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::Internal,
                format!("copy object failed: {error}"),
            )
        })?;
        let source_meta = Self::metadata_path(&source_path);
        if source_meta.exists() {
            let destination_meta = Self::metadata_path(&destination_path);
            fs::copy(source_meta, destination_meta).map_err(|error| {
                DriveObjectStoreError::new(
                    DriveObjectStoreErrorKind::Internal,
                    format!("copy object metadata failed: {error}"),
                )
            })?;
        }
        Ok(CopyObjectResponse {
            locator: request.destination,
            etag: None,
            version_id: None,
        })
    }

    async fn create_multipart_upload(
        &self,
        request: CreateMultipartUploadRequest,
    ) -> Result<CreateMultipartUploadResponse, DriveObjectStoreError> {
        let _ = request;
        Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::NotSupported,
            "local filesystem adapter does not support multipart upload",
        ))
    }

    async fn presign_upload_part(
        &self,
        request: PresignUploadPartRequest,
    ) -> Result<PresignedUploadPartResponse, DriveObjectStoreError> {
        let _ = request;
        Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::NotSupported,
            "local filesystem adapter does not support presigned upload part",
        ))
    }

    async fn complete_multipart_upload(
        &self,
        request: CompleteMultipartUploadRequest,
    ) -> Result<CompleteMultipartUploadResponse, DriveObjectStoreError> {
        let _ = request;
        Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::NotSupported,
            "local filesystem adapter does not support complete multipart upload",
        ))
    }

    async fn abort_multipart_upload(
        &self,
        request: AbortMultipartUploadRequest,
    ) -> Result<(), DriveObjectStoreError> {
        let _ = request;
        Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::NotSupported,
            "local filesystem adapter does not support abort multipart upload",
        ))
    }

    async fn presign_download(
        &self,
        request: PresignDownloadRequest,
    ) -> Result<PresignedDownloadResponse, DriveObjectStoreError> {
        Self::validate_presign_expiry(request.expires_in_seconds)?;
        let object_path = self.object_path(&request.locator)?;
        if !object_path.exists() {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::NotFound,
                "object not found",
            ));
        }
        Ok(PresignedDownloadResponse {
            method: "GET".to_string(),
            url: format!(
                "file://{}",
                object_path.to_string_lossy().replace('\\', "/")
            ),
            headers: std::collections::BTreeMap::new(),
            expires_at_epoch_ms: Self::now_epoch_ms()
                + i64::from(request.expires_in_seconds) * 1000,
        })
    }

    async fn read_object_range(
        &self,
        request: ReadObjectRangeRequest,
    ) -> Result<(ReadObjectRangeResponse, Box<dyn DriveObjectChunkStream>), DriveObjectStoreError>
    {
        let object_path = self.object_path(&request.locator)?;
        let mut file = std::fs::File::open(&object_path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                return DriveObjectStoreError::new(
                    DriveObjectStoreErrorKind::NotFound,
                    "object not found",
                );
            }
            DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::Internal,
                format!("open object failed: {error}"),
            )
        })?;

        let total_len = file
            .metadata()
            .map_err(|error| {
                DriveObjectStoreError::new(
                    DriveObjectStoreErrorKind::Internal,
                    format!("read object metadata failed: {error}"),
                )
            })?
            .len();

        if request.range.start_inclusive >= total_len
            || request.range.end_inclusive >= total_len
            || request.range.start_inclusive > request.range.end_inclusive
        {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "byte range is out of bounds",
            ));
        }

        let start = request.range.start_inclusive;
        let end = request.range.end_inclusive;
        let byte_len = end
            .checked_sub(start)
            .and_then(|delta| delta.checked_add(1))
            .ok_or_else(|| {
                DriveObjectStoreError::new(
                    DriveObjectStoreErrorKind::InvalidRequest,
                    "byte range is out of bounds",
                )
            })? as usize;

        file.seek(SeekFrom::Start(start)).map_err(|error| {
            DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::Internal,
                format!("seek object failed: {error}"),
            )
        })?;

        let mut slice = vec![0_u8; byte_len];
        file.read_exact(&mut slice).map_err(|error| {
            DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::Internal,
                format!("read object range failed: {error}"),
            )
        })?;

        let response = ReadObjectRangeResponse {
            locator: request.locator,
            content_type: None,
            etag: None,
            content_length: slice.len() as u64,
        };
        Ok((response, Box::new(SingleChunkStream { body: Some(slice) })))
    }
}
