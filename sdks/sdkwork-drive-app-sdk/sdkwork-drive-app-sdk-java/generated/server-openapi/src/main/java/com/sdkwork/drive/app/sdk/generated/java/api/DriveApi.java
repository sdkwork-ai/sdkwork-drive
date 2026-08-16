package com.sdkwork.drive.app.sdk.generated.java.api;

import com.fasterxml.jackson.core.type.TypeReference;
import com.sdkwork.drive.app.sdk.generated.java.http.HttpClient;
import com.sdkwork.drive.app.sdk.generated.java.model.*;
import java.util.List;
import java.util.Map;

public class DriveApi {
    private final HttpClient client;

    public DriveApi(HttpClient client) {
        this.client = client;
    }

    public ChangeListHttpResponse changesList(String spaceId, Integer cursor, Integer pageSize) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("spaceId", spaceId, "form", true, false, null),
            new QueryParameterSpec("cursor", cursor, "form", true, false, null),
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/changes"), query));
        return client.convertValue(raw, new TypeReference<ChangeListHttpResponse>() {});
    }

    public StartPageTokenHttpResponse changesStartPageTokenRetrieve(String spaceId) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("spaceId", spaceId, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/changes/start_page_token"), query));
        return client.convertValue(raw, new TypeReference<StartPageTokenHttpResponse>() {});
    }

    public CreateDownloadUrlHttpResponse downloadTokensRetrieve(String token) throws Exception {
        Object raw = client.get(ApiPaths.appPath("/drive/download_tokens/" + serializePathParameter(token, new PathParameterSpec("token", "simple", false)) + ""));
        return client.convertValue(raw, new TypeReference<CreateDownloadUrlHttpResponse>() {});
    }

    public CreateDownloadUrlHttpResponse downloadUrlsCreate(CreateDownloadUrlRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/download_urls"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<CreateDownloadUrlHttpResponse>() {});
    }

    public DriveNodeListHttpResponse favoritesList(String spaceId, Integer pageSize, String cursor, String sortBy, String sortOrder) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("spaceId", spaceId, "form", true, false, null),
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null),
            new QueryParameterSpec("cursor", cursor, "form", true, false, null),
            new QueryParameterSpec("sortBy", sortBy, "form", true, false, null),
            new QueryParameterSpec("sortOrder", sortOrder, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/favorites"), query));
        return client.convertValue(raw, new TypeReference<DriveNodeListHttpResponse>() {});
    }

    public SdkWorkApiResponse favoritesCheck(CheckFavoriteNodesRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/favorites/check"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<SdkWorkApiResponse>() {});
    }

    public QuotaSummaryHttpResponse quotasRetrieve() throws Exception {
        Object raw = client.get(ApiPaths.appPath("/drive/quotas/summary"));
        return client.convertValue(raw, new TypeReference<QuotaSummaryHttpResponse>() {});
    }

    public DriveNodeHttpResponse nodesUpdate(String nodeId, UpdateNodeRequest body) throws Exception {
        Object raw = client.patch(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + ""), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<DriveNodeHttpResponse>() {});
    }

    public DriveNodeHttpResponse nodesRetrieve(String nodeId) throws Exception {
        Object raw = client.get(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + ""));
        return client.convertValue(raw, new TypeReference<DriveNodeHttpResponse>() {});
    }

    public Void nodesDelete(String nodeId) throws Exception {
        client.delete(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + ""));
        return null;
    }

    public NodeCapabilitiesHttpResponse nodesCapabilitiesList(String nodeId) throws Exception {
        Object raw = client.get(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/capabilities"));
        return client.convertValue(raw, new TypeReference<NodeCapabilitiesHttpResponse>() {});
    }

    public DriveCommentListHttpResponse commentsList(String nodeId, Integer pageSize, String cursor) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null),
            new QueryParameterSpec("cursor", cursor, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/comments"), query));
        return client.convertValue(raw, new TypeReference<DriveCommentListHttpResponse>() {});
    }

    public DriveCommentHttpResponse commentsCreate(String nodeId, CreateCommentRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/comments"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<DriveCommentHttpResponse>() {});
    }

    public DriveCommentHttpResponse commentsRetrieve(String nodeId, String commentId) throws Exception {
        Object raw = client.get(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/comments/" + serializePathParameter(commentId, new PathParameterSpec("commentId", "simple", false)) + ""));
        return client.convertValue(raw, new TypeReference<DriveCommentHttpResponse>() {});
    }

    public DriveCommentHttpResponse commentsUpdate(String nodeId, String commentId, UpdateCommentRequest body) throws Exception {
        Object raw = client.patch(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/comments/" + serializePathParameter(commentId, new PathParameterSpec("commentId", "simple", false)) + ""), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<DriveCommentHttpResponse>() {});
    }

    public Void commentsDelete(String nodeId, String commentId) throws Exception {
        client.delete(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/comments/" + serializePathParameter(commentId, new PathParameterSpec("commentId", "simple", false)) + ""));
        return null;
    }

    public DriveCommentReplyListHttpResponse commentRepliesList(String nodeId, String commentId, Integer pageSize, String cursor) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null),
            new QueryParameterSpec("cursor", cursor, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/comments/" + serializePathParameter(commentId, new PathParameterSpec("commentId", "simple", false)) + "/replies"), query));
        return client.convertValue(raw, new TypeReference<DriveCommentReplyListHttpResponse>() {});
    }

    public DriveCommentReplyHttpResponse commentRepliesCreate(String nodeId, String commentId, CreateCommentReplyRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/comments/" + serializePathParameter(commentId, new PathParameterSpec("commentId", "simple", false)) + "/replies"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<DriveCommentReplyHttpResponse>() {});
    }

    public DriveCommentReplyHttpResponse commentRepliesRetrieve(String nodeId, String commentId, String replyId) throws Exception {
        Object raw = client.get(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/comments/" + serializePathParameter(commentId, new PathParameterSpec("commentId", "simple", false)) + "/replies/" + serializePathParameter(replyId, new PathParameterSpec("replyId", "simple", false)) + ""));
        return client.convertValue(raw, new TypeReference<DriveCommentReplyHttpResponse>() {});
    }

    public DriveCommentReplyHttpResponse commentRepliesUpdate(String nodeId, String commentId, String replyId, UpdateCommentReplyRequest body) throws Exception {
        Object raw = client.patch(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/comments/" + serializePathParameter(commentId, new PathParameterSpec("commentId", "simple", false)) + "/replies/" + serializePathParameter(replyId, new PathParameterSpec("replyId", "simple", false)) + ""), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<DriveCommentReplyHttpResponse>() {});
    }

    public Void commentRepliesDelete(String nodeId, String commentId, String replyId) throws Exception {
        client.delete(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/comments/" + serializePathParameter(commentId, new PathParameterSpec("commentId", "simple", false)) + "/replies/" + serializePathParameter(replyId, new PathParameterSpec("replyId", "simple", false)) + ""));
        return null;
    }

    public DriveNodeHttpResponse nodesCopy(String nodeId, CopyNodeRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/copy"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<DriveNodeHttpResponse>() {});
    }

    public CreateDownloadUrlHttpResponse nodesDownloadUrlsRetrieve(String nodeId, Integer requestedTtlSeconds) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("requestedTtlSeconds", requestedTtlSeconds, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/download_url"), query));
        return client.convertValue(raw, new TypeReference<CreateDownloadUrlHttpResponse>() {});
    }

    public CreateDownloadUrlHttpResponse downloadGrantsCreate(String nodeId, CreateDownloadGrantRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/download_grants"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<CreateDownloadUrlHttpResponse>() {});
    }

    public FavoriteNodeHttpResponse favoritesUpdate(String nodeId, FavoriteNodeRequest body) throws Exception {
        Object raw = client.put(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/favorite"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<FavoriteNodeHttpResponse>() {});
    }

    public Void favoritesDelete(String nodeId) throws Exception {
        client.delete(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/favorite"));
        return null;
    }

    /** List labels applied to a node */
    public NodeLabelListHttpResponse nodeLabelsList(String nodeId, String labelKey, Integer pageSize, String cursor) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("labelKey", labelKey, "form", true, false, null),
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null),
            new QueryParameterSpec("cursor", cursor, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/labels"), query));
        return client.convertValue(raw, new TypeReference<NodeLabelListHttpResponse>() {});
    }

    /** Apply a label to a node */
    public NodeLabelHttpResponse nodeLabelsUpdate(String nodeId, String labelId, ApplyNodeLabelRequest body) throws Exception {
        Object raw = client.put(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/labels/" + serializePathParameter(labelId, new PathParameterSpec("labelId", "simple", false)) + ""), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<NodeLabelHttpResponse>() {});
    }

    /** Remove a label from a node */
    public Void nodeLabelsDelete(String nodeId, String labelId) throws Exception {
        client.delete(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/labels/" + serializePathParameter(labelId, new PathParameterSpec("labelId", "simple", false)) + ""));
        return null;
    }

    public DriveNodeHttpResponse nodesMove(String nodeId, MoveNodeRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/move"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<DriveNodeHttpResponse>() {});
    }

    public NodePathHttpResponse nodesPathRetrieve(String nodeId) throws Exception {
        Object raw = client.get(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/path"));
        return client.convertValue(raw, new TypeReference<NodePathHttpResponse>() {});
    }

    public DrivePermissionListHttpResponse permissionsList(String nodeId, Integer pageSize, String cursor) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null),
            new QueryParameterSpec("cursor", cursor, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/permissions"), query));
        return client.convertValue(raw, new TypeReference<DrivePermissionListHttpResponse>() {});
    }

    public DrivePermissionHttpResponse permissionsCreate(String nodeId, CreatePermissionRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/permissions"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<DrivePermissionHttpResponse>() {});
    }

    public Void permissionsDelete(String nodeId, String permissionId) throws Exception {
        client.delete(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/permissions/" + serializePathParameter(permissionId, new PathParameterSpec("permissionId", "simple", false)) + ""));
        return null;
    }

    public DrivePermissionHttpResponse permissionsUpdate(String nodeId, String permissionId, UpdatePermissionRequest body) throws Exception {
        Object raw = client.patch(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/permissions/" + serializePathParameter(permissionId, new PathParameterSpec("permissionId", "simple", false)) + ""), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<DrivePermissionHttpResponse>() {});
    }

    public DrivePermissionHttpResponse permissionsRetrieve(String nodeId, String permissionId) throws Exception {
        Object raw = client.get(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/permissions/" + serializePathParameter(permissionId, new PathParameterSpec("permissionId", "simple", false)) + ""));
        return client.convertValue(raw, new TypeReference<DrivePermissionHttpResponse>() {});
    }

    public EffectivePermissionListHttpResponse permissionsEffectiveList(String nodeId, Integer pageSize, String cursor) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null),
            new QueryParameterSpec("cursor", cursor, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/permissions/effective"), query));
        return client.convertValue(raw, new TypeReference<EffectivePermissionListHttpResponse>() {});
    }

    /** List node custom properties */
    public DriveNodePropertyListHttpResponse nodePropertiesList(String nodeId, String visibility, Integer pageSize, String cursor) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("visibility", visibility, "form", true, false, null),
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null),
            new QueryParameterSpec("cursor", cursor, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/properties"), query));
        return client.convertValue(raw, new TypeReference<DriveNodePropertyListHttpResponse>() {});
    }

    /** Create or update a node custom property */
    public DriveNodePropertyHttpResponse nodePropertiesUpdate(String nodeId, String propertyKey, SetNodePropertyRequest body) throws Exception {
        Object raw = client.put(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/properties/" + serializePathParameter(propertyKey, new PathParameterSpec("propertyKey", "simple", false)) + ""), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<DriveNodePropertyHttpResponse>() {});
    }

    /** Delete a node custom property */
    public Void nodePropertiesDelete(String nodeId, String propertyKey, String visibility) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("visibility", visibility, "form", true, false, null)
        ));
        client.delete(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/properties/" + serializePathParameter(propertyKey, new PathParameterSpec("propertyKey", "simple", false)) + ""), query));
        return null;
    }

    public CreateShareLinkHttpResponse shareLinksCreate(String nodeId, CreateShareLinkRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/share_links"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<CreateShareLinkHttpResponse>() {});
    }

    public ShareLinkListHttpResponse shareLinksList(String nodeId, Integer pageSize, String cursor) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null),
            new QueryParameterSpec("cursor", cursor, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/share_links"), query));
        return client.convertValue(raw, new TypeReference<ShareLinkListHttpResponse>() {});
    }

    public DriveNodeHttpResponse trashCreate(String nodeId, NodeCommandRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/trash"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<DriveNodeHttpResponse>() {});
    }

    public FileVersionListHttpResponse versionsList(String nodeId, Integer pageSize, String cursor) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null),
            new QueryParameterSpec("cursor", cursor, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/versions"), query));
        return client.convertValue(raw, new TypeReference<FileVersionListHttpResponse>() {});
    }

    public Void versionsDelete(String nodeId, String versionId) throws Exception {
        client.delete(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/versions/" + serializePathParameter(versionId, new PathParameterSpec("versionId", "simple", false)) + ""));
        return null;
    }

    public FileVersionHttpResponse versionsRetrieve(String nodeId, String versionId) throws Exception {
        Object raw = client.get(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/versions/" + serializePathParameter(versionId, new PathParameterSpec("versionId", "simple", false)) + ""));
        return client.convertValue(raw, new TypeReference<FileVersionHttpResponse>() {});
    }

    public DriveNodeHttpResponse versionsRestore(String nodeId, String versionId, NodeCommandRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/versions/" + serializePathParameter(versionId, new PathParameterSpec("versionId", "simple", false)) + "/restore"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<DriveNodeHttpResponse>() {});
    }

    public CreateFileHttpResponse nodesFilesCreate(CreateFileRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/nodes/files"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<CreateFileHttpResponse>() {});
    }

    public DriveNodeHttpResponse nodesFoldersCreate(CreateFolderRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/nodes/folders"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<DriveNodeHttpResponse>() {});
    }

    /** Create a shortcut node */
    public DriveNodeHttpResponse nodesShortcutsCreate(CreateShortcutRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/nodes/shortcuts"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<DriveNodeHttpResponse>() {});
    }

    /** List nodes carrying an app_public property */
    public DriveNodeListHttpResponse propertyNodesList(String propertyKey, Integer pageSize, String cursor) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null),
            new QueryParameterSpec("cursor", cursor, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/properties/" + serializePathParameter(propertyKey, new PathParameterSpec("propertyKey", "simple", false)) + "/nodes"), query));
        return client.convertValue(raw, new TypeReference<DriveNodeListHttpResponse>() {});
    }

    public DriveNodeListHttpResponse recentList(String spaceId, Integer pageSize, String cursor, String sortBy, String sortOrder) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("spaceId", spaceId, "form", true, false, null),
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null),
            new QueryParameterSpec("cursor", cursor, "form", true, false, null),
            new QueryParameterSpec("sortBy", sortBy, "form", true, false, null),
            new QueryParameterSpec("sortOrder", sortOrder, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/recent"), query));
        return client.convertValue(raw, new TypeReference<DriveNodeListHttpResponse>() {});
    }

    public DriveNodeListHttpResponse searchList(String q, String spaceId, Integer pageSize, String cursor) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("q", q, "form", true, false, null),
            new QueryParameterSpec("spaceId", spaceId, "form", true, false, null),
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null),
            new QueryParameterSpec("cursor", cursor, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/search"), query));
        return client.convertValue(raw, new TypeReference<DriveNodeListHttpResponse>() {});
    }

    public ClaimShareLinkHttpResponse shareLinksClaim(String token) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/share_links/" + serializePathParameter(token, new PathParameterSpec("token", "simple", false)) + "/claim"), null);
        return client.convertValue(raw, new TypeReference<ClaimShareLinkHttpResponse>() {});
    }

    public Void shareLinksDelete(String shareLinkId) throws Exception {
        client.delete(ApiPaths.appPath("/drive/share_links/" + serializePathParameter(shareLinkId, new PathParameterSpec("shareLinkId", "simple", false)) + ""));
        return null;
    }

    public ShareLinkHttpResponse shareLinksUpdate(String shareLinkId, UpdateShareLinkRequest body) throws Exception {
        Object raw = client.patch(ApiPaths.appPath("/drive/share_links/" + serializePathParameter(shareLinkId, new PathParameterSpec("shareLinkId", "simple", false)) + ""), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<ShareLinkHttpResponse>() {});
    }

    public ShareLinkHttpResponse shareLinksRetrieve(String shareLinkId) throws Exception {
        Object raw = client.get(ApiPaths.appPath("/drive/share_links/" + serializePathParameter(shareLinkId, new PathParameterSpec("shareLinkId", "simple", false)) + ""));
        return client.convertValue(raw, new TypeReference<ShareLinkHttpResponse>() {});
    }

    public DriveNodeListHttpResponse sharedWithMeList(String spaceId, Integer pageSize, String cursor, String sortBy, String sortOrder) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("spaceId", spaceId, "form", true, false, null),
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null),
            new QueryParameterSpec("cursor", cursor, "form", true, false, null),
            new QueryParameterSpec("sortBy", sortBy, "form", true, false, null),
            new QueryParameterSpec("sortOrder", sortOrder, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/shared_with_me"), query));
        return client.convertValue(raw, new TypeReference<DriveNodeListHttpResponse>() {});
    }

    public DriveSandboxVolumeListHttpResponse sandboxesList(Integer page, Integer pageSize) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("page", page, "form", true, false, null),
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/sandboxes"), query));
        return client.convertValue(raw, new TypeReference<DriveSandboxVolumeListHttpResponse>() {});
    }

    public DriveSandboxEntryListHttpResponse sandboxEntriesList(String sandboxId, String parentPath, String cursor, Integer pageSize) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("parent_path", parentPath, "form", true, false, null),
            new QueryParameterSpec("cursor", cursor, "form", true, false, null),
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/sandboxes/" + serializePathParameter(sandboxId, new PathParameterSpec("sandboxId", "simple", false)) + "/entries"), query));
        return client.convertValue(raw, new TypeReference<DriveSandboxEntryListHttpResponse>() {});
    }

    public DriveSandboxEntryHttpResponse sandboxDirectoriesCreate(String sandboxId, CreateDriveSandboxDirectoryRequest body, String idempotencyKey) throws Exception {
        Map<String, String> requestHeaders = buildRequestHeaders(
                Map.of("Idempotency-Key", new HeaderParameterSpec(idempotencyKey, "simple", false, null)),
                Map.of()
        );
        Object raw = client.post(ApiPaths.appPath("/drive/sandboxes/" + serializePathParameter(sandboxId, new PathParameterSpec("sandboxId", "simple", false)) + "/directories"), body, null, requestHeaders, "application/json");
        return client.convertValue(raw, new TypeReference<DriveSandboxEntryHttpResponse>() {});
    }

    public DriveSandboxEntryHttpResponse sandboxFilesCreate(String sandboxId, CreateDriveSandboxFileRequest body, String idempotencyKey) throws Exception {
        Map<String, String> requestHeaders = buildRequestHeaders(
                Map.of("Idempotency-Key", new HeaderParameterSpec(idempotencyKey, "simple", false, null)),
                Map.of()
        );
        Object raw = client.post(ApiPaths.appPath("/drive/sandboxes/" + serializePathParameter(sandboxId, new PathParameterSpec("sandboxId", "simple", false)) + "/files"), body, null, requestHeaders, "application/json");
        return client.convertValue(raw, new TypeReference<DriveSandboxEntryHttpResponse>() {});
    }

    public DriveSandboxFileContentHttpResponse sandboxFileContentsRetrieve(String sandboxId, String entryId, String logicalPath, String encoding) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("logical_path", logicalPath, "form", true, false, null),
            new QueryParameterSpec("encoding", encoding, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/sandboxes/" + serializePathParameter(sandboxId, new PathParameterSpec("sandboxId", "simple", false)) + "/files/" + serializePathParameter(entryId, new PathParameterSpec("entryId", "simple", false)) + "/content"), query));
        return client.convertValue(raw, new TypeReference<DriveSandboxFileContentHttpResponse>() {});
    }

    public DriveSandboxEntryHttpResponse sandboxFileContentsUpdate(String sandboxId, String entryId, UpdateDriveSandboxFileContentRequest body, String ifMatch, String idempotencyKey) throws Exception {
        Map<String, String> requestHeaders = buildRequestHeaders(
                Map.of("If-Match", new HeaderParameterSpec(ifMatch, "simple", false, null), "Idempotency-Key", new HeaderParameterSpec(idempotencyKey, "simple", false, null)),
                Map.of()
        );
        Object raw = client.put(ApiPaths.appPath("/drive/sandboxes/" + serializePathParameter(sandboxId, new PathParameterSpec("sandboxId", "simple", false)) + "/files/" + serializePathParameter(entryId, new PathParameterSpec("entryId", "simple", false)) + "/content"), body, null, requestHeaders, "application/json");
        return client.convertValue(raw, new TypeReference<DriveSandboxEntryHttpResponse>() {});
    }

    public DriveSandboxEntryHttpResponse sandboxEntriesUpdate(String sandboxId, String entryId, UpdateDriveSandboxEntryRequest body, String ifMatch, String idempotencyKey) throws Exception {
        Map<String, String> requestHeaders = buildRequestHeaders(
                Map.of("If-Match", new HeaderParameterSpec(ifMatch, "simple", false, null), "Idempotency-Key", new HeaderParameterSpec(idempotencyKey, "simple", false, null)),
                Map.of()
        );
        Object raw = client.patch(ApiPaths.appPath("/drive/sandboxes/" + serializePathParameter(sandboxId, new PathParameterSpec("sandboxId", "simple", false)) + "/entries/" + serializePathParameter(entryId, new PathParameterSpec("entryId", "simple", false)) + ""), body, null, requestHeaders, "application/json");
        return client.convertValue(raw, new TypeReference<DriveSandboxEntryHttpResponse>() {});
    }

    public DriveSandboxMutationCommandHttpResponse sandboxEntriesPurge(String sandboxId, String entryId, PurgeDriveSandboxEntryRequest body, String ifMatch, String idempotencyKey) throws Exception {
        Map<String, String> requestHeaders = buildRequestHeaders(
                Map.of("If-Match", new HeaderParameterSpec(ifMatch, "simple", false, null), "Idempotency-Key", new HeaderParameterSpec(idempotencyKey, "simple", false, null)),
                Map.of()
        );
        Object raw = client.post(ApiPaths.appPath("/drive/sandboxes/" + serializePathParameter(sandboxId, new PathParameterSpec("sandboxId", "simple", false)) + "/entries/" + serializePathParameter(entryId, new PathParameterSpec("entryId", "simple", false)) + "/purge"), body, null, requestHeaders, "application/json");
        return client.convertValue(raw, new TypeReference<DriveSandboxMutationCommandHttpResponse>() {});
    }

    public DriveSpaceListHttpResponse spacesList(String ownerSubjectType, String ownerSubjectId, String spaceType, Integer pageSize, String cursor) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("ownerSubjectType", ownerSubjectType, "form", true, false, null),
            new QueryParameterSpec("ownerSubjectId", ownerSubjectId, "form", true, false, null),
            new QueryParameterSpec("spaceType", spaceType, "form", true, false, null),
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null),
            new QueryParameterSpec("cursor", cursor, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/spaces"), query));
        return client.convertValue(raw, new TypeReference<DriveSpaceListHttpResponse>() {});
    }

    public DriveSpaceHttpResponse spacesCreate(CreateSpaceRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/spaces"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<DriveSpaceHttpResponse>() {});
    }

    public WebsiteRootListHttpResponse websiteRootsList(String spaceId, Integer pageSize, String cursor) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null),
            new QueryParameterSpec("cursor", cursor, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/spaces/" + serializePathParameter(spaceId, new PathParameterSpec("spaceId", "simple", false)) + "/website_roots"), query));
        return client.convertValue(raw, new TypeReference<WebsiteRootListHttpResponse>() {});
    }

    public WebsiteRootHttpResponse websiteRootsCreate(String spaceId, CreateWebsiteRootRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/spaces/" + serializePathParameter(spaceId, new PathParameterSpec("spaceId", "simple", false)) + "/website_roots"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<WebsiteRootHttpResponse>() {});
    }

    public WebsiteRootHttpResponse websiteRootsRetrieve(String rootUuid) throws Exception {
        Object raw = client.get(ApiPaths.appPath("/drive/website_roots/" + serializePathParameter(rootUuid, new PathParameterSpec("rootUuid", "simple", false)) + ""));
        return client.convertValue(raw, new TypeReference<WebsiteRootHttpResponse>() {});
    }

    /** Create an isolated atomic website synchronization */
    public WebsiteSyncHttpResponse websiteRootsSyncsCreate(String rootUuid, CreateWebsiteSyncRequest body, String idempotencyKey) throws Exception {
        Map<String, String> requestHeaders = buildRequestHeaders(
                Map.of("Idempotency-Key", new HeaderParameterSpec(idempotencyKey, "simple", false, null)),
                Map.of()
        );
        Object raw = client.post(ApiPaths.appPath("/drive/website_roots/" + serializePathParameter(rootUuid, new PathParameterSpec("rootUuid", "simple", false)) + "/syncs"), body, null, requestHeaders, "application/json");
        return client.convertValue(raw, new TypeReference<WebsiteSyncHttpResponse>() {});
    }

    /** Retrieve an atomic website synchronization */
    public WebsiteSyncHttpResponse websiteRootsSyncsRetrieve(String rootUuid, String syncId) throws Exception {
        Object raw = client.get(ApiPaths.appPath("/drive/website_roots/" + serializePathParameter(rootUuid, new PathParameterSpec("rootUuid", "simple", false)) + "/syncs/" + serializePathParameter(syncId, new PathParameterSpec("syncId", "simple", false)) + ""));
        return client.convertValue(raw, new TypeReference<WebsiteSyncHttpResponse>() {});
    }

    /** Validate and atomically activate a complete website tree */
    public WebsiteSyncActivationHttpResponse websiteRootsSyncsFinalize(String rootUuid, String syncId, WebsiteSyncVersionRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/website_roots/" + serializePathParameter(rootUuid, new PathParameterSpec("rootUuid", "simple", false)) + "/syncs/" + serializePathParameter(syncId, new PathParameterSpec("syncId", "simple", false)) + "/finalize"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<WebsiteSyncActivationHttpResponse>() {});
    }

    /** Abort an unactivated website synchronization */
    public WebsiteSyncHttpResponse websiteRootsSyncsAbort(String rootUuid, String syncId, WebsiteSyncVersionRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/website_roots/" + serializePathParameter(rootUuid, new PathParameterSpec("rootUuid", "simple", false)) + "/syncs/" + serializePathParameter(syncId, new PathParameterSpec("syncId", "simple", false)) + "/abort"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<WebsiteSyncHttpResponse>() {});
    }

    /** Activate a retained website generation as a new logical generation */
    public WebsiteGenerationActivationHttpResponse websiteRootsGenerationsActivate(String rootUuid, String generation, ActivateWebsiteGenerationRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/website_roots/" + serializePathParameter(rootUuid, new PathParameterSpec("rootUuid", "simple", false)) + "/generations/" + serializePathParameter(generation, new PathParameterSpec("generation", "simple", false)) + "/activate"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<WebsiteGenerationActivationHttpResponse>() {});
    }

    public DriveNodeListHttpResponse moveDestinationsList(String spaceId, String excludeNodeIds, Integer pageSize, String cursor) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("excludeNodeIds", excludeNodeIds, "form", true, false, null),
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null),
            new QueryParameterSpec("cursor", cursor, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/spaces/" + serializePathParameter(spaceId, new PathParameterSpec("spaceId", "simple", false)) + "/move_destinations"), query));
        return client.convertValue(raw, new TypeReference<DriveNodeListHttpResponse>() {});
    }

    public DriveSpaceHttpResponse spacesRetrieve(String spaceId) throws Exception {
        Object raw = client.get(ApiPaths.appPath("/drive/spaces/" + serializePathParameter(spaceId, new PathParameterSpec("spaceId", "simple", false)) + ""));
        return client.convertValue(raw, new TypeReference<DriveSpaceHttpResponse>() {});
    }

    public DriveSpaceHttpResponse spacesUpdate(String spaceId, UpdateSpaceRequest body) throws Exception {
        Object raw = client.patch(ApiPaths.appPath("/drive/spaces/" + serializePathParameter(spaceId, new PathParameterSpec("spaceId", "simple", false)) + ""), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<DriveSpaceHttpResponse>() {});
    }

    public Void spacesDelete(String spaceId) throws Exception {
        client.delete(ApiPaths.appPath("/drive/spaces/" + serializePathParameter(spaceId, new PathParameterSpec("spaceId", "simple", false)) + ""));
        return null;
    }

    public DriveNodeListHttpResponse nodesList(String spaceId, String parentNodeId, Integer pageSize, String cursor, String sortBy, String sortOrder) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("parentNodeId", parentNodeId, "form", true, false, null),
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null),
            new QueryParameterSpec("cursor", cursor, "form", true, false, null),
            new QueryParameterSpec("sortBy", sortBy, "form", true, false, null),
            new QueryParameterSpec("sortOrder", sortOrder, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/spaces/" + serializePathParameter(spaceId, new PathParameterSpec("spaceId", "simple", false)) + "/nodes"), query));
        return client.convertValue(raw, new TypeReference<DriveNodeListHttpResponse>() {});
    }

    public DriveNodeListHttpResponse trashList(String spaceId, Integer pageSize, String cursor, String parentNodeId, String sortBy, String sortOrder) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("spaceId", spaceId, "form", true, false, null),
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null),
            new QueryParameterSpec("cursor", cursor, "form", true, false, null),
            new QueryParameterSpec("parentNodeId", parentNodeId, "form", true, false, null),
            new QueryParameterSpec("sortBy", sortBy, "form", true, false, null),
            new QueryParameterSpec("sortOrder", sortOrder, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/trash"), query));
        return client.convertValue(raw, new TypeReference<DriveNodeListHttpResponse>() {});
    }

    public DriveNodeHttpResponse trashRestore(String nodeId, NodeCommandRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/trash/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/restore"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<DriveNodeHttpResponse>() {});
    }

    public EmptyTrashHttpResponse trashEmpty(EmptyTrashRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/trash/empty"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<EmptyTrashHttpResponse>() {});
    }

    public DriveUploadSessionHttpResponse uploadSessionsCreate(CreateUploadSessionRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/upload_sessions"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<DriveUploadSessionHttpResponse>() {});
    }

    public DriveUploadSessionHttpResponse uploadSessionsRetrieve(String uploadSessionId) throws Exception {
        Object raw = client.get(ApiPaths.appPath("/drive/upload_sessions/" + serializePathParameter(uploadSessionId, new PathParameterSpec("uploadSessionId", "simple", false)) + ""));
        return client.convertValue(raw, new TypeReference<DriveUploadSessionHttpResponse>() {});
    }

    public DriveUploadSessionHttpResponse uploadSessionsAbort(String uploadSessionId, NodeCommandRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/upload_sessions/" + serializePathParameter(uploadSessionId, new PathParameterSpec("uploadSessionId", "simple", false)) + "/abort"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<DriveUploadSessionHttpResponse>() {});
    }

    public DriveUploadSessionHttpResponse uploadSessionsComplete(String uploadSessionId, CompleteUploadSessionRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/upload_sessions/" + serializePathParameter(uploadSessionId, new PathParameterSpec("uploadSessionId", "simple", false)) + "/complete"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<DriveUploadSessionHttpResponse>() {});
    }

    public PresignedUploadPartHttpResponse uploadSessionsPartsUpdate(String uploadSessionId, Integer partNo, PresignUploadPartRequest body) throws Exception {
        Object raw = client.put(ApiPaths.appPath("/drive/upload_sessions/" + serializePathParameter(uploadSessionId, new PathParameterSpec("uploadSessionId", "simple", false)) + "/parts/" + serializePathParameter(partNo, new PathParameterSpec("partNo", "simple", false)) + ""), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<PresignedUploadPartHttpResponse>() {});
    }

    /** Create a push notification channel for Drive changes */
    public DriveWatchChannelHttpResponse changesWatch(CreateWatchChannelRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/changes/watch"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<DriveWatchChannelHttpResponse>() {});
    }

    /** Create a push notification channel for a Drive node */
    public DriveWatchChannelHttpResponse nodesWatch(String nodeId, CreateWatchChannelRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/watch"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<DriveWatchChannelHttpResponse>() {});
    }

    /** List Drive watch channels */
    public DriveWatchChannelListHttpResponse watchChannelsList(String resourceType, String lifecycleStatus, Integer pageSize, String cursor) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("resourceType", resourceType, "form", true, false, null),
            new QueryParameterSpec("lifecycleStatus", lifecycleStatus, "form", true, false, null),
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null),
            new QueryParameterSpec("cursor", cursor, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/drive/watch_channels"), query));
        return client.convertValue(raw, new TypeReference<DriveWatchChannelListHttpResponse>() {});
    }

    /** Get a Drive watch channel */
    public DriveWatchChannelHttpResponse watchChannelsRetrieve(String channelId) throws Exception {
        Object raw = client.get(ApiPaths.appPath("/drive/watch_channels/" + serializePathParameter(channelId, new PathParameterSpec("channelId", "simple", false)) + ""));
        return client.convertValue(raw, new TypeReference<DriveWatchChannelHttpResponse>() {});
    }

    /** Stop a Drive watch channel */
    public StopWatchChannelHttpResponse watchChannelsStop(String channelId, StopWatchChannelRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/watch_channels/" + serializePathParameter(channelId, new PathParameterSpec("channelId", "simple", false)) + "/stop"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<StopWatchChannelHttpResponse>() {});
    }

    public DownloadPackageHttpResponse downloadPackagesCreate(CreateDownloadPackageRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/download_packages"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<DownloadPackageHttpResponse>() {});
    }

    public DownloadPackageHttpResponse downloadPackagesUrlsRetrieve(String packageId) throws Exception {
        Object raw = client.get(ApiPaths.appPath("/drive/download_packages/" + serializePathParameter(packageId, new PathParameterSpec("packageId", "simple", false)) + "/download_url"));
        return client.convertValue(raw, new TypeReference<DownloadPackageHttpResponse>() {});
    }

    public ArchiveEntryListHttpResponse archiveEntriesList(String nodeId) throws Exception {
        Object raw = client.get(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/archive_entries"));
        return client.convertValue(raw, new TypeReference<ArchiveEntryListHttpResponse>() {});
    }

    public ExtractArchiveEntriesHttpResponse archiveEntriesExtract(String nodeId, ExtractArchiveEntriesRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/nodes/" + serializePathParameter(nodeId, new PathParameterSpec("nodeId", "simple", false)) + "/archive_entries/extract"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<ExtractArchiveEntriesHttpResponse>() {});
    }

    public PrepareUploaderUploadHttpResponse uploaderUploadsCreate(PrepareUploaderUploadRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/drive/uploader/uploads"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<PrepareUploaderUploadHttpResponse>() {});
    }

    public UploaderUploadPartHttpResponse uploaderUploadsPartsUpdate(String uploadItemId, Integer partNo, MarkUploaderPartUploadedRequest body) throws Exception {
        Object raw = client.put(ApiPaths.appPath("/drive/uploader/uploads/" + serializePathParameter(uploadItemId, new PathParameterSpec("uploadItemId", "simple", false)) + "/parts/" + serializePathParameter(partNo, new PathParameterSpec("partNo", "simple", false)) + ""), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<UploaderUploadPartHttpResponse>() {});
    }

    /** List global assets */
    public AssetListHttpResponse assetsList(String cursor, Integer pageSize, String kind, String sourceType, String q) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("cursor", cursor, "form", true, false, null),
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null),
            new QueryParameterSpec("kind", kind, "form", true, false, null),
            new QueryParameterSpec("sourceType", sourceType, "form", true, false, null),
            new QueryParameterSpec("q", q, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.appPath("/assets"), query));
        return client.convertValue(raw, new TypeReference<AssetListHttpResponse>() {});
    }

    /** Create a global asset metadata record */
    public AssetItemHttpResponse assetsCreate(CreateAssetRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/assets"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<AssetItemHttpResponse>() {});
    }

    /** Get a global asset */
    public AssetItemHttpResponse assetsRetrieve(String assetId) throws Exception {
        Object raw = client.get(ApiPaths.appPath("/assets/" + serializePathParameter(assetId, new PathParameterSpec("assetId", "simple", false)) + ""));
        return client.convertValue(raw, new TypeReference<AssetItemHttpResponse>() {});
    }

    /** Update a global asset */
    public AssetItemHttpResponse assetsUpdate(String assetId, UpdateAssetRequest body) throws Exception {
        Object raw = client.patch(ApiPaths.appPath("/assets/" + serializePathParameter(assetId, new PathParameterSpec("assetId", "simple", false)) + ""), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<AssetItemHttpResponse>() {});
    }

    /** Archive a global asset */
    public AssetItemHttpResponse assetsArchive(String assetId, AssetActionRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/assets/" + serializePathParameter(assetId, new PathParameterSpec("assetId", "simple", false)) + "/archive"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<AssetItemHttpResponse>() {});
    }

    /** Restore an archived global asset */
    public AssetItemHttpResponse assetsRestore(String assetId, AssetActionRequest body) throws Exception {
        Object raw = client.post(ApiPaths.appPath("/assets/" + serializePathParameter(assetId, new PathParameterSpec("assetId", "simple", false)) + "/restore"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<AssetItemHttpResponse>() {});
    }

    private record PathParameterSpec(String name, String style, boolean explode) {}

    private static String serializePathParameter(Object value, PathParameterSpec spec) {
        if (value == null) {
            return "";
        }
        String style = spec.style() == null || spec.style().isBlank() ? "simple" : spec.style();
        if (value instanceof Iterable<?> iterable) {
            return serializePathArray(spec.name(), iterable, style, spec.explode());
        }
        if (value instanceof Map<?, ?> map) {
            return serializePathObject(spec.name(), map, style, spec.explode());
        }
        return pathPrimitivePrefix(spec.name(), style) + pathEncode(String.valueOf(value));
    }

    private static String serializePathArray(String name, Iterable<?> values, String style, boolean explode) {
        List<String> serialized = new java.util.ArrayList<>();
        for (Object item : values) {
            if (item != null) {
                serialized.add(pathEncode(String.valueOf(item)));
            }
        }
        if (serialized.isEmpty()) {
            return pathPrefix(name, style);
        }
        if ("matrix".equals(style)) {
            if (explode) {
                List<String> parts = new java.util.ArrayList<>();
                for (String item : serialized) {
                    parts.add(";" + name + "=" + item);
                }
                return String.join("", parts);
            }
            return ";" + name + "=" + String.join(",", serialized);
        }
        String separator = explode ? "." : ",";
        return pathPrefix(name, style) + String.join(separator, serialized);
    }

    private static String serializePathObject(String name, Map<?, ?> values, String style, boolean explode) {
        List<String> entries = new java.util.ArrayList<>();
        List<String> exploded = new java.util.ArrayList<>();
        values.forEach((key, value) -> {
            if (value == null) {
                return;
            }
            String escapedKey = pathEncode(String.valueOf(key));
            String escapedValue = pathEncode(String.valueOf(value));
            if (explode) {
                if ("matrix".equals(style)) {
                    exploded.add(";" + escapedKey + "=" + escapedValue);
                } else {
                    exploded.add(escapedKey + "=" + escapedValue);
                }
            } else {
                entries.add(escapedKey);
                entries.add(escapedValue);
            }
        });
        if ("matrix".equals(style)) {
            if (explode) {
                return String.join("", exploded);
            }
            return ";" + name + "=" + String.join(",", entries);
        }
        if (explode) {
            String separator = "label".equals(style) ? "." : ",";
            return pathPrefix(name, style) + String.join(separator, exploded);
        }
        return pathPrefix(name, style) + String.join(",", entries);
    }

    private static String pathPrefix(String name, String style) {
        if ("label".equals(style)) {
            return ".";
        }
        if ("matrix".equals(style)) {
            return ";" + name;
        }
        return "";
    }

    private static String pathPrimitivePrefix(String name, String style) {
        if ("matrix".equals(style)) {
            return ";" + name + "=";
        }
        return pathPrefix(name, style);
    }

    private static String pathEncode(String value) {
        return java.net.URLEncoder.encode(value, java.nio.charset.StandardCharsets.UTF_8).replace("+", "%20");
    }

    private record QueryParameterSpec(String name, Object value, String style, boolean explode, boolean allowReserved, String contentType) {}

    private static String buildQueryString(List<QueryParameterSpec> parameters) throws Exception {
        List<String> pairs = new java.util.ArrayList<>();
        for (QueryParameterSpec parameter : parameters) {
            appendSerializedParameter(pairs, parameter);
        }
        return String.join("&", pairs);
    }

    private static void appendSerializedParameter(List<String> pairs, QueryParameterSpec parameter) throws Exception {
        if (parameter.value() == null) {
            return;
        }
        if (parameter.contentType() != null && !parameter.contentType().isBlank()) {
            String json = clientObjectMapper().writeValueAsString(parameter.value());
            pairs.add(urlEncode(parameter.name()) + "=" + encodeQueryValue(json, parameter.allowReserved()));
            return;
        }

        String style = parameter.style() == null || parameter.style().isBlank() ? "form" : parameter.style();
        Object value = parameter.value();
        if ("deepObject".equals(style) && value instanceof Map<?, ?> map) {
            appendDeepObjectParameter(pairs, parameter.name(), map, parameter.allowReserved());
        } else if (value instanceof Iterable<?> iterable) {
            appendArrayParameter(pairs, parameter.name(), iterable, style, parameter.explode(), parameter.allowReserved());
        } else if (value instanceof Map<?, ?> map) {
            appendObjectParameter(pairs, parameter.name(), map, style, parameter.explode(), parameter.allowReserved());
        } else {
            pairs.add(urlEncode(parameter.name()) + "=" + encodeQueryValue(String.valueOf(value), parameter.allowReserved()));
        }
    }

    private static void appendArrayParameter(List<String> pairs, String name, Iterable<?> values, String style, boolean explode, boolean allowReserved) {
        List<String> serialized = new java.util.ArrayList<>();
        for (Object item : values) {
            if (item != null) {
                serialized.add(String.valueOf(item));
            }
        }
        if (serialized.isEmpty()) {
            return;
        }
        if ("form".equals(style) && explode) {
            for (String item : serialized) {
                pairs.add(urlEncode(name) + "=" + encodeQueryValue(item, allowReserved));
            }
            return;
        }
        pairs.add(urlEncode(name) + "=" + encodeQueryValue(String.join(",", serialized), allowReserved));
    }

    private static void appendObjectParameter(List<String> pairs, String name, Map<?, ?> values, String style, boolean explode, boolean allowReserved) {
        List<String> serialized = new java.util.ArrayList<>();
        values.forEach((key, value) -> {
            if (value == null) {
                return;
            }
            if ("form".equals(style) && explode) {
                pairs.add(urlEncode(String.valueOf(key)) + "=" + encodeQueryValue(String.valueOf(value), allowReserved));
            } else {
                serialized.add(String.valueOf(key));
                serialized.add(String.valueOf(value));
            }
        });
        if (!serialized.isEmpty()) {
            pairs.add(urlEncode(name) + "=" + encodeQueryValue(String.join(",", serialized), allowReserved));
        }
    }

    private static void appendDeepObjectParameter(List<String> pairs, String name, Map<?, ?> values, boolean allowReserved) {
        values.forEach((key, value) -> {
            if (value != null) {
                pairs.add(urlEncode(name + "[" + key + "]") + "=" + encodeQueryValue(String.valueOf(value), allowReserved));
            }
        });
    }

    private static String encodeQueryValue(String value, boolean allowReserved) {
        String encoded = urlEncode(value);
        if (!allowReserved) {
            return encoded;
        }
        return encoded
            .replace("%3A", ":").replace("%2F", "/").replace("%3F", "?").replace("%23", "#")
            .replace("%5B", "[").replace("%5D", "]").replace("%40", "@").replace("%21", "!")
            .replace("%24", "$").replace("%26", "&").replace("%27", "'").replace("%28", "(")
            .replace("%29", ")").replace("%2A", "*").replace("%2B", "+").replace("%2C", ",")
            .replace("%3B", ";").replace("%3D", "=");
    }

    private static com.fasterxml.jackson.databind.ObjectMapper clientObjectMapper() {
        return new com.fasterxml.jackson.databind.ObjectMapper();
    }

    private record HeaderParameterSpec(Object value, String style, boolean explode, String contentType) {}

    private static Map<String, String> buildRequestHeaders(Map<String, HeaderParameterSpec> headers, Map<String, HeaderParameterSpec> cookies) throws Exception {
        Map<String, String> requestHeaders = new java.util.LinkedHashMap<>();
        for (Map.Entry<String, HeaderParameterSpec> entry : headers.entrySet()) {
            String serialized = serializeParameterValue(entry.getValue());
            if (serialized != null) {
                requestHeaders.put(entry.getKey(), serialized);
            }
        }

        String cookieHeader = buildCookieHeader(cookies);
        if (cookieHeader != null && !cookieHeader.isEmpty()) {
            requestHeaders.merge("Cookie", cookieHeader, (left, right) -> left + "; " + right);
        }

        return requestHeaders.isEmpty() ? null : requestHeaders;
    }

    private static String buildCookieHeader(Map<String, HeaderParameterSpec> cookies) throws Exception {
        java.util.List<String> pairs = new java.util.ArrayList<>();
        for (Map.Entry<String, HeaderParameterSpec> entry : cookies.entrySet()) {
            String serialized = serializeParameterValue(entry.getValue());
            if (serialized != null) {
                pairs.add(urlEncode(entry.getKey()) + "=" + urlEncode(serialized));
            }
        }
        return String.join("; ", pairs);
    }

    private static String serializeParameterValue(HeaderParameterSpec parameter) throws Exception {
        if (parameter == null || parameter.value() == null) {
            return null;
        }
        Object value = parameter.value();
        if (parameter.contentType() != null && !parameter.contentType().isBlank()) {
            return headerObjectMapper().writeValueAsString(value);
        }
        if (value instanceof Iterable<?> iterable) {
            java.util.List<String> values = new java.util.ArrayList<>();
            for (Object item : iterable) {
                if (item != null) {
                    values.add(String.valueOf(item));
                }
            }
            return String.join(",", values);
        }
        if (value instanceof Map<?, ?> map) {
            java.util.List<String> values = new java.util.ArrayList<>();
            map.forEach((key, item) -> {
                if (item == null) {
                    return;
                }
                if (parameter.explode()) {
                    values.add(String.valueOf(key) + "=" + String.valueOf(item));
                } else {
                    values.add(String.valueOf(key));
                    values.add(String.valueOf(item));
                }
            });
            return String.join(",", values);
        }
        return String.valueOf(value);
    }

    private static com.fasterxml.jackson.databind.ObjectMapper headerObjectMapper() {
        return new com.fasterxml.jackson.databind.ObjectMapper();
    }

    private static String urlEncode(String value) {
        return java.net.URLEncoder.encode(value, java.nio.charset.StandardCharsets.UTF_8);
    }
}
