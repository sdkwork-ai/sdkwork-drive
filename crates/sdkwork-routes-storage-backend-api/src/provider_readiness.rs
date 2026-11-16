use sdkwork_drive_storage_contract::{
    AbortMultipartUploadRequest, CreateMultipartUploadRequest, DeleteObjectRequest, DriveByteRange,
    DriveObjectLocator, DriveObjectStore, HeadObjectRequest, PresignDownloadRequest,
    PresignUploadPartRequest, PutObjectRequest, ReadObjectRangeRequest,
};
use std::time::{SystemTime, UNIX_EPOCH};

/// Bytes written by the readiness probe. Kept tiny: this only proves the
/// provider accepts, returns and deletes an object with the configured account.
const PROBE_BODY: &[u8] = b"sdkwork-drive-initialization-probe";

/// Result of the one-click storage readiness probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StorageProviderReadiness {
    pub object_write_verified: bool,
    pub object_read_verified: bool,
    pub object_delete_verified: bool,
    pub multipart_verified: bool,
    pub presigned_upload_verified: bool,
    pub presigned_download_verified: bool,
    pub ready: bool,
    pub readiness_error: Option<String>,
}

/// Exercise the operations every Drive storage workflow depends on:
/// write, head, range read, multipart create/abort, and delete.
///
/// A failure is reported in `readiness_error` rather than propagated, because
/// the bucket, default binding and logical plane may already be initialized and
/// the console still needs their counts. `ready` stays false so the operator
/// does not mistake a partial initialization for a working storage plane.
pub(crate) async fn probe_storage_readiness(
    object_store: &dyn DriveObjectStore,
    bucket: &str,
    tenant_id: &str,
) -> StorageProviderReadiness {
    let probe_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or(0);
    let locator = DriveObjectLocator {
        bucket: bucket.to_string(),
        object_key: format!(
            "sdkwork-drive/v1/tenants/{tenant_id}/.sdkwork-initialization/probe-{probe_id}"
        ),
    };
    let mut readiness = StorageProviderReadiness {
        object_write_verified: false,
        object_read_verified: false,
        object_delete_verified: false,
        multipart_verified: false,
        presigned_upload_verified: false,
        presigned_download_verified: false,
        ready: false,
        readiness_error: None,
    };

    let put = object_store
        .put_object(PutObjectRequest {
            locator: locator.clone(),
            content_type: Some("application/octet-stream".to_string()),
            metadata: Default::default(),
            body: PROBE_BODY.to_vec(),
            checksum_sha256_hex: None,
        })
        .await;
    if let Err(error) = put {
        readiness.readiness_error = Some(format!("object write failed: {error}"));
        return readiness;
    }
    readiness.object_write_verified = true;

    let head = object_store
        .head_object(HeadObjectRequest {
            locator: locator.clone(),
        })
        .await;
    match head {
        Ok(head) if head.content_length == PROBE_BODY.len() as u64 => {
            readiness.object_read_verified = true;
        }
        Ok(head) => {
            readiness.readiness_error = Some(format!(
                "object head returned unexpected length {} (expected {})",
                head.content_length,
                PROBE_BODY.len()
            ));
        }
        Err(error) => {
            readiness.readiness_error = Some(format!("object head failed: {error}"));
        }
    }

    if readiness.object_read_verified {
        let read = object_store
            .read_object_range(ReadObjectRangeRequest {
                locator: locator.clone(),
                range: DriveByteRange {
                    start_inclusive: 0,
                    end_inclusive: PROBE_BODY.len() as u64 - 1,
                },
            })
            .await;
        match read {
            Ok((_, mut stream)) => {
                let mut body = Vec::new();
                loop {
                    match stream.next_chunk().await {
                        Ok(Some(chunk)) => body.extend_from_slice(&chunk),
                        Ok(None) => break,
                        Err(error) => {
                            readiness.readiness_error =
                                Some(format!("object read failed: {error}"));
                            break;
                        }
                    }
                }
                if readiness.readiness_error.is_none() && body == PROBE_BODY {
                    // Head already proved the object exists; a matching range
                    // read proves download works end to end.
                } else if readiness.readiness_error.is_none() {
                    readiness.object_read_verified = false;
                    readiness.readiness_error =
                        Some("object read returned different content".to_string());
                }
            }
            Err(error) => {
                readiness.object_read_verified = false;
                readiness.readiness_error = Some(format!("object read failed: {error}"));
            }
        }
    }

    let capabilities = object_store.capabilities();
    if capabilities.supports_presigned_download {
        match object_store
            .presign_download(PresignDownloadRequest {
                locator: locator.clone(),
                expires_in_seconds: 300,
            })
            .await
        {
            Ok(presigned) => readiness.presigned_download_verified = !presigned.url.is_empty(),
            Err(error) => {
                if readiness.readiness_error.is_none() {
                    readiness.readiness_error = Some(format!("presigned download failed: {error}"));
                }
            }
        }
    } else if readiness.readiness_error.is_none() {
        readiness.readiness_error = Some(
            "provider does not support presigned downloads required by Drive read workflows"
                .to_string(),
        );
    }

    match object_store
        .delete_object(DeleteObjectRequest {
            locator: locator.clone(),
        })
        .await
    {
        Ok(deleted) if deleted.deleted => readiness.object_delete_verified = true,
        Ok(_) => {
            if readiness.readiness_error.is_none() {
                readiness.readiness_error = Some("object delete was not confirmed".to_string());
            }
        }
        Err(error) => {
            if readiness.readiness_error.is_none() {
                readiness.readiness_error = Some(format!("object delete failed: {error}"));
            }
        }
    }

    let capabilities = object_store.capabilities();
    if !capabilities.supports_multipart_upload || !capabilities.supports_presigned_upload_part {
        if readiness.readiness_error.is_none() {
            readiness.readiness_error = Some(
                "provider does not support the multipart/presigned-upload capabilities Drive upload sessions require"
                    .to_string(),
            );
        }
    } else {
        let multipart_key = format!("{}.multipart", locator.object_key);
        let created = object_store
            .create_multipart_upload(CreateMultipartUploadRequest {
                locator: DriveObjectLocator {
                    bucket: bucket.to_string(),
                    object_key: multipart_key.clone(),
                },
                content_type: Some("application/octet-stream".to_string()),
                metadata: Default::default(),
                checksum_sha256_hex: None,
            })
            .await;
        match created {
            Ok(created) => {
                let presigned = object_store
                    .presign_upload_part(PresignUploadPartRequest {
                        locator: DriveObjectLocator {
                            bucket: bucket.to_string(),
                            object_key: multipart_key.clone(),
                        },
                        upload_id: created.upload_id.clone(),
                        part_number: 1,
                        expires_in_seconds: 3600,
                    })
                    .await;
                readiness.presigned_upload_verified =
                    presigned.is_ok_and(|presigned| !presigned.url.is_empty());
                if !readiness.presigned_upload_verified && readiness.readiness_error.is_none() {
                    readiness.readiness_error =
                        Some("presigned upload part signing failed".to_string());
                }
                readiness.multipart_verified = object_store
                    .abort_multipart_upload(AbortMultipartUploadRequest {
                        locator: DriveObjectLocator {
                            bucket: bucket.to_string(),
                            object_key: multipart_key,
                        },
                        upload_id: created.upload_id,
                    })
                    .await
                    .is_ok();
                if !readiness.multipart_verified && readiness.readiness_error.is_none() {
                    readiness.readiness_error =
                        Some("multipart upload could not be aborted".to_string());
                }
            }
            Err(error) => {
                if readiness.readiness_error.is_none() {
                    readiness.readiness_error = Some(format!("multipart upload failed: {error}"));
                }
            }
        }
    }

    readiness.ready = readiness.object_write_verified
        && readiness.object_read_verified
        && readiness.object_delete_verified
        && readiness.multipart_verified
        && readiness.presigned_upload_verified
        && readiness.presigned_download_verified;
    readiness
}
