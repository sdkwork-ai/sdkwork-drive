export declare const sdkMetadata: {
    name: string;
    packageName: string;
    sdkOwner: string;
    apiAuthority: string;
    language: string;
    standardProfile: string;
    baseUrl: string;
    apiPrefix: string;
    sdkDependencies: {
        workspace: string;
        role: string;
        required: boolean;
        dependencyMode: string;
        apiPrefix: string;
        apiAuthority: string;
        generatedTransportImportPolicy: string;
        packageByLanguage: {
            typescript: string;
            rust: string;
            java: string;
            python: string;
            go: string;
        };
    }[];
};
export declare const operations: {
    readonly "archiveEntries.extract": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/archive_entries/extract";
    };
    readonly "archiveEntries.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/archive_entries";
    };
    readonly "assetCollectionItems.create": {
        readonly method: "POST";
        readonly path: "/app/v3/api/assets/collections/{collectionId}/items";
    };
    readonly "assetCollectionItems.delete": {
        readonly method: "DELETE";
        readonly path: "/app/v3/api/assets/collections/{collectionId}/items/{itemId}";
    };
    readonly "assetCollections.create": {
        readonly method: "POST";
        readonly path: "/app/v3/api/assets/collections";
    };
    readonly "assetCollections.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/assets/collections";
    };
    readonly "assetRelations.create": {
        readonly method: "POST";
        readonly path: "/app/v3/api/assets/{assetId}/relations";
    };
    readonly "assetRelations.delete": {
        readonly method: "DELETE";
        readonly path: "/app/v3/api/assets/{assetId}/relations/{relationId}";
    };
    readonly "assets.archive": {
        readonly method: "POST";
        readonly path: "/app/v3/api/assets/{assetId}/archive";
    };
    readonly "assets.create": {
        readonly method: "POST";
        readonly path: "/app/v3/api/assets";
    };
    readonly "assets.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/assets";
    };
    readonly "assets.restore": {
        readonly method: "POST";
        readonly path: "/app/v3/api/assets/{assetId}/restore";
    };
    readonly "assets.retrieve": {
        readonly method: "GET";
        readonly path: "/app/v3/api/assets/{assetId}";
    };
    readonly "assets.update": {
        readonly method: "PATCH";
        readonly path: "/app/v3/api/assets/{assetId}";
    };
    readonly "changes.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/changes";
    };
    readonly "changes.startPageToken.retrieve": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/changes/start_page_token";
    };
    readonly "changes.watch": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/changes/watch";
    };
    readonly "commentReplies.create": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/comments/{commentId}/replies";
    };
    readonly "commentReplies.delete": {
        readonly method: "DELETE";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/comments/{commentId}/replies/{replyId}";
    };
    readonly "commentReplies.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/comments/{commentId}/replies";
    };
    readonly "commentReplies.retrieve": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/comments/{commentId}/replies/{replyId}";
    };
    readonly "commentReplies.update": {
        readonly method: "PATCH";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/comments/{commentId}/replies/{replyId}";
    };
    readonly "comments.create": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/comments";
    };
    readonly "comments.delete": {
        readonly method: "DELETE";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/comments/{commentId}";
    };
    readonly "comments.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/comments";
    };
    readonly "comments.retrieve": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/comments/{commentId}";
    };
    readonly "comments.update": {
        readonly method: "PATCH";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/comments/{commentId}";
    };
    readonly "downloadGrants.create": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/download_grants";
    };
    readonly "downloadPackages.create": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/download_packages";
    };
    readonly "downloadPackages.downloadUrls.retrieve": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/download_packages/{packageId}/download_url";
    };
    readonly "downloadTokens.retrieve": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/download_tokens/{token}";
    };
    readonly "downloadUrls.create": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/download_urls";
    };
    readonly "favorites.check": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/favorites/check";
    };
    readonly "favorites.delete": {
        readonly method: "DELETE";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/favorite";
    };
    readonly "favorites.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/favorites";
    };
    readonly "favorites.update": {
        readonly method: "PUT";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/favorite";
    };
    readonly "moveDestinations.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/spaces/{spaceId}/move_destinations";
    };
    readonly "nodeLabels.delete": {
        readonly method: "DELETE";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/labels/{labelId}";
    };
    readonly "nodeLabels.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/labels";
    };
    readonly "nodeLabels.update": {
        readonly method: "PUT";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/labels/{labelId}";
    };
    readonly "nodeProperties.delete": {
        readonly method: "DELETE";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/properties/{propertyKey}";
    };
    readonly "nodeProperties.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/properties";
    };
    readonly "nodeProperties.update": {
        readonly method: "PUT";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/properties/{propertyKey}";
    };
    readonly "nodes.capabilities.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/capabilities";
    };
    readonly "nodes.copy": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/copy";
    };
    readonly "nodes.delete": {
        readonly method: "DELETE";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}";
    };
    readonly "nodes.downloadUrls.retrieve": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/download_url";
    };
    readonly "nodes.files.create": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/nodes/files";
    };
    readonly "nodes.folders.create": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/nodes/folders";
    };
    readonly "nodes.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/spaces/{spaceId}/nodes";
    };
    readonly "nodes.move": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/move";
    };
    readonly "nodes.path.retrieve": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/path";
    };
    readonly "nodes.retrieve": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}";
    };
    readonly "nodes.shortcuts.create": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/nodes/shortcuts";
    };
    readonly "nodes.update": {
        readonly method: "PATCH";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}";
    };
    readonly "nodes.watch": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/watch";
    };
    readonly "permissions.create": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/permissions";
    };
    readonly "permissions.delete": {
        readonly method: "DELETE";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/permissions/{permissionId}";
    };
    readonly "permissions.effective.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/permissions/effective";
    };
    readonly "permissions.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/permissions";
    };
    readonly "permissions.retrieve": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/permissions/{permissionId}";
    };
    readonly "permissions.update": {
        readonly method: "PATCH";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/permissions/{permissionId}";
    };
    readonly "propertyNodes.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/properties/{propertyKey}/nodes";
    };
    readonly "quotas.retrieve": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/quotas/summary";
    };
    readonly "recent.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/recent";
    };
    readonly "sandboxDirectories.create": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/sandboxes/{sandboxId}/directories";
    };
    readonly "sandboxEntries.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/sandboxes/{sandboxId}/entries";
    };
    readonly "sandboxEntries.purge": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/sandboxes/{sandboxId}/entries/{entryId}/purge";
    };
    readonly "sandboxEntries.update": {
        readonly method: "PATCH";
        readonly path: "/app/v3/api/drive/sandboxes/{sandboxId}/entries/{entryId}";
    };
    readonly "sandboxes.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/sandboxes";
    };
    readonly "sandboxFileContents.retrieve": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/sandboxes/{sandboxId}/files/{entryId}/content";
    };
    readonly "sandboxFileContents.update": {
        readonly method: "PUT";
        readonly path: "/app/v3/api/drive/sandboxes/{sandboxId}/files/{entryId}/content";
    };
    readonly "sandboxFiles.create": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/sandboxes/{sandboxId}/files";
    };
    readonly "search.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/search";
    };
    readonly "sharedWithMe.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/shared_with_me";
    };
    readonly "shareLinks.claim": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/share_links/{token}/claim";
    };
    readonly "shareLinks.create": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/share_links";
    };
    readonly "shareLinks.delete": {
        readonly method: "DELETE";
        readonly path: "/app/v3/api/drive/share_links/{shareLinkId}";
    };
    readonly "shareLinks.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/share_links";
    };
    readonly "shareLinks.retrieve": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/share_links/{shareLinkId}";
    };
    readonly "shareLinks.update": {
        readonly method: "PATCH";
        readonly path: "/app/v3/api/drive/share_links/{shareLinkId}";
    };
    readonly "spaces.create": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/spaces";
    };
    readonly "spaces.delete": {
        readonly method: "DELETE";
        readonly path: "/app/v3/api/drive/spaces/{spaceId}";
    };
    readonly "spaces.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/spaces";
    };
    readonly "spaces.retrieve": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/spaces/{spaceId}";
    };
    readonly "spaces.update": {
        readonly method: "PATCH";
        readonly path: "/app/v3/api/drive/spaces/{spaceId}";
    };
    readonly "trash.create": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/trash";
    };
    readonly "trash.empty": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/trash/empty";
    };
    readonly "trash.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/trash";
    };
    readonly "trash.restore": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/trash/{nodeId}/restore";
    };
    readonly "uploader.uploads.create": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/uploader/uploads";
    };
    readonly "uploader.uploads.parts.update": {
        readonly method: "PUT";
        readonly path: "/app/v3/api/drive/uploader/uploads/{uploadItemId}/parts/{partNo}";
    };
    readonly "uploadSessions.abort": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/upload_sessions/{uploadSessionId}/abort";
    };
    readonly "uploadSessions.complete": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/upload_sessions/{uploadSessionId}/complete";
    };
    readonly "uploadSessions.create": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/upload_sessions";
    };
    readonly "uploadSessions.parts.update": {
        readonly method: "PUT";
        readonly path: "/app/v3/api/drive/upload_sessions/{uploadSessionId}/parts/{partNo}";
    };
    readonly "uploadSessions.retrieve": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/upload_sessions/{uploadSessionId}";
    };
    readonly "versions.delete": {
        readonly method: "DELETE";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/versions/{versionId}";
    };
    readonly "versions.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/versions";
    };
    readonly "versions.restore": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/versions/{versionId}/restore";
    };
    readonly "versions.retrieve": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/nodes/{nodeId}/versions/{versionId}";
    };
    readonly "watchChannels.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/watch_channels";
    };
    readonly "watchChannels.retrieve": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/watch_channels/{channelId}";
    };
    readonly "watchChannels.stop": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/watch_channels/{channelId}/stop";
    };
    readonly "websiteRoots.create": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/spaces/{spaceId}/website_roots";
    };
    readonly "websiteRoots.generations.activate": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/website_roots/{rootUuid}/generations/{generation}/activate";
    };
    readonly "websiteRoots.list": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/spaces/{spaceId}/website_roots";
    };
    readonly "websiteRoots.retrieve": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/website_roots/{rootUuid}";
    };
    readonly "websiteRoots.syncs.abort": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/website_roots/{rootUuid}/syncs/{syncId}/abort";
    };
    readonly "websiteRoots.syncs.create": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/website_roots/{rootUuid}/syncs";
    };
    readonly "websiteRoots.syncs.finalize": {
        readonly method: "POST";
        readonly path: "/app/v3/api/drive/website_roots/{rootUuid}/syncs/{syncId}/finalize";
    };
    readonly "websiteRoots.syncs.retrieve": {
        readonly method: "GET";
        readonly path: "/app/v3/api/drive/website_roots/{rootUuid}/syncs/{syncId}";
    };
};
//# sourceMappingURL=operations.d.ts.map