//! `list_buckets` region resolution against a stub S3 endpoint.
//!
//! Three channels are in play and only a stub can show which one answered:
//!
//! * the SDK's own `<BucketRegion>` element (what AWS sends) — no extra call;
//! * the vendor's `<Location>` element, which the SDK's typed bucket drops and
//!   the inventory client's HTTP layer therefore keeps a copy of
//!   (`sdkwork_drive_storage_s3::InventoryRegionCapture`) — no extra call either;
//! * `GetBucketLocation`, the bucket-scoped fallback, which resolves a bucket
//!   only when the endpoint that asks is in that bucket's own region (a
//!   cross-region ask is answered `404 NoSuchBucket`; probed against real COS
//!   endpoints, and the reason the fallback is bounded and best effort).
//!
//! The stub is plain `std::net`: this crate has no HTTP test harness, and the
//! SDK needs nothing more than a listener that speaks HTTP/1.1 with
//! `Connection: close`, so every request arrives on its own connection. It also
//! records the request targets, which is what proves a bucket the inventory
//! already answered is never asked about again.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use sdkwork_drive_storage_contract::{
    DriveObjectStore, DriveStorageProviderKind, ListBucketsRequest,
};
use sdkwork_drive_storage_s3::{S3DriveObjectStore, S3ProviderProfile, S3StoreConfig};

/// One bucket per channel: the legacy vendor element only, the AWS element only,
/// the bucket-scoped fallback, and one the vendor refuses.
const INVENTORY: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListAllMyBucketsResult>
  <Owner><ID>stub</ID><DisplayName>stub</DisplayName></Owner>
  <Buckets>
    <Bucket>
      <Name>cos-bucket</Name>
      <Location>ap-beijing</Location>
      <CreationDate>2024-01-02T00:00:00.000Z</CreationDate>
    </Bucket>
    <Bucket>
      <Name>legacy-bucket</Name>
      <CreationDate>2024-01-02T00:00:00.000Z</CreationDate>
    </Bucket>
    <Bucket>
      <Name>modern-bucket</Name>
      <BucketRegion>eu-west-1</BucketRegion>
      <CreationDate>2024-01-02T00:00:00.000Z</CreationDate>
    </Bucket>
    <Bucket>
      <Name>denied-bucket</Name>
      <CreationDate>2024-01-02T00:00:00.000Z</CreationDate>
    </Bucket>
  </Buckets>
</ListAllMyBucketsResult>"#;

const LEGACY_LOCATION: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LocationConstraint>ap-guangzhou</LocationConstraint>"#;

/// Start a stub endpoint; hand back its address plus the request log.
fn spawn_stub() -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("stub binds");
    let address = listener.local_addr().expect("stub address");
    let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let recorder = Arc::clone(&log);
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            let recorder = Arc::clone(&recorder);
            thread::spawn(move || serve(stream, recorder));
        }
    });
    (format!("http://{address}"), log)
}

fn serve(mut stream: TcpStream, log: Arc<Mutex<Vec<String>>>) {
    let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }
    // Drain the headers so the client finishes writing before the answer.
    loop {
        let mut header = String::new();
        match reader.read_line(&mut header) {
            Ok(0) => break,
            Ok(_) if header == "\r\n" => break,
            Ok(_) => continue,
            Err(_) => return,
        }
    }

    let target = request_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("/")
        .to_string();
    if let Ok(mut entries) = log.lock() {
        entries.push(target.clone());
    }

    let response = if target.contains("legacy-bucket") && target.contains("location") {
        reply(200, "OK", LEGACY_LOCATION)
    } else if target.contains("denied-bucket") && target.contains("location") {
        // A vendor that will not answer (missing permission, unsupported
        // operation): the row must stay empty rather than inherit a guess.
        reply(
            403,
            "Forbidden",
            "<Error><Code>AccessDenied</Code><Message>denied</Message></Error>",
        )
    } else if target.contains("location") {
        // `cos-bucket` and `modern-bucket` are answered by the inventory itself;
        // asking again would mean one of those channels silently stopped working.
        reply(
            500,
            "Internal Server Error",
            "<Error><Code>UnexpectedLookup</Code></Error>",
        )
    } else {
        reply(200, "OK", INVENTORY)
    };

    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn reply(status: u16, reason: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/xml\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

/// A store whose every call lands on the stub, whatever the ambient AWS config
/// says (an explicit endpoint wins over the developer's own environment).
fn config_for(endpoint: String) -> S3StoreConfig {
    S3StoreConfig {
        provider_kind: DriveStorageProviderKind::S3Compatible,
        provider_profile: S3ProviderProfile::Minio,
        endpoint: Some(endpoint),
        region: "us-east-1".to_string(),
        default_bucket: "legacy-bucket".to_string(),
        access_key_id: "stub-access-key".to_string(),
        secret_access_key: "stub-secret-key".to_string(),
        session_token: None,
        force_path_style: true,
        strict_tls: false,
    }
}

#[tokio::test]
async fn list_buckets_resolves_each_region_from_the_channel_that_has_it() {
    let (endpoint, log) = spawn_stub();
    let store = S3DriveObjectStore::new(config_for(endpoint))
        .await
        .expect("store builds");
    let listed = store
        .list_buckets(ListBucketsRequest)
        .await
        .expect("inventory reads");

    let region = |bucket: &str| {
        listed
            .items
            .iter()
            .find(|item| item.bucket == bucket)
            .unwrap_or_else(|| panic!("{bucket} is missing from the inventory"))
            .region
            .clone()
    };

    // COS/OSS shape: the vendor publishes the region as `<Location>`, which the
    // SDK's typed bucket drops and the inventory client's HTTP layer keeps.
    assert_eq!(region("cos-bucket"), Some("ap-beijing".to_string()));
    // AWS shape: the SDK parses `<BucketRegion>` itself.
    assert_eq!(region("modern-bucket"), Some("eu-west-1".to_string()));
    // Neither channel answered, so the bucket-scoped lookup runs — and answers,
    // because the stub happens to be in the bucket's own region.
    assert_eq!(region("legacy-bucket"), Some("ap-guangzhou".to_string()));
    // The vendor refused: the row keeps no region instead of a guess, and the
    // inventory still returns.
    assert_eq!(region("denied-bucket"), None);
    assert_eq!(listed.items.len(), 4);

    // Only the buckets with no region on the inventory are asked about: a
    // `<Location>` the vendor already sent must never cost a round trip.
    let requests = log.lock().expect("request log").clone();
    let lookups: Vec<String> = requests
        .iter()
        .filter(|target| target.contains("location"))
        .cloned()
        .collect();
    assert_eq!(
        lookups.len(),
        2,
        "two bucket-scoped lookups are expected, got {lookups:?}"
    );
    assert!(
        lookups.iter().all(|target| !target.contains("cos-bucket")
            && !target.contains("modern-bucket")),
        "a bucket the inventory answered must not be looked up again: {lookups:?}"
    );
}
