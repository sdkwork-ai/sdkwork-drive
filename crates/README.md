# Rust Crates

`crates/` contains Rust service crates, route crates, workers, host/server crates, and reusable Rust libraries for `sdkwork-drive`.

Current layout:

- `sdkwork-drive-config/`: Database configuration.
- `sdkwork-drive-contract/`: Drive contract types and tests.
- `sdkwork-drive-http/`: HTTP utilities.
- `sdkwork-drive-security/`: Security context and validation.
- `sdkwork-drive-observability/`: Telemetry and instrumentation.
- `sdkwork-drive-storage-contract/`: Storage provider contract (DriveObjectStore trait).
- `sdkwork-drive-storage-local/`: Local filesystem storage provider.
- `sdkwork-drive-storage-opendal/`: OpenDAL-based S3 storage provider.
- `sdkwork-drive-storage-s3/`: Native AWS SDK S3 storage provider.
- `sdkwork-drive-test-support/`: Test utilities and fixtures.
- `sdkwork-drive-workspace-service/`: Workspace service.
- `sdkwork-routes-drive-open-api/`: Open API route crate.
- `sdkwork-routes-drive-app-api/`: App API route crate.
- `sdkwork-routes-drive-backend-api/`: Backend API route crate.
- `sdkwork-routes-storage-backend-api/`: Storage backend API route crate.
- `sdkwork-drive-install-worker/`: Install worker.
- `sdkwork-api-drive-standalone-gateway/`: Standalone gateway (embedded IAM + Drive API proxy loop).
