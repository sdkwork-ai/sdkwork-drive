package com.sdkwork.generated;

import java.util.LinkedHashMap;
import java.util.Map;

public final class SdkMetadata {
  public static final String SDK_NAME = "sdkwork-drive-app-sdk";
  public static final String PACKAGE_NAME = "sdkwork-drive-app-sdk-generated-java";
  public static final String STANDARD_PROFILE = "sdkwork-v3";
  public static final String BASE_URL = "http://127.0.0.1:18080";
  public static final String API_PREFIX = "/app/v3/api";

  public static Map<String, String> operations() {
    Map<String, String> operations = new LinkedHashMap<>();
    operations.put("changes.list", "GET /app/v3/api/drive/changes");
    operations.put("changes.startPageToken.get", "GET /app/v3/api/drive/changes/start_page_token");
    operations.put("changes.watch", "POST /app/v3/api/drive/changes/watch");
    operations.put("commentReplies.create", "POST /app/v3/api/drive/nodes/{nodeId}/comments/{commentId}/replies");
    operations.put("commentReplies.delete", "DELETE /app/v3/api/drive/nodes/{nodeId}/comments/{commentId}/replies/{replyId}");
    operations.put("commentReplies.get", "GET /app/v3/api/drive/nodes/{nodeId}/comments/{commentId}/replies/{replyId}");
    operations.put("commentReplies.list", "GET /app/v3/api/drive/nodes/{nodeId}/comments/{commentId}/replies");
    operations.put("commentReplies.update", "PATCH /app/v3/api/drive/nodes/{nodeId}/comments/{commentId}/replies/{replyId}");
    operations.put("comments.create", "POST /app/v3/api/drive/nodes/{nodeId}/comments");
    operations.put("comments.delete", "DELETE /app/v3/api/drive/nodes/{nodeId}/comments/{commentId}");
    operations.put("comments.get", "GET /app/v3/api/drive/nodes/{nodeId}/comments/{commentId}");
    operations.put("comments.list", "GET /app/v3/api/drive/nodes/{nodeId}/comments");
    operations.put("comments.update", "PATCH /app/v3/api/drive/nodes/{nodeId}/comments/{commentId}");
    operations.put("downloadPackages.create", "POST /app/v3/api/drive/download_packages");
    operations.put("downloadPackages.downloadUrls.get", "GET /app/v3/api/drive/download_packages/{packageId}/download_url");
    operations.put("downloadTokens.resolve", "GET /app/v3/api/drive/download_tokens/{token}");
    operations.put("downloadUrls.create", "POST /app/v3/api/drive/download_urls");
    operations.put("favorites.delete", "DELETE /app/v3/api/drive/nodes/{nodeId}/favorite");
    operations.put("favorites.list", "GET /app/v3/api/drive/favorites");
    operations.put("favorites.set", "PUT /app/v3/api/drive/nodes/{nodeId}/favorite");
    operations.put("nodeLabels.apply", "PUT /app/v3/api/drive/nodes/{nodeId}/labels/{labelId}");
    operations.put("nodeLabels.list", "GET /app/v3/api/drive/nodes/{nodeId}/labels");
    operations.put("nodeLabels.remove", "DELETE /app/v3/api/drive/nodes/{nodeId}/labels/{labelId}");
    operations.put("nodeProperties.delete", "DELETE /app/v3/api/drive/nodes/{nodeId}/properties/{propertyKey}");
    operations.put("nodeProperties.list", "GET /app/v3/api/drive/nodes/{nodeId}/properties");
    operations.put("nodeProperties.set", "PUT /app/v3/api/drive/nodes/{nodeId}/properties/{propertyKey}");
    operations.put("nodes.capabilities.get", "GET /app/v3/api/drive/nodes/{nodeId}/capabilities");
    operations.put("nodes.copy", "POST /app/v3/api/drive/nodes/{nodeId}/copy");
    operations.put("nodes.delete", "DELETE /app/v3/api/drive/nodes/{nodeId}");
    operations.put("nodes.downloadUrls.create", "GET /app/v3/api/drive/nodes/{nodeId}/download_url");
    operations.put("nodes.files.create", "POST /app/v3/api/drive/nodes/files");
    operations.put("nodes.folders.create", "POST /app/v3/api/drive/nodes/folders");
    operations.put("nodes.get", "GET /app/v3/api/drive/nodes/{nodeId}");
    operations.put("nodes.list", "GET /app/v3/api/drive/spaces/{spaceId}/nodes");
    operations.put("nodes.move", "POST /app/v3/api/drive/nodes/{nodeId}/move");
    operations.put("nodes.path.get", "GET /app/v3/api/drive/nodes/{nodeId}/path");
    operations.put("nodes.shortcuts.create", "POST /app/v3/api/drive/nodes/shortcuts");
    operations.put("nodes.update", "PATCH /app/v3/api/drive/nodes/{nodeId}");
    operations.put("nodes.watch", "POST /app/v3/api/drive/nodes/{nodeId}/watch");
    operations.put("permissions.create", "POST /app/v3/api/drive/nodes/{nodeId}/permissions");
    operations.put("permissions.delete", "DELETE /app/v3/api/drive/nodes/{nodeId}/permissions/{permissionId}");
    operations.put("permissions.effective.list", "GET /app/v3/api/drive/nodes/{nodeId}/permissions/effective");
    operations.put("permissions.get", "GET /app/v3/api/drive/nodes/{nodeId}/permissions/{permissionId}");
    operations.put("permissions.list", "GET /app/v3/api/drive/nodes/{nodeId}/permissions");
    operations.put("permissions.update", "PATCH /app/v3/api/drive/nodes/{nodeId}/permissions/{permissionId}");
    operations.put("recent.list", "GET /app/v3/api/drive/recent");
    operations.put("search.query", "GET /app/v3/api/drive/search");
    operations.put("sharedWithMe.list", "GET /app/v3/api/drive/shared_with_me");
    operations.put("shareLinks.create", "POST /app/v3/api/drive/nodes/{nodeId}/share_links");
    operations.put("shareLinks.claim", "POST /app/v3/api/drive/share_links/{token}/claim");
    operations.put("shareLinks.get", "GET /app/v3/api/drive/share_links/{shareLinkId}");
    operations.put("shareLinks.list", "GET /app/v3/api/drive/nodes/{nodeId}/share_links");
    operations.put("shareLinks.revoke", "DELETE /app/v3/api/drive/share_links/{shareLinkId}");
    operations.put("shareLinks.update", "PATCH /app/v3/api/drive/share_links/{shareLinkId}");
    operations.put("spaces.create", "POST /app/v3/api/drive/spaces");
    operations.put("spaces.delete", "DELETE /app/v3/api/drive/spaces/{spaceId}");
    operations.put("spaces.get", "GET /app/v3/api/drive/spaces/{spaceId}");
    operations.put("spaces.list", "GET /app/v3/api/drive/spaces");
    operations.put("spaces.update", "PATCH /app/v3/api/drive/spaces/{spaceId}");
    operations.put("trash.empty", "POST /app/v3/api/drive/trash/empty");
    operations.put("trash.list", "GET /app/v3/api/drive/trash");
    operations.put("trash.move", "POST /app/v3/api/drive/nodes/{nodeId}/trash");
    operations.put("trash.restore", "POST /app/v3/api/drive/trash/{nodeId}/restore");
    operations.put("uploadSessions.abort", "POST /app/v3/api/drive/upload_sessions/{uploadSessionId}/abort");
    operations.put("uploadSessions.complete", "POST /app/v3/api/drive/upload_sessions/{uploadSessionId}/complete");
    operations.put("uploadSessions.create", "POST /app/v3/api/drive/upload_sessions");
    operations.put("uploadSessions.get", "GET /app/v3/api/drive/upload_sessions/{uploadSessionId}");
    operations.put("uploadSessions.parts.presign", "PUT /app/v3/api/drive/upload_sessions/{uploadSessionId}/parts/{partNo}");
    operations.put("versions.delete", "DELETE /app/v3/api/drive/nodes/{nodeId}/versions/{versionId}");
    operations.put("versions.get", "GET /app/v3/api/drive/nodes/{nodeId}/versions/{versionId}");
    operations.put("versions.list", "GET /app/v3/api/drive/nodes/{nodeId}/versions");
    operations.put("versions.restore", "POST /app/v3/api/drive/nodes/{nodeId}/versions/{versionId}/restore");
    operations.put("watchChannels.get", "GET /app/v3/api/drive/watch_channels/{channelId}");
    operations.put("watchChannels.list", "GET /app/v3/api/drive/watch_channels");
    operations.put("watchChannels.stop", "POST /app/v3/api/drive/watch_channels/{channelId}/stop");
    return operations;
  }

  private SdkMetadata() {}
}
