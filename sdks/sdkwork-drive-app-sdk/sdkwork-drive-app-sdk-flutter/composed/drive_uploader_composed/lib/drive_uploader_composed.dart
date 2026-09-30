/// Composed Drive App SDK for Flutter: the Drive Uploader facade over the
/// generator-owned `sdkwork_drive_app_sdk_generated_flutter` transport.
///
/// Uploads enter Drive only through `DriveUploaderClient` (`DRIVE_SPEC.md`
/// sections 8.1 and 9); feature code must not reimplement upload-session
/// orchestration or call raw Drive uploader endpoints.
library drive_uploader_composed;

export 'src/drive_app_client.dart';
export 'src/types.dart';
export 'src/uploader_state_store.dart';
export 'src/uploader_client.dart';

// Hosts needing the generated surface directly can also import the generated
// package; SdkConfig re-exported here keeps single-import ergonomics.
export 'package:sdkwork_common_flutter/sdkwork_common_flutter.dart'
    show SdkConfig;
export 'package:sdkwork_drive_app_sdk_generated_flutter/sdkwork_drive_app_sdk_generated_flutter.dart'
    show SdkworkAppClient;
