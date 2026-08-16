use std::path::PathBuf;

use serde_json::Value;

fn workspace_root() -> PathBuf {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.pop();
    root
}

#[test]
fn openapi_paths_follow_sdkwork_v3_prefixes() {
    let open = std::fs::read_to_string(
        workspace_root().join("apis/open-api/drive/drive-open-api.openapi.json"),
    )
    .expect("open openapi missing");
    let app = std::fs::read_to_string(
        workspace_root().join("apis/app-api/drive/drive-app-api.openapi.json"),
    )
    .expect("app openapi missing");
    let backend = std::fs::read_to_string(
        workspace_root().join("apis/backend-api/drive/drive-backend-api.openapi.json"),
    )
    .expect("backend openapi missing");
    let admin_storage = std::fs::read_to_string(
        workspace_root().join("apis/backend-api/drive/drive-admin-storage-api.openapi.json"),
    )
    .expect("admin storage openapi missing");
    assert!(open.contains("\"title\": \"SDKWork Drive Open API\""));
    assert!(open.contains("/open/v3/api/drive/share_links/{token}"));
    assert!(open.contains("/open/v3/api/drive/share_links/{token}/download_url"));
    assert!(open.contains("\"operationId\": \"openShareLinks.retrieve\""));
    assert!(open.contains("\"operationId\": \"openShareLinks.downloadUrls.create\""));
    assert!(open.contains("\"DriveOpenShareLink\""));
    assert!(app.contains("/app/v3/api/drive/spaces"));
    assert!(app.contains("/app/v3/api/drive/spaces/{spaceId}"));
    assert!(app.contains("/app/v3/api/drive/spaces/{spaceId}/website_roots"));
    assert!(app.contains("/app/v3/api/drive/website_roots/{rootUuid}"));
    assert!(app.contains("\"title\": \"SDKWork Drive App API\""));
    assert!(app.contains("/app/v3/api/drive/spaces/{spaceId}/nodes"));
    assert!(app.contains("/app/v3/api/drive/nodes/folders"));
    assert!(app.contains("/app/v3/api/drive/nodes/files"));
    assert!(app.contains("/app/v3/api/drive/nodes/shortcuts"));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}"));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}/path"));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}/capabilities"));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}/properties"));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}/properties/{propertyKey}"));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}/labels"));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}/labels/{labelId}"));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}/move"));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}/copy"));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}/trash"));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}/download_url"));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}/download_grants"));
    assert!(app.contains("\"operationId\": \"downloadGrants.create\""));
    assert!(app.contains("/app/v3/api/drive/trash/{nodeId}/restore"));
    assert!(app.contains("/app/v3/api/drive/trash"));
    assert!(app.contains("/app/v3/api/drive/trash/empty"));
    assert!(app.contains("/app/v3/api/drive/recent"));
    assert!(app.contains("/app/v3/api/drive/shared_with_me"));
    assert!(app.contains("/app/v3/api/drive/favorites"));
    assert!(app.contains("/app/v3/api/drive/quotas/summary"));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}/favorite"));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}/versions"));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}/versions/{versionId}/restore"));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}/permissions"));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}/permissions/{permissionId}"));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}/share_links"));
    assert!(app.contains("/app/v3/api/drive/share_links/{shareLinkId}"));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}/comments"));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}/comments/{commentId}"));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}/comments/{commentId}/replies"));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}/comments/{commentId}/replies/{replyId}"));
    assert!(app.contains("/app/v3/api/drive/search"));
    assert!(app.contains("/app/v3/api/drive/changes"));
    assert!(app.contains("/app/v3/api/drive/changes/watch"));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}/watch"));
    assert!(app.contains("/app/v3/api/drive/watch_channels"));
    assert!(app.contains("/app/v3/api/drive/watch_channels/{channelId}"));
    assert!(app.contains("/app/v3/api/drive/watch_channels/{channelId}/stop"));
    assert!(app.contains("/app/v3/api/drive/upload_sessions/{uploadSessionId}"));
    assert!(app.contains("/app/v3/api/drive/upload_sessions/{uploadSessionId}/parts/{partNo}"));
    assert!(app.contains("/app/v3/api/drive/upload_sessions/{uploadSessionId}/complete"));
    assert!(app.contains("/app/v3/api/drive/upload_sessions/{uploadSessionId}/abort"));
    assert!(app.contains("\"operationId\": \"spaces.list\""));
    assert!(app.contains("\"operationId\": \"spaces.create\""));
    assert!(app.contains("\"operationId\": \"spaces.retrieve\""));
    assert!(app.contains("\"operationId\": \"spaces.update\""));
    assert!(app.contains("\"operationId\": \"spaces.delete\""));
    assert!(app.contains("\"operationId\": \"websiteRoots.list\""));
    assert!(app.contains("\"operationId\": \"websiteRoots.create\""));
    assert!(app.contains("\"operationId\": \"websiteRoots.retrieve\""));
    assert!(app.contains("\"operationId\": \"nodes.list\""));
    assert!(app.contains("\"operationId\": \"nodes.folders.create\""));
    assert!(app.contains("\"operationId\": \"nodes.files.create\""));
    assert!(app.contains("\"operationId\": \"nodes.shortcuts.create\""));
    assert!(app.contains("\"operationId\": \"nodes.retrieve\""));
    assert!(app.contains("\"operationId\": \"nodes.path.retrieve\""));
    assert!(app.contains("\"operationId\": \"nodes.capabilities.list\""));
    assert!(app.contains("\"operationId\": \"nodeProperties.list\""));
    assert!(app.contains("\"operationId\": \"nodeProperties.update\""));
    assert!(app.contains("\"operationId\": \"nodeProperties.delete\""));
    assert!(app.contains("\"operationId\": \"nodeLabels.list\""));
    assert!(app.contains("\"operationId\": \"nodeLabels.update\""));
    assert!(app.contains("\"operationId\": \"nodeLabels.delete\""));
    assert!(app.contains("\"operationId\": \"nodes.update\""));
    assert!(app.contains("\"operationId\": \"nodes.move\""));
    assert!(app.contains("\"operationId\": \"nodes.copy\""));
    assert!(app.contains("\"operationId\": \"nodes.delete\""));
    assert!(app.contains("\"operationId\": \"nodes.downloadUrls.retrieve\""));
    assert!(app.contains("\"incompletePage\""));
    assert!(app.contains("\"uri\""));
    assert!(app.contains("\"operationId\": \"trash.create\""));
    assert!(app.contains("\"operationId\": \"trash.restore\""));
    assert!(app.contains("\"operationId\": \"trash.list\""));
    assert!(app.contains("\"operationId\": \"trash.empty\""));
    assert!(app.contains("\"operationId\": \"recent.list\""));
    assert!(app.contains("\"operationId\": \"sharedWithMe.list\""));
    assert!(app.contains("\"operationId\": \"favorites.list\""));
    assert!(app.contains("\"operationId\": \"favorites.update\""));
    assert!(app.contains("\"operationId\": \"favorites.delete\""));
    assert!(app.contains("\"operationId\": \"quotas.retrieve\""));
    assert!(app.contains("\"operationId\": \"uploadSessions.retrieve\""));
    assert!(app.contains("\"operationId\": \"uploadSessions.parts.update\""));
    assert!(app.contains("\"operationId\": \"uploadSessions.complete\""));
    assert!(app.contains("\"operationId\": \"uploadSessions.abort\""));
    assert!(app.contains("\"operationId\": \"versions.list\""));
    assert!(app.contains("\"operationId\": \"versions.retrieve\""));
    assert!(app.contains("\"operationId\": \"versions.delete\""));
    assert!(app.contains("\"operationId\": \"permissions.retrieve\""));
    assert!(app.contains("\"operationId\": \"permissions.create\""));
    assert!(app.contains("\"operationId\": \"permissions.update\""));
    assert!(app.contains("/app/v3/api/drive/nodes/{nodeId}/permissions/effective"));
    assert!(app.contains("\"operationId\": \"permissions.effective.list\""));
    assert!(app.contains("\"EffectivePermission\""));
    assert!(app.contains("\"operationId\": \"shareLinks.retrieve\""));
    assert!(app.contains("\"operationId\": \"shareLinks.create\""));
    assert!(app.contains("\"operationId\": \"shareLinks.list\""));
    assert!(app.contains("\"operationId\": \"shareLinks.update\""));
    assert!(app.contains("\"operationId\": \"comments.list\""));
    assert!(app.contains("\"operationId\": \"comments.create\""));
    assert!(app.contains("\"operationId\": \"comments.retrieve\""));
    assert!(app.contains("\"operationId\": \"comments.update\""));
    assert!(app.contains("\"operationId\": \"comments.delete\""));
    assert!(app.contains("\"operationId\": \"commentReplies.list\""));
    assert!(app.contains("\"operationId\": \"commentReplies.create\""));
    assert!(app.contains("\"operationId\": \"commentReplies.retrieve\""));
    assert!(app.contains("\"operationId\": \"commentReplies.update\""));
    assert!(app.contains("\"operationId\": \"commentReplies.delete\""));
    assert!(app.contains("\"operationId\": \"search.list\""));
    assert!(app.contains("\"operationId\": \"changes.list\""));
    assert!(app.contains("/app/v3/api/drive/changes/start_page_token"));
    assert!(app.contains("\"operationId\": \"changes.startPageToken.retrieve\""));
    assert!(app.contains("\"operationId\": \"changes.watch\""));
    assert!(app.contains("\"operationId\": \"nodes.watch\""));
    assert!(app.contains("\"operationId\": \"watchChannels.list\""));
    assert!(app.contains("\"operationId\": \"watchChannels.retrieve\""));
    assert!(app.contains("\"operationId\": \"watchChannels.stop\""));
    assert!(backend.contains("\"title\": \"SDKWork Drive Backend API\""));
    assert!(
        !backend.contains("/backend/v3/api/drive/storage_providers"),
        "backend OpenAPI must not retain legacy flat storage provider paths"
    );
    assert!(
        !backend.contains("\"operationId\": \"storageProviders."),
        "backend OpenAPI must not retain storageProviders.* operationIds"
    );
    assert!(admin_storage.contains("\"title\": \"SDKWork Drive Admin Storage API\""));
    assert!(admin_storage.contains("/backend/v3/api/drive/storage/providers"));
    assert!(admin_storage.contains("/backend/v3/api/drive/storage/providers/{providerId}"));
    assert!(admin_storage.contains("/backend/v3/api/drive/storage/providers/{providerId}/bucket"));
    assert!(admin_storage.contains("/backend/v3/api/drive/storage/providers/{providerId}/buckets"));
    assert!(admin_storage.contains("/backend/v3/api/drive/storage/providers/{providerId}/objects"));
    assert!(admin_storage
        .contains("/backend/v3/api/drive/storage/providers/{providerId}/objects/{objectKey}"));
    assert!(admin_storage.contains("/backend/v3/api/drive/storage/bindings"));
    assert!(admin_storage.contains("/backend/v3/api/drive/storage/bindings/default"));
    assert!(admin_storage.contains("\"operationId\": \"storageProviders.buckets.list\""));
    assert!(admin_storage.contains("\"operationId\": \"storageProviderBindings.list\""));
    assert!(admin_storage.contains("\"operationId\": \"storageProviderBindings.default.delete\""));
    assert!(app.contains("\"operationId\": \"spaces.list\""));
    assert!(
        !app.contains("\"name\": \"tenantId\""),
        "app OpenAPI must not expose client tenantId query parameters",
    );
    assert!(app.contains("\"201\""));
    assert!(app.contains("/app/v3/api/drive/download_tokens/{token}"));
    assert!(app.contains("/app/v3/api/drive/download_packages"));
    assert!(app.contains("/app/v3/api/drive/download_packages/{packageId}/download_url"));
    assert!(
        !app.contains("/app/v3/api/drive/storage_providers"),
        "app api must not expose storage provider administration routes"
    );
    assert!(
        !app.contains("/app/v3/api/drive/storage_provider_bindings"),
        "app api must not expose storage provider binding administration routes"
    );
    let app_json: Value = serde_json::from_str(&app).expect("app openapi must be valid json");
    for dependency_path in [
        "/app/v3/api/auth/",
        "/app/v3/api/iam/",
        "/app/v3/api/oauth/",
        "/app/v3/api/open_platform/",
        "/app/v3/api/system/iam/",
    ] {
        assert_no_path_starts_with(&app_json, dependency_path);
    }
    assert!(app.contains("\"operationId\": \"downloadUrls.create\""));
    assert!(app.contains("\"operationId\": \"downloadPackages.create\""));
    assert!(app.contains("\"operationId\": \"downloadPackages.downloadUrls.retrieve\""));
    assert!(
        !app.contains("\"operationId\": \"storageProviders."),
        "app api must not expose storage provider administration operation ids"
    );
    assert!(
        !app.contains("\"operationId\": \"storageProviderBindings."),
        "app api must not expose storage provider binding administration operation ids"
    );
    assert!(!app.contains("x-sdkwork-composed-from-owner"));
    assert!(!app.contains("x-sdkwork-composed-from-api-authority"));
    assert!(app.contains("\"signedSourceUrl\""));
    assert!(backend.contains("/backend/v3/api/drive/quotas"));
    assert!(backend.contains("\"operationId\": \"quotas.retrieve\""));
    assert!(backend.contains("\"operationId\": \"quotas.update\""));
    assert!(backend.contains("/backend/v3/api/drive/labels"));
    assert!(backend.contains("/backend/v3/api/drive/labels/{labelId}"));
    assert!(backend.contains("\"operationId\": \"labels.list\""));
    assert!(backend.contains("\"operationId\": \"labels.create\""));
    assert!(backend.contains("\"operationId\": \"labels.retrieve\""));
    assert!(backend.contains("\"operationId\": \"labels.update\""));
    assert!(backend.contains("\"operationId\": \"labels.delete\""));
    assert!(backend.contains("/backend/v3/api/drive/audit_events"));
    assert!(backend.contains("\"operationId\": \"auditEvents.list\""));
    assert!(backend.contains("/backend/v3/api/drive/maintenance/object_sweep"));
    assert!(backend.contains("\"operationId\": \"maintenance.objectSweep\""));
    assert!(backend.contains("/backend/v3/api/drive/maintenance/upload_session_sweep"));
    assert!(backend.contains("\"operationId\": \"maintenance.uploadSessionSweep\""));
    assert!(backend.contains("/backend/v3/api/drive/maintenance/expired_upload_content_sweep"));
    assert!(backend.contains("\"operationId\": \"maintenance.expiredUploadContentSweep\""));
    assert!(backend.contains("/backend/v3/api/drive/maintenance/abandoned_upload_task_sweep"));
    assert!(backend.contains("\"operationId\": \"maintenance.abandonedUploadTaskSweep\""));
    assert!(backend.contains("/backend/v3/api/drive/maintenance/jobs"));
    assert!(backend.contains("\"operationId\": \"maintenance.jobs.list\""));
    assert!(backend.contains("/backend/v3/api/drive/download_packages"));
    assert!(backend.contains("\"operationId\": \"downloadPackages.list\""));

    let open_json: Value = serde_json::from_str(&open).expect("open openapi must be valid json");
    assert_all_paths_start_with(&open_json, "/open/v3/api/drive/");
    assert_schema_property_enum_contains(&app_json, "MediaResource", "source", "drive");
    assert_schema_property_enum_contains(&app_json, "MediaResource", "source", "external_url");
    assert_all_paths_start_with(&app_json, "/app/v3/api/");
    assert_dual_token_security_contract_for_prefix(
        &app_json,
        "app openapi drive routes",
        "/app/v3/api/drive/",
    );
    let backend_json: Value =
        serde_json::from_str(&backend).expect("backend openapi must be valid json");
    assert_all_paths_start_with(&backend_json, "/backend/v3/api/drive/");
    assert_dual_token_security_contract(&backend_json, "backend openapi");
    let admin_storage_json: Value =
        serde_json::from_str(&admin_storage).expect("admin storage openapi must be valid json");
    assert_all_paths_start_with(&admin_storage_json, "/backend/v3/api/drive/storage/");
    assert_dual_token_security_contract(&admin_storage_json, "admin storage openapi");
    assert_eq!(
        admin_storage_json
            .get("x-sdkwork-api-authority")
            .and_then(Value::as_str),
        Some("sdkwork-drive.admin.storage")
    );
    assert_schema_property_enum_contains(
        &admin_storage_json,
        "CreateStorageProviderRequest",
        "providerKind",
        "volcengine_tos",
    );
    assert_public_security_contract(&open_json, "open openapi");
    assert_schema_property_exists(&app_json, "CreateShareLinkRequest", "token");
    assert_schema_property_minimum(&app_json, "CreateShareLinkRequest", "expiresAtEpochMs", 1);
    assert_schema_property_minimum(&app_json, "CreateShareLinkRequest", "downloadLimit", 0);
    assert_schema_property_exists(&app_json, "UpdatePermissionRequest", "role");
    assert_schema_property_exists(&app_json, "UpdateShareLinkRequest", "expiresAtEpochMs");
    assert_schema_property_minimum(&app_json, "UpdateShareLinkRequest", "expiresAtEpochMs", 1);
    assert_schema_property_exists(&app_json, "UpdateShareLinkRequest", "downloadLimit");
    assert_schema_property_minimum(&app_json, "UpdateShareLinkRequest", "downloadLimit", 0);
    assert_schema_property_minimum(&app_json, "DriveShareLink", "expiresAtEpochMs", 1);
    assert_schema_property_minimum(&app_json, "DriveShareLink", "downloadLimit", 0);
    assert_schema_property_minimum(&app_json, "DriveShareLink", "downloadCount", 0);
    for context_field in ["subjectType", "subjectId", "operatorId"] {
        assert_schema_property_absent(&app_json, "FavoriteNodeRequest", context_field);
    }
    assert_schema_property_exists(&app_json, "CreatePermissionRequest", "subjectType");
    assert_schema_property_exists(&app_json, "CreatePermissionRequest", "subjectId");
    assert_schema_required(&app_json, "CreatePermissionRequest", "subjectType");
    assert_schema_required(&app_json, "CreatePermissionRequest", "subjectId");
    assert_schema_property_exists(&app_json, "FavoriteNodeResponse", "favorited");
    assert_schema_property_absent(&app_json, "DriveShareLink", "token");
    assert_schema_property_absent(&app_json, "DriveShareLink", "tokenHash");
    assert_schema_property_exists(&app_json, "CreateShareLinkResponse", "token");
    assert_schema_property_exists(&app_json, "MoveNodeRequest", "targetParentNodeId");
    assert_schema_property_exists(&app_json, "DriveComment", "content");
    assert_schema_property_exists(&app_json, "DriveComment", "resolved");
    assert_schema_property_exists(&app_json, "CreateCommentRequest", "anchor");
    assert_schema_property_exists(&app_json, "UpdateCommentRequest", "resolved");
    assert_schema_property_exists(&app_json, "DriveCommentReply", "commentId");
    assert_schema_property_exists(&app_json, "CreateCommentReplyRequest", "content");
    assert_schema_property_exists(&app_json, "UpdateCommentReplyRequest", "content");
    assert_schema_property_exists(&app_json, "StartPageTokenResponse", "startPageToken");
    assert_schema_property_exists(&app_json, "CreateFileRequest", "uploadSessionId");
    assert_schema_property_deprecated(&app_json, "CreateFileRequest", "objectKey");
    assert_drive_space_type_enum(&app_json, "CreateSpaceRequest", "spaceType");
    assert_drive_space_type_enum(&app_json, "DriveSpace", "spaceType");
    assert_drive_space_type_enum(&app_json, "DriveNode", "spaceType");
    assert_schema_property_exists(&app_json, "CreateWebsiteRootRequest", "rootKey");
    assert_schema_property_exists(&app_json, "CreateWebsiteRootRequest", "sourceRoot");
    assert_schema_property_exists(&app_json, "CreateWebsiteRootRequest", "contentMode");
    assert_schema_property_absent(&app_json, "CreateWebsiteRootRequest", "operatorId");
    assert_schema_required(&app_json, "CreateWebsiteRootRequest", "rootKey");
    assert_schema_required(&app_json, "CreateWebsiteRootRequest", "sourceRoot");
    assert_schema_required(&app_json, "CreateWebsiteRootRequest", "contentMode");
    assert_schema_array_items_ref(
        &app_json,
        "WebsiteRootPageData",
        "items",
        "#/components/schemas/WebsiteRoot",
    );
    assert_schema_property_ref(
        &app_json,
        "WebsiteRootPageData",
        "pageInfo",
        "#/components/schemas/PageInfo",
    );
    assert_envelope_data_schema_ref(
        &app_json,
        "WebsiteRootListHttpResponse",
        "#/components/schemas/WebsiteRootPageData",
    );
    assert_envelope_data_schema_ref(
        &app_json,
        "WebsiteRootHttpResponse",
        "#/components/schemas/WebsiteRootResourceData",
    );
    assert_operation_response_schema_ref(
        &app_json,
        "/app/v3/api/drive/spaces/{spaceId}/website_roots",
        "get",
        "200",
        "#/components/schemas/WebsiteRootListHttpResponse",
    );
    assert_operation_response_schema_ref(
        &app_json,
        "/app/v3/api/drive/spaces/{spaceId}/website_roots",
        "post",
        "200",
        "#/components/schemas/WebsiteRootHttpResponse",
    );
    assert_operation_response_schema_ref(
        &app_json,
        "/app/v3/api/drive/spaces/{spaceId}/website_roots",
        "post",
        "201",
        "#/components/schemas/WebsiteRootHttpResponse",
    );
    assert_operation_response_schema_ref(
        &app_json,
        "/app/v3/api/drive/website_roots/{rootUuid}",
        "get",
        "200",
        "#/components/schemas/WebsiteRootHttpResponse",
    );
    assert_schema_property_deprecated(&app_json, "CreateUploadSessionRequest", "objectKey");
    assert_schema_required_absent(&app_json, "CreateUploadSessionRequest", "objectKey");
    assert_schema_property_exists(&app_json, "CreateFileResponse", "uploadSession");
    assert_schema_property_exists(&app_json, "CreateShortcutRequest", "targetNodeId");
    assert_schema_property_exists(&app_json, "DriveNode", "shortcutTargetNodeId");
    assert_schema_property_exists(&app_json, "DriveNode", "createdAt");
    assert_schema_property_exists(&app_json, "DriveNode", "updatedAt");
    assert_schema_required(&app_json, "DriveNode", "createdAt");
    assert_schema_required(&app_json, "DriveNode", "updatedAt");
    assert_schema_array_items_ref(
        &app_json,
        "DriveNodeListData",
        "items",
        "#/components/schemas/DriveNode",
    );
    assert_schema_property_ref(
        &app_json,
        "DriveNodeListData",
        "pageInfo",
        "#/components/schemas/PageInfo",
    );
    assert_schema_required(&app_json, "DriveNodeListData", "items");
    assert_schema_required(&app_json, "DriveNodeListData", "pageInfo");
    assert_envelope_data_schema_ref(
        &app_json,
        "DriveNodeListHttpResponse",
        "#/components/schemas/DriveNodeListData",
    );
    assert_operation_response_schema_ref(
        &app_json,
        "/app/v3/api/drive/recent",
        "get",
        "200",
        "#/components/schemas/DriveNodeListHttpResponse",
    );
    assert_schema_property_exists(&app_json, "EmptyTrashRequest", "spaceId");
    assert_schema_property_exists(&app_json, "EmptyTrashResponse", "deletedCount");
    assert_schema_property_exists(&app_json, "CopyNodeRequest", "id");
    assert_schema_property_exists(&app_json, "PresignUploadPartRequest", "uploadId");
    assert_schema_property_minimum(
        &app_json,
        "PresignUploadPartRequest",
        "requestedTtlSeconds",
        30,
    );
    assert_schema_property_maximum(
        &app_json,
        "PresignUploadPartRequest",
        "requestedTtlSeconds",
        300,
    );
    assert_schema_property_exists(&app_json, "PresignedUploadPart", "uploadUrl");
    assert_schema_property_minimum(
        &app_json,
        "CreateUploadSessionRequest",
        "expiresAtEpochMs",
        1,
    );
    assert_schema_property_exists(&app_json, "CompleteUploadSessionRequest", "parts");
    assert_schema_property_minimum(
        &app_json,
        "CompleteUploadSessionRequest",
        "contentLength",
        0,
    );
    assert_schema_property_exists(&app_json, "UploadSessionMutationResponse", "state");
    assert_schema_property_enum_contains(
        &app_json,
        "UploadSessionMutationResponse",
        "state",
        "completing",
    );
    assert_schema_property_exists(
        &app_json,
        "UploadSessionMutationResponse",
        "storageUploadId",
    );
    assert_schema_property_exists(
        &app_json,
        "UploadSessionMutationResponse",
        "storageProviderId",
    );
    assert_schema_property_exists(&app_json, "DriveUploadSession", "storageUploadId");
    assert_schema_property_exists(&app_json, "DriveUploadSession", "storageProviderId");
    assert_schema_property_enum_contains(&app_json, "DriveUploadSession", "state", "completing");
    assert_schema_property_exists(&app_json, "NodePathResponse", "items");
    assert_schema_property_exists(&app_json, "NodePathResponse", "pathSegments");
    assert_schema_property_exists(&app_json, "SetNodePropertyRequest", "value");
    assert_schema_property_exists(&app_json, "SetNodePropertyRequest", "visibility");
    assert_schema_property_exists(&app_json, "DriveNodeProperty", "propertyKey");
    assert_schema_property_exists(&app_json, "DriveNodeProperty", "propertyValue");
    assert_schema_property_exists(&app_json, "DriveNodeProperty", "visibility");
    assert_schema_property_exists(&app_json, "DriveLabelSummary", "labelKey");
    assert_schema_property_exists(&app_json, "NodeLabel", "label");
    assert_schema_property_absent(&app_json, "ApplyNodeLabelRequest", "tenantId");
    assert_schema_property_exists(&app_json, "DriveWatchChannel", "resourceType");
    assert_schema_property_exists(&app_json, "DriveWatchChannel", "expirationEpochMs");
    assert_schema_property_exists(&app_json, "CreateWatchChannelRequest", "address");
    assert_schema_property_absent(&app_json, "StopWatchChannelRequest", "tenantId");
    assert_schema_property_exists(&app_json, "StopWatchChannelResponse", "stopped");
    assert_schema_property_exists(&app_json, "CreateDownloadPackageRequest", "nodeIds");
    assert_schema_property_exists(&app_json, "PrepareUploaderUploadRequest", "scene");
    assert_schema_property_exists(&app_json, "PrepareUploaderUploadRequest", "source");
    assert_schema_property_exists(&app_json, "PrepareUploaderUploadRequest", "shareToken");
    assert_schema_property_exists(&app_json, "UploaderUploadItem", "scene");
    assert_schema_property_exists(&app_json, "UploaderUploadItem", "source");
    assert_schema_property_minimum(
        &app_json,
        "CreateDownloadUrlRequest",
        "requestedTtlSeconds",
        30,
    );
    assert_schema_property_maximum(
        &app_json,
        "CreateDownloadUrlRequest",
        "requestedTtlSeconds",
        300,
    );
    assert_query_parameter_minimum(
        &app_json,
        "/app/v3/api/drive/nodes/{nodeId}/download_url",
        "get",
        "requestedTtlSeconds",
        30,
    );
    assert_query_parameter_maximum(
        &app_json,
        "/app/v3/api/drive/nodes/{nodeId}/download_url",
        "get",
        "requestedTtlSeconds",
        300,
    );
    assert_schema_property_minimum(
        &app_json,
        "CreateDownloadPackageRequest",
        "requestedTtlSeconds",
        30,
    );
    assert_schema_property_maximum(
        &app_json,
        "CreateDownloadPackageRequest",
        "requestedTtlSeconds",
        3600,
    );
    assert_schema_property_exists(&app_json, "DownloadPackageResponse", "archiveObjectKey");
    assert_schema_property_exists(&app_json, "DownloadPackageResponse", "signedSourceUrl");
    assert_schema_property_exists(&app_json, "DownloadPackageResponse", "items");
    assert_schema_property_minimum(&app_json, "DownloadPackageResponse", "fileCount", 0);
    assert_schema_property_minimum(&app_json, "DownloadPackageResponse", "totalBytes", 0);
    assert_schema_property_minimum(&app_json, "DownloadPackageResponse", "archiveSizeBytes", 0);
    assert_schema_property_exists(&app_json, "DownloadPackageItem", "archivePath");
    assert_schema_property_absent(&open_json, "DriveOpenShareLink", "token");
    assert_schema_property_absent(&open_json, "DriveOpenShareLink", "tokenHash");
    assert_response_status_exists(
        &open_json,
        "/open/v3/api/drive/share_links/{token}/download_url",
        "post",
        "400",
    );
    assert_response_status_exists(
        &open_json,
        "/open/v3/api/drive/share_links/{token}/download_url",
        "post",
        "409",
    );
    assert_schema_property_minimum(
        &open_json,
        "CreateOpenDownloadUrlRequest",
        "requestedTtlSeconds",
        1,
    );
    assert_schema_property_maximum(
        &open_json,
        "CreateOpenDownloadUrlRequest",
        "requestedTtlSeconds",
        3600,
    );
    let schemas = backend_json
        .get("components")
        .and_then(|value| value.get("schemas"))
        .expect("backend openapi components.schemas should exist");
    let maintenance_job = schemas
        .get("MaintenanceJob")
        .expect("MaintenanceJob schema should exist");
    let properties = maintenance_job
        .get("properties")
        .expect("MaintenanceJob.properties should exist");
    for field_name in ["startedAt", "finishedAt", "createdAt"] {
        let field = properties
            .get(field_name)
            .unwrap_or_else(|| panic!("MaintenanceJob.{} should exist", field_name));
        assert_eq!(
            field.get("format").and_then(Value::as_str),
            Some("date-time"),
            "MaintenanceJob.{} should use date-time format",
            field_name
        );
    }
    assert_schema_property_minimum(&backend_json, "MaintenanceJob", "scannedCount", 0);
    assert_schema_property_minimum(&backend_json, "MaintenanceJob", "affectedCount", 0);
    assert_schema_property_minimum(&backend_json, "DownloadPackage", "fileCount", 0);
    assert_schema_property_minimum(&backend_json, "DownloadPackage", "totalBytes", 0);
    assert_schema_property_minimum(&backend_json, "DownloadPackage", "archiveSizeBytes", 0);
    assert_enum_values(
        properties,
        "jobType",
        &[
            "object_sweep",
            "upload_session_sweep",
            "expired_upload_content_sweep",
            "abandoned_upload_task_sweep",
        ],
    );
    assert_enum_values(properties, "status", &["completed", "failed"]);
    assert_query_parameter_enum(
        &backend_json,
        "/backend/v3/api/drive/maintenance/jobs",
        "get",
        "jobType",
        &[
            "object_sweep",
            "upload_session_sweep",
            "expired_upload_content_sweep",
            "abandoned_upload_task_sweep",
        ],
    );
    assert_query_parameter_enum(
        &backend_json,
        "/backend/v3/api/drive/maintenance/jobs",
        "get",
        "status",
        &["completed", "failed"],
    );
    assert_query_parameter_string_constraints(
        &backend_json,
        "/backend/v3/api/drive/maintenance/jobs",
        "get",
        "operatorId",
        128,
        "^[A-Za-z0-9._:@-]+$",
    );
    assert_query_parameter_string_constraints(
        &backend_json,
        "/backend/v3/api/drive/audit_events",
        "get",
        "action",
        128,
        "^[A-Za-z0-9._:@-]+$",
    );
    assert_schema_property_exists(&backend_json, "DownloadPackage", "archiveObjectKey");
    assert_schema_property_exists(&backend_json, "DownloadPackage", "archiveSizeBytes");
    assert_query_parameter_enum(
        &backend_json,
        "/backend/v3/api/drive/download_packages",
        "get",
        "state",
        &["creating", "ready", "failed", "expired"],
    );

    assert_query_parameter_exists(
        &app_json,
        "/app/v3/api/drive/spaces/{spaceId}/nodes",
        "get",
        "page_size",
    );
    assert_query_parameter_exists(
        &app_json,
        "/app/v3/api/drive/spaces/{spaceId}/nodes",
        "get",
        "cursor",
    );
    assert_query_parameter_exists(&app_json, "/app/v3/api/drive/recent", "get", "page_size");
    assert_query_parameter_exists(
        &app_json,
        "/app/v3/api/drive/nodes/{nodeId}/versions",
        "get",
        "cursor",
    );
    assert_query_parameter_exists(
        &app_json,
        "/app/v3/api/drive/nodes/{nodeId}/comments",
        "get",
        "page_size",
    );
    assert_query_parameter_exists(
        &app_json,
        "/app/v3/api/drive/nodes/{nodeId}/comments",
        "get",
        "cursor",
    );
    assert_query_parameter_exists(
        &app_json,
        "/app/v3/api/drive/nodes/{nodeId}/comments/{commentId}/replies",
        "get",
        "page_size",
    );
    assert_query_parameter_exists(
        &app_json,
        "/app/v3/api/drive/nodes/{nodeId}/comments/{commentId}/replies",
        "get",
        "cursor",
    );
    assert_query_parameter_exists(
        &app_json,
        "/app/v3/api/drive/nodes/{nodeId}/permissions/effective",
        "get",
        "page_size",
    );
    assert_query_parameter_exists(
        &app_json,
        "/app/v3/api/drive/nodes/{nodeId}/permissions/effective",
        "get",
        "cursor",
    );
    assert_query_parameter_exists(
        &app_json,
        "/app/v3/api/drive/nodes/{nodeId}/properties",
        "get",
        "page_size",
    );
    assert_query_parameter_exists(
        &app_json,
        "/app/v3/api/drive/nodes/{nodeId}/properties",
        "get",
        "cursor",
    );
    assert_query_parameter_exists(&app_json, "/app/v3/api/drive/changes", "get", "page_size");
    assert_query_parameter_exists(&app_json, "/app/v3/api/drive/changes", "get", "cursor");
    assert_query_parameter_minimum(
        &app_json,
        "/app/v3/api/drive/spaces/{spaceId}/nodes",
        "get",
        "page_size",
        1,
    );
    assert_query_parameter_maximum(
        &app_json,
        "/app/v3/api/drive/spaces/{spaceId}/nodes",
        "get",
        "page_size",
        200,
    );
    assert_query_parameter_minimum(
        &app_json,
        "/app/v3/api/drive/changes",
        "get",
        "page_size",
        1,
    );
    assert_query_parameter_maximum(
        &app_json,
        "/app/v3/api/drive/changes",
        "get",
        "page_size",
        200,
    );
    assert_query_parameter_minimum(
        &backend_json,
        "/backend/v3/api/drive/download_packages",
        "get",
        "page",
        1,
    );
    assert_query_parameter_maximum(
        &backend_json,
        "/backend/v3/api/drive/download_packages",
        "get",
        "page",
        10000,
    );
    assert_query_parameter_minimum(
        &backend_json,
        "/backend/v3/api/drive/download_packages",
        "get",
        "page_size",
        1,
    );
    assert_query_parameter_maximum(
        &backend_json,
        "/backend/v3/api/drive/download_packages",
        "get",
        "page_size",
        100,
    );
    assert_query_parameter_minimum(
        &backend_json,
        "/backend/v3/api/drive/labels",
        "get",
        "page_size",
        1,
    );
    assert_query_parameter_maximum(
        &backend_json,
        "/backend/v3/api/drive/labels",
        "get",
        "page_size",
        200,
    );
    assert_query_parameter_string_constraints(
        &backend_json,
        "/backend/v3/api/drive/audit_events",
        "get",
        "resourceType",
        64,
        "^[A-Za-z0-9._:@-]+$",
    );
    assert_query_parameter_string_constraints(
        &backend_json,
        "/backend/v3/api/drive/audit_events",
        "get",
        "resourceId",
        128,
        "^[A-Za-z0-9._:@-]+$",
    );
    assert_query_parameter_string_constraints(
        &backend_json,
        "/backend/v3/api/drive/audit_events",
        "get",
        "correlationId",
        64,
        "^[A-Za-z0-9._:@-]+$",
    );
    assert_query_parameter_string_constraints(
        &backend_json,
        "/backend/v3/api/drive/audit_events",
        "get",
        "traceId",
        128,
        "^[A-Za-z0-9._:@-]+$",
    );

    for schema_name in ["SweepObjectStoreRequest", "SweepUploadSessionsRequest"] {
        for context_field in ["operatorId", "requestId", "correlationId", "traceId"] {
            assert_schema_property_absent(&backend_json, schema_name, context_field);
        }
    }

    assert_property_string_constraints(properties, "operatorId", 128, "^[A-Za-z0-9._:@-]+$");
    assert_property_string_constraints(properties, "correlationId", 64, "^[A-Za-z0-9._:@-]+$");
    assert_property_string_constraints(properties, "traceId", 128, "^[A-Za-z0-9._:@-]+$");

    let admin_storage_schemas = admin_storage_json
        .get("components")
        .and_then(|value| value.get("schemas"))
        .expect("admin storage openapi components.schemas should exist");
    let create_storage_provider_properties = admin_storage_schemas
        .get("CreateStorageProviderRequest")
        .and_then(|value| value.get("properties"))
        .expect("CreateStorageProviderRequest.properties should exist");
    let update_storage_provider_properties = admin_storage_schemas
        .get("UpdateStorageProviderRequest")
        .and_then(|value| value.get("properties"))
        .expect("UpdateStorageProviderRequest.properties should exist");
    let storage_provider_properties = admin_storage_schemas
        .get("StorageProvider")
        .and_then(|value| value.get("properties"))
        .expect("StorageProvider.properties should exist");
    assert_drive_space_type_enum(&backend_json, "DriveSpace", "spaceType");
    let provider_kind_enum = [
        "local_filesystem",
        "s3_compatible",
        "google_cloud_storage",
        "aliyun_oss",
        "tencent_cos",
        "huawei_obs",
        "volcengine_tos",
    ];
    let provider_kind_pattern =
        "^(local_filesystem|s3_compatible|google_cloud_storage|aliyun_oss|tencent_cos|huawei_obs|volcengine_tos|custom:[a-z0-9_-]{2,32})$";
    let object_key_pattern = "^(?!/)(?!.*//)(?!.*(?:^|/)\\.{1,2}(?:/|$))(?!.*\\u0000).*(?:[^/])$";
    let object_list_entry_key_pattern =
        "^(?!/)(?!.*//)(?!.*(?:^|/)\\.{1,2}(?:/|$))(?!.*\\u0000).+$";
    assert_enum_values(
        create_storage_provider_properties,
        "providerKind",
        &provider_kind_enum,
    );
    assert_eq!(
        create_storage_provider_properties
            .get("providerKind")
            .and_then(|value| value.get("pattern"))
            .and_then(Value::as_str),
        Some(provider_kind_pattern),
        "CreateStorageProviderRequest.providerKind pattern should match"
    );
    for field_name in [
        "name",
        "region",
        "pathStyle",
        "strictTls",
        "serverSideEncryptionMode",
        "defaultStorageClass",
    ] {
        assert!(
            create_storage_provider_properties.get(field_name).is_some(),
            "CreateStorageProviderRequest.{} should exist",
            field_name
        );
        assert!(
            update_storage_provider_properties.get(field_name).is_some(),
            "UpdateStorageProviderRequest.{} should exist",
            field_name
        );
        assert!(
            storage_provider_properties.get(field_name).is_some(),
            "StorageProvider.{} should exist",
            field_name
        );
    }
    assert!(
        storage_provider_properties
            .get("credentialConfigured")
            .is_some(),
        "StorageProvider.credentialConfigured should exist"
    );
    assert_schema_property_exists(
        &admin_storage_json,
        "StorageProviderCapabilities",
        "supportsMultipartUpload",
    );
    assert_schema_property_exists(&admin_storage_json, "ProviderBucket", "exists");
    assert_schema_property_exists(&admin_storage_json, "ProviderBucketMutation", "changed");
    assert_schema_property_exists(&admin_storage_json, "ProviderObject", "objectKind");
    assert_schema_property_exists(&admin_storage_json, "ProviderObject", "objectKey");
    assert_schema_property_exists(&admin_storage_json, "ProviderObject", "contentLength");
    assert_schema_property_enum_equals(
        &admin_storage_json,
        "ProviderObject",
        "objectKind",
        &["object", "prefix"],
    );
    assert_schema_property_exists(&admin_storage_json, "ProviderObjectMutation", "changed");
    assert_schema_property_exists(
        &admin_storage_json,
        "CopyProviderObjectRequest",
        "destinationObjectKey",
    );
    assert_string_schema_bounds(
        admin_storage_json
            .pointer("/components/schemas/ProviderObject/properties/objectKey")
            .expect("ProviderObject.objectKey should exist"),
        1,
        1024,
        "ProviderObject.objectKey",
    );
    assert_string_schema_pattern(
        admin_storage_json
            .pointer("/components/schemas/ProviderObject/properties/objectKey")
            .expect("ProviderObject.objectKey should exist"),
        object_list_entry_key_pattern,
        "ProviderObject.objectKey",
    );
    assert_string_schema_bounds(
        admin_storage_json
            .pointer("/components/schemas/CopyProviderObjectRequest/properties/sourceObjectKey")
            .expect("CopyProviderObjectRequest.sourceObjectKey should exist"),
        1,
        1024,
        "CopyProviderObjectRequest.sourceObjectKey",
    );
    assert_string_schema_pattern(
        admin_storage_json
            .pointer("/components/schemas/CopyProviderObjectRequest/properties/sourceObjectKey")
            .expect("CopyProviderObjectRequest.sourceObjectKey should exist"),
        object_key_pattern,
        "CopyProviderObjectRequest.sourceObjectKey",
    );
    assert_string_schema_bounds(
        admin_storage_json
            .pointer(
                "/components/schemas/CopyProviderObjectRequest/properties/destinationObjectKey",
            )
            .expect("CopyProviderObjectRequest.destinationObjectKey should exist"),
        1,
        1024,
        "CopyProviderObjectRequest.destinationObjectKey",
    );
    assert_string_schema_pattern(
        admin_storage_json
            .pointer(
                "/components/schemas/CopyProviderObjectRequest/properties/destinationObjectKey",
            )
            .expect("CopyProviderObjectRequest.destinationObjectKey should exist"),
        object_key_pattern,
        "CopyProviderObjectRequest.destinationObjectKey",
    );
    assert_admin_storage_object_key_path_parameters_are_bounded(
        &admin_storage_json,
        object_key_pattern,
    );
    assert_schema_property_exists(
        &admin_storage_json,
        "RotateStorageProviderCredentialRequest",
        "credentialRef",
    );
    assert_schema_property_exists(
        &admin_storage_json,
        "SetDefaultStorageProviderBindingRequest",
        "providerId",
    );
    assert_schema_property_exists(
        &admin_storage_json,
        "StorageProviderBinding",
        "storageProvider",
    );
    assert_schema_property_exists(&admin_storage_json, "ProviderBucketListItem", "configured");
    assert_schema_property_exists(&backend_json, "DriveLabel", "labelKey");
    assert_schema_property_exists(&backend_json, "CreateLabelRequest", "labelKey");
    assert_schema_property_absent(&backend_json, "UpdateLabelRequest", "operatorId");
    assert_enum_values(
        storage_provider_properties,
        "providerKind",
        &provider_kind_enum,
    );
    assert_eq!(
        storage_provider_properties
            .get("providerKind")
            .and_then(|value| value.get("pattern"))
            .and_then(Value::as_str),
        Some(provider_kind_pattern),
        "StorageProvider.providerKind pattern should match"
    );
    for path_key in [
        "/app/v3/api/drive/nodes/{nodeId}",
        "/app/v3/api/drive/nodes/{nodeId}/comments/{commentId}",
        "/app/v3/api/drive/nodes/{nodeId}/comments/{commentId}/replies/{replyId}",
        "/app/v3/api/drive/nodes/{nodeId}/favorite",
        "/app/v3/api/drive/nodes/{nodeId}/labels/{labelId}",
        "/app/v3/api/drive/nodes/{nodeId}/permissions/{permissionId}",
        "/app/v3/api/drive/nodes/{nodeId}/properties/{propertyKey}",
        "/app/v3/api/drive/nodes/{nodeId}/versions/{versionId}",
        "/app/v3/api/drive/share_links/{shareLinkId}",
        "/app/v3/api/drive/spaces/{spaceId}",
        "/app/v3/api/assets/collections/{collectionId}/items/{itemId}",
        "/app/v3/api/assets/{assetId}/relations/{relationId}",
    ] {
        assert_no_content_response(&app_json, path_key, "delete");
    }
    assert_no_content_response(
        &backend_json,
        "/backend/v3/api/drive/labels/{labelId}",
        "delete",
    );
    for path_key in [
        "/backend/v3/api/drive/storage/bindings/default",
        "/backend/v3/api/drive/storage/providers/{providerId}",
        "/backend/v3/api/drive/storage/providers/{providerId}/bucket",
        "/backend/v3/api/drive/storage/providers/{providerId}/objects/{objectKey}",
    ] {
        assert_no_content_response(&admin_storage_json, path_key, "delete");
    }
    for schema_name in [
        "ArchiveEntryListResponse",
        "AssetCollectionPage",
        "AssetPage",
        "ChangeListResponse",
        "CommentListResponse",
        "CommentReplyListResponse",
        "DeleteAssetCollectionItemHttpResponse",
        "DeleteAssetCollectionItemResponse",
        "DeleteAssetRelationHttpResponse",
        "DeleteAssetRelationResponse",
        "DeleteNodePropertyHttpResponse",
        "DeleteNodePropertyResponse",
        "DeleteNodeResponse",
        "DeleteSpaceHttpResponse",
        "DeleteSpaceResponse",
        "DeleteVersionHttpResponse",
        "DeleteVersionResponse",
        "DeletedCommandHttpResponse",
        "DriveWatchChannelListResponse",
        "EffectivePermissionListResponse",
        "ListSpacesResponse",
        "NodeLabelListResponse",
        "NodeListResponse",
        "NodePropertyListResponse",
        "PermissionListResponse",
        "RemoveNodeLabelResponse",
        "RevokeShareLinkResponse",
        "SdkWorkListResponse",
        "ShareLinkListResponse",
        "VersionListResponse",
    ] {
        assert_schema_missing(&app_json, schema_name);
    }
    for schema_name in [
        "AuditEventPage",
        "AuditEventPageSdkWorkEnvelope",
        "DeleteLabelResponse",
        "DownloadPackagePage",
        "DownloadPackagePageSdkWorkEnvelope",
        "LabelListResponse",
        "LabelListResponseSdkWorkEnvelope",
        "ListSpacesResponse",
        "ListSpacesResponseSdkWorkEnvelope",
        "MaintenanceJobPage",
        "MaintenanceJobPageSdkWorkEnvelope",
        "SdkWorkListResponse",
    ] {
        assert_schema_missing(&backend_json, schema_name);
    }
    for schema_name in [
        "DeleteStorageProviderBindingResponse",
        "DeleteStorageProviderResponse",
        "ListStorageProvidersResponse",
        "ProviderBucketList",
        "ProviderObjectList",
        "SdkWorkListResponse",
        "StorageProviderBindingListResponse",
    ] {
        assert_schema_missing(&admin_storage_json, schema_name);
    }
}

#[test]
fn admin_storage_mutation_operations_derive_operator_id_from_request_context() {
    let admin_json: Value = serde_json::from_str(
        &std::fs::read_to_string(
            workspace_root().join("apis/backend-api/drive/drive-admin-storage-api.openapi.json"),
        )
        .expect("admin storage openapi missing"),
    )
    .expect("admin storage openapi should be valid json");

    for (path_key, method) in [
        (
            "/backend/v3/api/drive/storage/providers/{providerId}/bucket",
            "put",
        ),
        (
            "/backend/v3/api/drive/storage/providers/{providerId}/bucket",
            "delete",
        ),
        (
            "/backend/v3/api/drive/storage/providers/{providerId}/objects/{objectKey}",
            "delete",
        ),
        ("/backend/v3/api/drive/storage/bindings/default", "delete"),
    ] {
        assert_query_parameter_absent(&admin_json, path_key, method, "operatorId");
    }

    assert_schema_property_absent(&admin_json, "CopyProviderObjectRequest", "operatorId");
    assert_no_content_response(
        &admin_json,
        "/backend/v3/api/drive/storage/bindings/default",
        "delete",
    );
}

fn assert_string_schema_bounds(schema: &Value, min: i64, max: i64, name: &str) {
    assert_eq!(
        schema.get("type").and_then(Value::as_str),
        Some("string"),
        "{name} should be a string schema"
    );
    assert_eq!(
        schema.get("minLength").and_then(Value::as_i64),
        Some(min),
        "{name} should expose minLength={min}"
    );
    assert_eq!(
        schema.get("maxLength").and_then(Value::as_i64),
        Some(max),
        "{name} should expose maxLength={max}"
    );
}

fn assert_string_schema_pattern(schema: &Value, pattern: &str, name: &str) {
    assert_eq!(
        schema.get("pattern").and_then(Value::as_str),
        Some(pattern),
        "{name} should expose pattern={pattern}"
    );
}

fn assert_admin_storage_object_key_path_parameters_are_bounded(admin_json: &Value, pattern: &str) {
    for method in ["get", "delete"] {
        let parameters = admin_json
            .pointer(&format!(
                "/paths/~1backend~1v3~1api~1drive~1storage~1providers~1{{providerId}}~1objects~1{{objectKey}}/{method}/parameters"
            ))
            .and_then(Value::as_array)
            .expect("admin storage object route parameters should exist");
        let object_key_parameter = parameters
            .iter()
            .find(|value| value.get("name").and_then(Value::as_str) == Some("objectKey"))
            .expect("objectKey path parameter should exist");
        assert_string_schema_bounds(
            object_key_parameter
                .get("schema")
                .expect("objectKey parameter schema should exist"),
            1,
            1024,
            "admin storage objectKey path parameter",
        );
        assert_string_schema_pattern(
            object_key_parameter
                .get("schema")
                .expect("objectKey parameter schema should exist"),
            pattern,
            "admin storage objectKey path parameter",
        );
    }
}

fn assert_all_paths_start_with(openapi: &Value, expected_prefix: &str) {
    let paths = openapi
        .get("paths")
        .and_then(Value::as_object)
        .expect("openapi paths should be an object");
    for path_key in paths.keys() {
        assert!(
            path_key.starts_with(expected_prefix),
            "path {path_key} should start with {expected_prefix}",
        );
    }
}

fn assert_no_path_starts_with(openapi: &Value, forbidden_prefix: &str) {
    let paths = openapi
        .get("paths")
        .and_then(Value::as_object)
        .expect("openapi paths should be an object");
    for path_key in paths.keys() {
        assert!(
            !path_key.starts_with(forbidden_prefix),
            "Drive app authority must consume dependency path {path_key} through sdkDependencies",
        );
    }
}

fn assert_dual_token_security_contract(openapi: &Value, label: &str) {
    assert_dual_token_security_contract_for_prefix(openapi, label, "");
}

fn assert_dual_token_security_contract_for_prefix(openapi: &Value, label: &str, path_prefix: &str) {
    let schemes = openapi
        .get("components")
        .and_then(|value| value.get("securitySchemes"))
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("{label} securitySchemes should exist"));
    assert_eq!(
        schemes
            .get("AuthToken")
            .and_then(|value| value.get("type"))
            .and_then(Value::as_str),
        Some("http"),
        "{label} AuthToken should be an HTTP bearer scheme",
    );
    assert_eq!(
        schemes
            .get("AuthToken")
            .and_then(|value| value.get("scheme"))
            .and_then(Value::as_str),
        Some("bearer"),
        "{label} AuthToken should use bearer transport",
    );
    assert_eq!(
        schemes
            .get("AccessToken")
            .and_then(|value| value.get("type"))
            .and_then(Value::as_str),
        Some("apiKey"),
        "{label} AccessToken should be an apiKey header scheme",
    );
    assert_eq!(
        schemes
            .get("AccessToken")
            .and_then(|value| value.get("in"))
            .and_then(Value::as_str),
        Some("header"),
        "{label} AccessToken should be a header",
    );
    assert_eq!(
        schemes
            .get("AccessToken")
            .and_then(|value| value.get("name"))
            .and_then(Value::as_str),
        Some("Access-Token"),
        "{label} AccessToken header name should be canonical",
    );

    for (path_key, path_item) in openapi
        .get("paths")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("{label} paths should exist"))
    {
        if !path_key.starts_with(path_prefix) {
            continue;
        }
        for (method, operation) in path_item.as_object().expect("path item should be object") {
            if !matches!(
                method.as_str(),
                "get" | "put" | "post" | "delete" | "options" | "head" | "patch" | "trace"
            ) {
                continue;
            }
            let security = operation
                .get("security")
                .and_then(Value::as_array)
                .unwrap_or_else(|| panic!("{label} {method} {path_key} security should exist"));
            assert!(
                security.iter().any(|entry| {
                    entry
                        .get("AuthToken")
                        .and_then(Value::as_array)
                        .is_some_and(Vec::is_empty)
                        && entry
                            .get("AccessToken")
                            .and_then(Value::as_array)
                            .is_some_and(Vec::is_empty)
                }),
                "{label} {method} {path_key} must require AuthToken and AccessToken"
            );
        }
    }
}

fn assert_public_security_contract(openapi: &Value, label: &str) {
    for (path_key, path_item) in openapi
        .get("paths")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("{label} paths should exist"))
    {
        for (method, operation) in path_item.as_object().expect("path item should be object") {
            let security = operation
                .get("security")
                .and_then(Value::as_array)
                .unwrap_or_else(|| panic!("{label} {method} {path_key} security should exist"));
            assert!(
                security.is_empty(),
                "{label} {method} {path_key} must be explicitly public"
            );
        }
    }
}

fn assert_enum_values(properties: &Value, field_name: &str, expected_values: &[&str]) {
    let field = properties
        .get(field_name)
        .unwrap_or_else(|| panic!("MaintenanceJob.{} should exist", field_name));
    let enum_values = field
        .get("enum")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("MaintenanceJob.{} enum should exist", field_name));
    let mut actual = enum_values
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    actual.sort();

    let mut expected = expected_values
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    expected.sort();

    assert_eq!(
        actual, expected,
        "MaintenanceJob.{} enum values should match",
        field_name
    );
}

fn assert_query_parameter_enum(
    openapi: &Value,
    path_key: &str,
    method: &str,
    parameter_name: &str,
    expected_values: &[&str],
) {
    let parameters = openapi
        .get("paths")
        .and_then(|value| value.get(path_key))
        .and_then(|value| value.get(method))
        .and_then(|value| value.get("parameters"))
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("{} {} parameters should exist", method, path_key));
    let parameter = parameters
        .iter()
        .find(|item| {
            item.get("in").and_then(Value::as_str) == Some("query")
                && item.get("name").and_then(Value::as_str) == Some(parameter_name)
        })
        .unwrap_or_else(|| {
            panic!(
                "{} {} query parameter {} should exist",
                method, path_key, parameter_name
            )
        });
    let enum_values = parameter
        .get("schema")
        .and_then(|value| value.get("enum"))
        .and_then(Value::as_array)
        .unwrap_or_else(|| {
            panic!(
                "{} {} query parameter {} enum should exist",
                method, path_key, parameter_name
            )
        });

    let mut actual = enum_values
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    actual.sort();

    let mut expected = expected_values
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    expected.sort();

    assert_eq!(
        actual, expected,
        "{} {} query parameter {} enum values should match",
        method, path_key, parameter_name
    );
}

fn assert_query_parameter_exists(
    openapi: &Value,
    path_key: &str,
    method: &str,
    parameter_name: &str,
) {
    let parameters = openapi
        .get("paths")
        .and_then(|value| value.get(path_key))
        .and_then(|value| value.get(method))
        .and_then(|value| value.get("parameters"))
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("{} {} parameters should exist", method, path_key));
    assert!(
        parameters.iter().any(|item| {
            item.get("in").and_then(Value::as_str) == Some("query")
                && item.get("name").and_then(Value::as_str) == Some(parameter_name)
        }),
        "{} {} query parameter {} should exist",
        method,
        path_key,
        parameter_name
    );
}

fn assert_query_parameter_absent(
    openapi: &Value,
    path_key: &str,
    method: &str,
    parameter_name: &str,
) {
    let parameters = openapi
        .get("paths")
        .and_then(|value| value.get(path_key))
        .and_then(|value| value.get(method))
        .and_then(|value| value.get("parameters"))
        .and_then(Value::as_array);
    let is_exposed = parameters.is_some_and(|items| {
        items.iter().any(|item| {
            item.get("in").and_then(Value::as_str) == Some("query")
                && item.get("name").and_then(Value::as_str) == Some(parameter_name)
        })
    });
    assert!(
        !is_exposed,
        "{} {} query parameter {} must not be client writable",
        method, path_key, parameter_name
    );
}

fn assert_query_parameter_minimum(
    openapi: &Value,
    path_key: &str,
    method: &str,
    parameter_name: &str,
    minimum: i64,
) {
    let schema = query_parameter_schema(openapi, path_key, method, parameter_name);
    assert_eq!(
        schema.get("minimum").and_then(Value::as_i64),
        Some(minimum),
        "{} {} query parameter {} minimum should be {}",
        method,
        path_key,
        parameter_name,
        minimum
    );
}

fn assert_query_parameter_maximum(
    openapi: &Value,
    path_key: &str,
    method: &str,
    parameter_name: &str,
    maximum: i64,
) {
    let schema = query_parameter_schema(openapi, path_key, method, parameter_name);
    assert_eq!(
        schema.get("maximum").and_then(Value::as_i64),
        Some(maximum),
        "{} {} query parameter {} maximum should be {}",
        method,
        path_key,
        parameter_name,
        maximum
    );
}

fn query_parameter_schema<'a>(
    openapi: &'a Value,
    path_key: &str,
    method: &str,
    parameter_name: &str,
) -> &'a Value {
    let parameters = openapi
        .get("paths")
        .and_then(|value| value.get(path_key))
        .and_then(|value| value.get(method))
        .and_then(|value| value.get("parameters"))
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("{} {} parameters should exist", method, path_key));
    parameters
        .iter()
        .find(|item| {
            item.get("in").and_then(Value::as_str) == Some("query")
                && item.get("name").and_then(Value::as_str) == Some(parameter_name)
        })
        .and_then(|item| item.get("schema"))
        .unwrap_or_else(|| {
            panic!(
                "{} {} query parameter {} schema should exist",
                method, path_key, parameter_name
            )
        })
}

fn assert_property_string_constraints(
    properties: &Value,
    field_name: &str,
    max_length: u64,
    pattern: &str,
) {
    let field = properties
        .get(field_name)
        .unwrap_or_else(|| panic!("{}.{} should exist", "properties", field_name));
    assert_eq!(
        field.get("maxLength").and_then(Value::as_u64),
        Some(max_length),
        "{} maxLength should be {}",
        field_name,
        max_length
    );
    assert_eq!(
        field.get("pattern").and_then(Value::as_str),
        Some(pattern),
        "{} pattern should be {}",
        field_name,
        pattern
    );
}

fn assert_schema_property_exists(openapi: &Value, schema_name: &str, field_name: &str) {
    let properties = schema_properties(openapi, schema_name);
    assert!(
        properties.get(field_name).is_some(),
        "{}.{} should exist",
        schema_name,
        field_name
    );
}

fn assert_schema_property_ref(
    openapi: &Value,
    schema_name: &str,
    field_name: &str,
    expected_ref: &str,
) {
    let field = schema_properties(openapi, schema_name)
        .get(field_name)
        .unwrap_or_else(|| panic!("{schema_name}.{field_name} should exist"));
    assert_eq!(
        field.get("$ref").and_then(Value::as_str),
        Some(expected_ref),
        "{schema_name}.{field_name} should reference {expected_ref}"
    );
}

fn assert_schema_array_items_ref(
    openapi: &Value,
    schema_name: &str,
    field_name: &str,
    expected_ref: &str,
) {
    let field = schema_properties(openapi, schema_name)
        .get(field_name)
        .unwrap_or_else(|| panic!("{schema_name}.{field_name} should exist"));
    assert_eq!(
        field
            .get("items")
            .and_then(|value| value.get("$ref"))
            .and_then(Value::as_str),
        Some(expected_ref),
        "{schema_name}.{field_name} items should reference {expected_ref}"
    );
}

fn assert_envelope_data_schema_ref(openapi: &Value, schema_name: &str, expected_ref: &str) {
    let schema = openapi
        .get("components")
        .and_then(|value| value.get("schemas"))
        .and_then(|value| value.get(schema_name))
        .unwrap_or_else(|| panic!("{schema_name} should exist"));
    let data_ref = schema
        .get("allOf")
        .and_then(Value::as_array)
        .and_then(|parts| {
            parts.iter().find_map(|part| {
                part.get("properties")
                    .and_then(|value| value.get("data"))
                    .and_then(|value| value.get("$ref"))
                    .and_then(Value::as_str)
            })
        });
    assert_eq!(
        data_ref,
        Some(expected_ref),
        "{schema_name}.data should reference {expected_ref}"
    );
}

fn assert_schema_property_absent(openapi: &Value, schema_name: &str, field_name: &str) {
    let properties = schema_properties(openapi, schema_name);
    assert!(
        properties.get(field_name).is_none(),
        "{}.{} must not be exposed",
        schema_name,
        field_name
    );
}

fn assert_schema_property_deprecated(openapi: &Value, schema_name: &str, field_name: &str) {
    let properties = schema_properties(openapi, schema_name);
    let field = properties
        .get(field_name)
        .unwrap_or_else(|| panic!("{}.{} should exist", schema_name, field_name));
    assert_eq!(
        field.get("deprecated").and_then(Value::as_bool),
        Some(true),
        "{}.{} should be marked deprecated",
        schema_name,
        field_name
    );
}

fn assert_schema_property_minimum(
    openapi: &Value,
    schema_name: &str,
    field_name: &str,
    minimum: i64,
) {
    let properties = schema_properties(openapi, schema_name);
    let field = properties
        .get(field_name)
        .unwrap_or_else(|| panic!("{}.{} should exist", schema_name, field_name));
    assert_eq!(
        field.get("minimum").and_then(Value::as_i64),
        Some(minimum),
        "{}.{} minimum should be {}",
        schema_name,
        field_name,
        minimum
    );
}

fn assert_schema_property_maximum(
    openapi: &Value,
    schema_name: &str,
    field_name: &str,
    maximum: i64,
) {
    let properties = schema_properties(openapi, schema_name);
    let field = properties
        .get(field_name)
        .unwrap_or_else(|| panic!("{}.{} should exist", schema_name, field_name));
    assert_eq!(
        field.get("maximum").and_then(Value::as_i64),
        Some(maximum),
        "{}.{} maximum should be {}",
        schema_name,
        field_name,
        maximum
    );
}

fn assert_response_status_exists(openapi: &Value, path_key: &str, method: &str, status: &str) {
    let responses = openapi
        .get("paths")
        .and_then(|value| value.get(path_key))
        .and_then(|value| value.get(method))
        .and_then(|value| value.get("responses"))
        .unwrap_or_else(|| panic!("{method} {path_key} responses should exist"));
    assert!(
        responses.get(status).is_some(),
        "{method} {path_key} response {status} should exist"
    );
}

fn assert_operation_response_schema_ref(
    openapi: &Value,
    path_key: &str,
    method: &str,
    status: &str,
    expected_ref: &str,
) {
    let schema = openapi
        .get("paths")
        .and_then(|value| value.get(path_key))
        .and_then(|value| value.get(method))
        .and_then(|value| value.get("responses"))
        .and_then(|value| value.get(status))
        .and_then(|value| value.get("content"))
        .and_then(|value| value.get("application/json"))
        .and_then(|value| value.get("schema"))
        .unwrap_or_else(|| panic!("{method} {path_key} response {status} schema should exist"));
    assert_eq!(
        schema.get("$ref").and_then(Value::as_str),
        Some(expected_ref),
        "{method} {path_key} response {status} should reference {expected_ref} directly"
    );
    assert!(
        schema.get("allOf").is_none(),
        "{method} {path_key} response {status} must not duplicate-wrap an enveloped schema"
    );
}

fn assert_no_content_response(openapi: &Value, path_key: &str, method: &str) {
    let responses = openapi
        .get("paths")
        .and_then(|value| value.get(path_key))
        .and_then(|value| value.get(method))
        .and_then(|value| value.get("responses"))
        .unwrap_or_else(|| panic!("{method} {path_key} responses should exist"));
    let no_content = responses
        .get("204")
        .unwrap_or_else(|| panic!("{method} {path_key} response 204 should exist"));
    assert!(
        no_content.get("content").is_none(),
        "{method} {path_key} 204 response must not declare content"
    );
    assert!(
        responses
            .get("200")
            .and_then(|value| value.get("content"))
            .is_none(),
        "{method} {path_key} must not keep legacy 200 JSON delete content"
    );
}

fn assert_schema_missing(openapi: &Value, schema_name: &str) {
    let schemas = openapi
        .get("components")
        .and_then(|value| value.get("schemas"))
        .unwrap_or_else(|| panic!("components.schemas should exist"));
    assert!(
        schemas.get(schema_name).is_none(),
        "{schema_name} must not be exposed"
    );
}

fn assert_schema_property_enum_contains(
    openapi: &Value,
    schema_name: &str,
    field_name: &str,
    expected_value: &str,
) {
    let properties = schema_properties(openapi, schema_name);
    let field = properties
        .get(field_name)
        .unwrap_or_else(|| panic!("{schema_name}.{field_name} should exist"));
    let enum_values = field
        .get("enum")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("{schema_name}.{field_name} should define enum values"));
    assert!(
        enum_values
            .iter()
            .any(|value| value.as_str() == Some(expected_value)),
        "{schema_name}.{field_name} enum should contain {expected_value}"
    );
}

fn assert_schema_property_enum_equals(
    openapi: &Value,
    schema_name: &str,
    field_name: &str,
    expected_values: &[&str],
) {
    let properties = schema_properties(openapi, schema_name);
    let field = properties
        .get(field_name)
        .unwrap_or_else(|| panic!("{schema_name}.{field_name} should exist"));
    let enum_values = field
        .get("enum")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("{schema_name}.{field_name} should define enum values"));
    let mut actual = enum_values
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    actual.sort();

    let mut expected = expected_values
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    expected.sort();

    assert_eq!(
        actual, expected,
        "{schema_name}.{field_name} enum values should match"
    );
}

fn assert_drive_space_type_enum(openapi: &Value, schema_name: &str, field_name: &str) {
    assert_schema_property_enum_equals(
        openapi,
        schema_name,
        field_name,
        &[
            "personal",
            "team",
            "knowledge_base",
            "ai_generated",
            "git_repository",
            "deployment",
            "app_upload",
            "im",
            "rtc",
            "notary",
            "website",
        ],
    );
}

fn assert_schema_required_absent(openapi: &Value, schema_name: &str, field_name: &str) {
    let required = openapi
        .get("components")
        .and_then(|value| value.get("schemas"))
        .and_then(|value| value.get(schema_name))
        .and_then(|value| value.get("required"))
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("{}.required should exist", schema_name));
    assert!(
        !required
            .iter()
            .any(|value| value.as_str() == Some(field_name)),
        "{}.{} must not be required",
        schema_name,
        field_name
    );
}

fn assert_schema_required(openapi: &Value, schema_name: &str, field_name: &str) {
    let required = openapi
        .get("components")
        .and_then(|value| value.get("schemas"))
        .and_then(|value| value.get(schema_name))
        .and_then(|value| value.get("required"))
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("{}.required should exist", schema_name));
    assert!(
        required
            .iter()
            .any(|value| value.as_str() == Some(field_name)),
        "{}.{} must be required",
        schema_name,
        field_name
    );
}

fn schema_properties<'a>(openapi: &'a Value, schema_name: &str) -> &'a Value {
    openapi
        .get("components")
        .and_then(|value| value.get("schemas"))
        .and_then(|value| value.get(schema_name))
        .and_then(|value| value.get("properties"))
        .unwrap_or_else(|| panic!("{}.properties should exist", schema_name))
}

fn assert_query_parameter_string_constraints(
    openapi: &Value,
    path_key: &str,
    method: &str,
    parameter_name: &str,
    max_length: u64,
    pattern: &str,
) {
    let parameters = openapi
        .get("paths")
        .and_then(|value| value.get(path_key))
        .and_then(|value| value.get(method))
        .and_then(|value| value.get("parameters"))
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("{} {} parameters should exist", method, path_key));
    let parameter = parameters
        .iter()
        .find(|item| {
            item.get("in").and_then(Value::as_str) == Some("query")
                && item.get("name").and_then(Value::as_str) == Some(parameter_name)
        })
        .unwrap_or_else(|| {
            panic!(
                "{} {} query parameter {} should exist",
                method, path_key, parameter_name
            )
        });
    let schema = parameter.get("schema").unwrap_or_else(|| {
        panic!(
            "{} {} query parameter {} schema should exist",
            method, path_key, parameter_name
        )
    });
    assert_eq!(
        schema.get("maxLength").and_then(Value::as_u64),
        Some(max_length),
        "{} {} query parameter {} maxLength should be {}",
        method,
        path_key,
        parameter_name,
        max_length
    );
    assert_eq!(
        schema.get("pattern").and_then(Value::as_str),
        Some(pattern),
        "{} {} query parameter {} pattern should be {}",
        method,
        path_key,
        parameter_name,
        pattern
    );
}
