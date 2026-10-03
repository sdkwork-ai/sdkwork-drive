//! Bucket-region discovery from the account bucket inventory.
//!
//! The region of a bucket is the vendor's answer, and the account inventory is
//! where the vendor gives it — but not in the element the S3 SDK reads.
//! `ListBuckets` answers with one `<Bucket>` per bucket, and the shape the SDK
//! generates from the AWS model carries `Name`, `CreationDate`, `BucketRegion`
//! and `BucketArn` only (verified against the generated `shape_bucket.rs` of
//! aws-sdk-s3 1.152), so a vendor that publishes the region in `<Location>` —
//! Tencent COS, Alibaba Cloud OSS — has that element dropped before the typed
//! output is built.
//!
//! Nothing else carries it for those vendors. `GetBucketLocation` is a
//! *bucket-scoped* operation: it must be sent to an endpoint in the bucket's own
//! region, and the configured endpoint answers `404 NoSuchBucket` for a bucket
//! that lives elsewhere — probed against `cos.ap-guangzhou.myqcloud.com` for an
//! `ap-beijing` bucket — with no redirect and no region header to follow. The
//! account-level service host refuses bucket operations (`400 InvalidArgument`)
//! and the global acceleration host requires per-bucket acceleration
//! (`BucketAccelerateNotEnabled`), so neither can be asked either. That lookup
//! stays as the last channel, for exactly the buckets it can resolve.
//!
//! The one channel that always has the answer is the inventory response body, so
//! this module reads it: the inventory client gets an [`HttpClient`] that wraps
//! the SDK's **own** default connector (`default_connector`, i.e. hyper 1.x +
//! rustls 0.23 through the `rustls-aws-lc` feature the SDK already enables — no
//! second TLS stack), buffers each response, keeps a copy, and hands the very
//! same bytes back so the SDK deserializes exactly what it would have.
//! [`bucket_locations`] then reads `<Location>` with the same XML decoder the SDK
//! uses for the rest of the shape.
//!
//! It is attached to the inventory client only — the one that never serves
//! object reads or writes — so no transfer ever pays for a buffered body.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use aws_sdk_s3::config::http::HttpRequest;
use aws_sdk_s3::config::HttpClient;
use aws_sdk_s3::primitives::{ByteStream, SdkBody};
use aws_smithy_http_client::default_connector;
use aws_smithy_runtime_api::client::http::{
    HttpConnector, HttpConnectorFuture, HttpConnectorSettings, SharedHttpConnector,
};
use aws_smithy_runtime_api::client::result::ConnectorError;
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_xml::decode::{try_data, Document};

/// Keeps the raw XML of the last inventory response its client answered.
///
/// Cloning shares the slot: the store keeps one clone to read the answer, the
/// HTTP layer holds another.
#[derive(Debug, Clone, Default)]
pub struct InventoryRegionCapture {
    xml: Arc<Mutex<Option<Vec<u8>>>>,
}

impl InventoryRegionCapture {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn record(&self, body: Vec<u8>) {
        if let Ok(mut slot) = self.xml.lock() {
            *slot = Some(body);
        }
    }

    /// Bucket name → region, as the vendor wrote it in `<Location>`.
    ///
    /// Takes the captured body with it: the answer belongs to the call that just
    /// finished, and a later call must not read a stale inventory. An empty map
    /// means the vendor did not publish locations (AWS sends `<BucketRegion>`,
    /// which the SDK parses itself) or the body could not be read — either way
    /// the caller falls back to its other channels.
    #[must_use]
    pub fn take_bucket_locations(&self) -> BTreeMap<String, String> {
        let captured = self.xml.lock().ok().and_then(|mut slot| slot.take());
        captured.map_or_else(BTreeMap::new, |xml| bucket_locations(&xml))
    }
}

/// HTTP client for the inventory call: the SDK's default connector with the
/// response body captured on the way past.
///
/// Only ever installed on the inventory client — the SDK's own connector is used
/// for every object read and write, where buffering a body would copy a transfer
/// into memory for nothing.
#[derive(Debug, Clone)]
pub struct InventoryHttpClient {
    capture: InventoryRegionCapture,
}

impl InventoryHttpClient {
    #[must_use]
    pub fn new(capture: InventoryRegionCapture) -> Self {
        Self { capture }
    }
}

impl HttpClient for InventoryHttpClient {
    fn http_connector(
        &self,
        settings: &HttpConnectorSettings,
        components: &RuntimeComponents,
    ) -> SharedHttpConnector {
        /*
         * `default_connector` is the SDK's own construction of hyper 1.x +
         * rustls 0.23 (this crate enables the same `rustls-aws-lc` feature the
         * SDK's `default-https-client` does), so the inventory call keeps exactly
         * the transport, proxy handling, timeouts and certificate set every
         * other call gets — it is observed, never replaced.
         *
         * A build without any TLS provider is the one case where it returns
         * `None`; the inventory call then fails with a plain connector error,
         * which is what such a build does for every other request too.
         */
        let inner = default_connector(settings, components.sleep_impl())
            .unwrap_or_else(|| SharedHttpConnector::new(UnavailableConnector));
        SharedHttpConnector::new(InventoryBodyConnector {
            inner,
            capture: self.capture.clone(),
        })
    }
}

#[derive(Debug, Clone)]
struct InventoryBodyConnector {
    inner: SharedHttpConnector,
    capture: InventoryRegionCapture,
}

impl HttpConnector for InventoryBodyConnector {
    fn call(&self, request: HttpRequest) -> HttpConnectorFuture {
        let inner = self.inner.clone();
        let capture = self.capture.clone();
        HttpConnectorFuture::new(async move {
            let mut response = inner.call(request).await?;
            /*
             * The body is consumed and immediately replaced by a buffered copy of
             * the same bytes: the SDK's deserializer still sees the response it
             * was handed, and only the inventory call runs this. A body that
             * cannot be read is left empty — that response was already broken,
             * and the capture stays without an answer so the caller falls back.
             */
            let streaming = std::mem::replace(response.body_mut(), SdkBody::empty());
            if let Ok(collected) = ByteStream::new(streaming).collect().await {
                let bytes = collected.into_bytes();
                capture.record(bytes.to_vec());
                *response.body_mut() = SdkBody::from(bytes);
            }
            Ok(response)
        })
    }
}

/// Stand-in connector for a build with no TLS provider compiled in.
#[derive(Debug, Clone)]
struct UnavailableConnector;

impl HttpConnector for UnavailableConnector {
    fn call(&self, _request: HttpRequest) -> HttpConnectorFuture {
        HttpConnectorFuture::new(async move {
            Err(ConnectorError::other(
                "no HTTPS connector is compiled into this build".into(),
                None,
            ))
        })
    }
}

/// Region per bucket, read from a `ListBuckets` response body.
///
/// Mirrors the traversal the SDK's own generated deserializer uses
/// (`Document::try_from` → `root_element` → `next_tag` → `try_data`), so the
/// vendor's element is read by the same rules as every element around it.
/// Buckets whose `<Location>` is absent or empty are simply not in the map.
#[must_use]
pub fn bucket_locations(xml: &[u8]) -> BTreeMap<String, String> {
    let Ok(mut document) = Document::try_from(xml) else {
        return BTreeMap::new();
    };
    let Ok(mut root) = document.root_element() else {
        return BTreeMap::new();
    };

    let mut locations = BTreeMap::new();
    while let Some(mut buckets) = root.next_tag() {
        if !buckets.start_el().matches("Buckets") {
            continue;
        }
        while let Some(mut bucket) = buckets.next_tag() {
            if !bucket.start_el().matches("Bucket") {
                continue;
            }
            let mut name: Option<String> = None;
            let mut location: Option<String> = None;
            while let Some(mut field) = bucket.next_tag() {
                let slot = match field.start_el().local() {
                    "Name" => &mut name,
                    "Location" => &mut location,
                    _ => continue,
                };
                if let Ok(value) = try_data(&mut field) {
                    let value = value.trim();
                    if !value.is_empty() {
                        *slot = Some(value.to_string());
                    }
                }
            }
            if let (Some(name), Some(location)) = (name, location) {
                locations.insert(name, location);
            }
        }
    }
    locations
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shape Tencent COS answers `GET Service` with: the region is
    /// `<Location>`, exactly the element the SDK's `Bucket` shape does not have.
    const COS_INVENTORY: &str = r#"<?xml version='1.0' encoding='utf-8' ?>
<ListAllMyBucketsResult>
  <Owner><ID>qcs::cam::uin/100000000001:uin/100000000001</ID></Owner>
  <Buckets>
    <Bucket>
      <Name>ai-1253947560</Name>
      <Location>ap-beijing</Location>
      <CreationDate>2024-06-23T11:52:51Z</CreationDate>
    </Bucket>
    <Bucket>
      <Name>image2-1253947560</Name>
      <Location>ap-guangzhou</Location>
      <CreationDate>2019-11-27T09:01:24Z</CreationDate>
    </Bucket>
    <Bucket>
      <Name>no-location-1253947560</Name>
      <CreationDate>2020-01-01T00:00:00Z</CreationDate>
    </Bucket>
  </Buckets>
</ListAllMyBucketsResult>"#;

    #[test]
    fn reads_the_region_vendors_publish_as_location() {
        let locations = bucket_locations(COS_INVENTORY.as_bytes());

        assert_eq!(locations.get("ai-1253947560"), Some(&"ap-beijing".to_string()));
        assert_eq!(
            locations.get("image2-1253947560"),
            Some(&"ap-guangzhou".to_string())
        );
        // No `<Location>`: the row keeps no region rather than an empty string
        // that would render as a blank cell pretending to be an answer.
        assert_eq!(locations.get("no-location-1253947560"), None);
    }

    #[test]
    fn an_aws_inventory_simply_yields_nothing() {
        // AWS answers `<BucketRegion>` (which the SDK parses itself) and no
        // `<Location>`; this reader must not invent one from the surrounding
        // elements.
        let aws = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListAllMyBucketsResult>
  <Buckets>
    <Bucket>
      <Name>modern-bucket</Name>
      <BucketRegion>eu-west-1</BucketRegion>
      <BucketArn>arn:aws:s3:::modern-bucket</BucketArn>
      <CreationDate>2024-01-02T00:00:00.000Z</CreationDate>
    </Bucket>
  </Buckets>
</ListAllMyBucketsResult>"#;

        assert!(bucket_locations(aws.as_bytes()).is_empty());
    }

    #[test]
    fn survives_a_body_that_is_not_the_inventory_at_all() {
        // An error document, a truncated body, or a non-UTF-8 payload must read
        // as "no locations", never as a panic on the inventory path.
        assert!(bucket_locations(b"").is_empty());
        assert!(bucket_locations(b"not xml at all").is_empty());
        assert!(bucket_locations(b"<Error><Code>AccessDenied</Code></Error>").is_empty());
        assert!(bucket_locations(&[0xff, 0xfe, 0xfd]).is_empty());
    }
}
