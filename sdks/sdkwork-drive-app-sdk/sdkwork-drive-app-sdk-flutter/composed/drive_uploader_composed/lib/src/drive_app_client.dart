import 'package:sdkwork_drive_app_sdk_generated_flutter/sdkwork_drive_app_sdk_generated_flutter.dart';

import 'uploader_client.dart';
import 'uploader_state_store.dart';

/// Composed Drive app client: the generated transport plus the composed
/// uploader, wired once for a host application.
///
/// Tokens can be rotated after construction (session changes) through
/// [updateTokens]; callers never assemble auth headers themselves.
class DriveAppClient {
  DriveAppClient._({
    required SdkworkAppClient generated,
    required DriveUploaderClient uploader,
  })  : _generated = generated,
        uploader = uploader;

  factory DriveAppClient.withBaseUrl({
    required String baseUrl,
    String? authToken,
    String? accessToken,
    Map<String, String>? headers,
    int timeout = 30000,
    DriveUploaderStateStore? uploaderStateStore,
    int defaultChunkSizeBytes = 8 * 1024 * 1024,
  }) {
    final generated = SdkworkAppClient.withBaseUrl(
      baseUrl: baseUrl,
      authToken: authToken,
      accessToken: accessToken,
      headers: headers,
      timeout: timeout,
    );
    final uploader = DriveUploaderClient(
      client: generated,
      stateStore: uploaderStateStore,
      defaultChunkSizeBytes: defaultChunkSizeBytes,
    );
    return DriveAppClient._(generated: generated, uploader: uploader);
  }

  final SdkworkAppClient _generated;

  /// Composed Drive Uploader entry (`client.uploader.*` parity).
  final DriveUploaderClient uploader;

  /// Generated Drive App API surface (spaces, nodes, uploader, sessions,
  /// download grants, ...).
  DriveApi get drive => _generated.drive;

  /// Syncs the authenticated session's token pair after session changes.
  void updateTokens({String? authToken, String? accessToken}) {
    if (authToken != null) {
      _generated.setAuthToken(authToken);
    }
    if (accessToken != null) {
      _generated.setAccessToken(accessToken);
    }
  }
}
