import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const testDir = path.dirname(fileURLToPath(import.meta.url));
const sdkRoot = path.resolve(testDir, "..");
const sdkName = "sdkwork-drive-app-sdk";
const apiPrefix = "/app/v3/api";
const languages = ["typescript", "rust", "java", "python", "go"];
const requiredOperations = [
  "sandboxes.list",
  "sandboxEntries.list",
  "sandboxDirectories.create",
  "spaces.list",
  "spaces.create",
  "spaces.retrieve",
  "spaces.update",
  "spaces.delete",
  "websiteRoots.list",
  "websiteRoots.create",
  "websiteRoots.retrieve",
  "nodes.list",
  "nodes.folders.create",
  "nodes.files.create",
  "nodes.shortcuts.create",
  "nodes.retrieve",
  "nodes.path.retrieve",
  "nodes.capabilities.list",
  "nodeProperties.list",
  "nodeProperties.update",
  "nodeProperties.delete",
  "nodeLabels.list",
  "nodeLabels.update",
  "nodeLabels.delete",
  "nodes.update",
  "nodes.move",
  "nodes.copy",
  "nodes.delete",
  "nodes.downloadUrls.retrieve",
  "archiveEntries.list",
  "archiveEntries.extract",
  "moveDestinations.list",
  "trash.empty",
  "trash.create",
  "trash.restore",
  "trash.list",
  "recent.list",
  "sharedWithMe.list",
  "favorites.list",
  "favorites.update",
  "favorites.delete",
  "quotas.retrieve",
  "uploadSessions.retrieve",
  "uploadSessions.create",
  "uploader.uploads.create",
  "uploader.uploads.parts.update",
  "uploadSessions.parts.update",
  "uploadSessions.complete",
  "uploadSessions.abort",
  "changes.list",
  "changes.startPageToken.retrieve",
  "changes.watch",
  "nodes.watch",
  "watchChannels.list",
  "watchChannels.retrieve",
  "watchChannels.stop",
  "permissions.create",
  "permissions.retrieve",
  "permissions.update",
  "permissions.delete",
  "permissions.list",
  "permissions.effective.list",
  "shareLinks.create",
  "shareLinks.retrieve",
  "shareLinks.list",
  "shareLinks.claim",
  "shareLinks.update",
  "shareLinks.delete",
  "comments.list",
  "comments.create",
  "comments.retrieve",
  "comments.update",
  "comments.delete",
  "commentReplies.list",
  "commentReplies.create",
  "commentReplies.retrieve",
  "commentReplies.update",
  "commentReplies.delete",
  "versions.list",
  "versions.retrieve",
  "versions.restore",
  "versions.delete",
];
const forbiddenStorageAdminOperations = [
  "storageProviders.list",
  "storageProviders.create",
  "storageProviders.retrieve",
  "storageProviders.update",
  "storageProviders.delete",
  "storageProviders.test",
  "storageProviders.capabilities.list",
  "storageProviders.activate",
  "storageProviders.deactivate",
  "storageProviders.credentials.rotate",
  "storageProviders.bucket.retrieve",
  "storageProviders.bucket.update",
  "storageProviders.bucket.delete",
  "storageProviders.objects.list",
  "storageProviders.objects.retrieve",
  "storageProviders.objects.delete",
  "storageProviders.objects.copy",
  "storageProviderBindings.default.retrieve",
  "storageProviderBindings.default.update",
];
const forbiddenStorageAdminSchemas = [
  "CopyProviderObjectRequest",
  "CreateStorageProviderRequest",
  "DeleteStorageProviderResponse",
  "ListStorageProvidersResponse",
  "OperatorRequest",
  "ProviderBucket",
  "ProviderBucketMutation",
  "ProviderObject",
  "ProviderObjectList",
  "ProviderObjectMutation",
  "RotateStorageProviderCredentialRequest",
  "SetDefaultStorageProviderBindingRequest",
  "StorageProvider",
  "StorageProviderBinding",
  "StorageProviderCapabilities",
  "TestStorageProviderRequest",
  "TestStorageProviderResponse",
  "UpdateStorageProviderRequest",
];
const forbiddenAppbaseAppOperations = [
  "oauth.authorizationUrls.create",
  "oauth.sessions.create",
  "passwordResetRequests.create",
  "passwordResets.create",
  "registrations.create",
  "sessions.create",
  "sessions.current.delete",
  "sessions.current.retrieve",
  "sessions.current.update",
  "sessions.organizationSelection.create",
  "sessions.refresh",
  "oauth.deviceAuthorizations.create",
  "oauth.deviceAuthorizations.retrieve",
  "oauth.deviceAuthorizations.scans.create",
  "oauth.deviceAuthorizations.passwordCompletions.create",
  "iam.runtime.retrieve",
  "iam.verificationPolicy.retrieve",
  "users.current.retrieve",
];
const forbiddenAppbaseTags = ["auth", "iam", "oauth", "openPlatform", "system"];
const forbiddenAppbaseSchemas = ["AppbaseApiResult", "AppbaseOperationCommand"];

function collectPathOperationIds(openapi) {
  const operationIds = [];
  for (const pathItem of Object.values(openapi.paths || {})) {
    for (const [method, operation] of Object.entries(pathItem || {})) {
      if (!["get", "put", "post", "patch", "delete", "head", "options", "trace"].includes(method)) {
        continue;
      }
      if (operation?.operationId) {
        operationIds.push(String(operation.operationId));
      }
    }
  }
  return operationIds;
}

test("sdkwork-drive-app-sdk uses sdkwork-v3 profile", () => {
  const source = readFileSync(path.join(sdkRoot, "bin/generate-sdk.mjs"), "utf8");
  assert.match(source, /--standard-profile/);
  assert.match(source, /sdkwork-v3/);
  assert.match(
    source,
    /defaultOpenapiPath:\s*"apis\/app-api\/drive\/drive-app-api\.openapi\.json"/,
  );
});

test("sdkwork-drive-app-sdk records family metadata outside generated output for every official language", () => {
  const manifest = JSON.parse(readFileSync(path.join(sdkRoot, "sdk-manifest.json"), "utf8"));
  assert.equal(manifest.sdkName, sdkName);
  assert.equal(manifest.apiPrefix, apiPrefix);
  assert.equal(manifest.standardProfile, "sdkwork-v3");
  assert.ok(
    manifest.ownerOnlyOperationCount >= 72,
    "family manifest should include the completed app drive operation surface",
  );
  assert.equal(manifest.sdkOwner, "sdkwork-drive");
  assert.equal(manifest.apiAuthority, "sdkwork-drive-app-api");
  assert.deepEqual(
    manifest.sdkDependencies?.map((dependency) => ({
      workspace: dependency.workspace,
      apiAuthority: dependency.apiAuthority,
      dependencyMode: dependency.dependencyMode,
      generatedTransportImportPolicy: dependency.generatedTransportImportPolicy,
    })),
    [
      {
        workspace: "sdkwork-iam-app-sdk",
        apiAuthority: "sdkwork-iam-app-api",
        dependencyMode: "consumer-sdk",
        generatedTransportImportPolicy: "forbidden",
      },
    ],
  );

  for (const language of languages) {
    const generatedOutput = `${sdkName}-${language}/generated/server-openapi`;
    assert.deepEqual(
      manifest.generatedPackages?.[language],
      {
        language,
        packageName: `${sdkName}-generated-${language}`,
        generatedOutput,
      },
      `${language} family manifest must record the generated package`,
    );
    assert.equal(
      existsSync(path.join(sdkRoot, generatedOutput, "sdk-manifest.json")),
      false,
      `${language} generated output must not carry SDK ownership manifest`,
    );

    const sourceOpenapi = JSON.parse(readFileSync(
      path.join(
        sdkRoot,
        `${sdkName}-${language}`,
        "generated/server-openapi/source-openapi.json",
      ),
      "utf8",
    ));
    const generatedOperationIds = collectPathOperationIds(sourceOpenapi);
    const generatedTags = (sourceOpenapi.tags || []).map((tag) => String(tag.name || ""));
    for (const operationId of requiredOperations) {
      assert(
        generatedOperationIds.includes(operationId),
        `${language} generated source OpenAPI should include ${operationId}`,
      );
    }
    for (const operationId of forbiddenAppbaseAppOperations) {
      assert(
        !generatedOperationIds.includes(operationId),
        `${language} generated source OpenAPI must consume appbase IAM operation ${operationId} through sdkDependencies`,
      );
    }
    for (const operationId of forbiddenStorageAdminOperations) {
      assert(
        !generatedOperationIds.includes(operationId),
        `${language} generated source OpenAPI must not expose storage administration operation ${operationId}`,
      );
    }
    for (const tagName of forbiddenAppbaseTags) {
      assert(
        !generatedTags.includes(tagName),
        `${language} generated source OpenAPI must not expose appbase tag ${tagName}`,
      );
    }
    for (const schemaName of forbiddenAppbaseSchemas) {
      assert(
        !sourceOpenapi.components?.schemas?.[schemaName],
        `${language} generated source OpenAPI must not expose appbase schema ${schemaName}`,
      );
    }
    for (const schemaName of forbiddenStorageAdminSchemas) {
      assert(
        !sourceOpenapi.components?.schemas?.[schemaName],
        `${language} generated source OpenAPI must not expose storage administration schema ${schemaName}`,
      );
    }
  }
});

test("sdkwork-drive-app-sdk composed TypeScript operations include completed drive operations", () => {
  const source = readFileSync(
    path.join(
      sdkRoot,
      "sdkwork-drive-app-sdk-typescript",
      "composed/operations.ts",
    ),
    "utf8",
  );
  for (const operationId of requiredOperations) {
    assert.match(source, new RegExp(`"${operationId.replace(".", "\\.")}"`));
  }
  for (const operationId of forbiddenAppbaseAppOperations) {
    assert.doesNotMatch(source, new RegExp(`"${operationId.replace(".", "\\.")}"`));
  }
  for (const operationId of forbiddenStorageAdminOperations) {
    assert.doesNotMatch(source, new RegExp(`"${operationId.replace(".", "\\.")}"`));
  }
});

test("sdkwork-drive-app-sdk language operation metadata includes shareLinks.claim", () => {
  const metadataSources = {
    python: path.join(
      sdkRoot,
      "sdkwork-drive-app-sdk-python",
      "generated/server-openapi/sdkwork_drive_generated/__init__.py",
    ),
    java: path.join(
      sdkRoot,
      "sdkwork-drive-app-sdk-java",
      "generated/server-openapi/src/main/java/com/sdkwork/generated/SdkMetadata.java",
    ),
    go: path.join(
      sdkRoot,
      "sdkwork-drive-app-sdk-go",
      "generated/server-openapi/client.go",
    ),
  };

  for (const [language, metadataPath] of Object.entries(metadataSources)) {
    const source = readFileSync(metadataPath, "utf8");
    assert.match(
      source,
      /"shareLinks\.claim"/,
      `${language} operation metadata should include shareLinks.claim`,
    );
  }
});

test("sdkwork-drive-app-sdk composed TypeScript exposes high-level uploader client", () => {
  const composedRoot = path.join(sdkRoot, "sdkwork-drive-app-sdk-typescript", "composed");
  const uploaderEntry = path.join(composedRoot, "uploader/index.ts");
  const uploaderClient = path.join(composedRoot, "uploader/uploaderClient.ts");
  const uploaderTypes = path.join(composedRoot, "uploader/types.ts");
  const uploadPlanner = path.join(composedRoot, "uploader/uploadPlanner.ts");
  const uploadStateStore = path.join(composedRoot, "uploader/uploadStateStore.ts");

  for (const filePath of [
    uploaderEntry,
    uploaderClient,
    uploaderTypes,
    uploadPlanner,
    uploadStateStore,
  ]) {
    assert.equal(existsSync(filePath), true, `${path.basename(filePath)} should exist`);
  }

  const clientSource = readFileSync(uploaderClient, "utf8");
  for (const methodName of [
    "upload(",
    "uploadVideo(",
    "uploadImage(",
    "uploadAudio(",
    "uploadDocument(",
    "uploadArchive(",
    "uploadText(",
    "uploadDataset(",
    "uploadAttachment(",
    "uploadAvatar(",
    "uploadThumbnail(",
  ]) {
    assert.match(clientSource, new RegExp(methodName.replace("(", "\\(")));
  }
  assert.match(clientSource, /uploader\.uploads\.create/);
  assert.match(clientSource, /shareToken:\s*normalized\.shareToken/);
  assert.match(clientSource, /uploadSessions\.parts\.update/);
  assert.match(clientSource, /uploader\.uploads\.parts\.update/);
  assert.match(clientSource, /uploadSessions\.complete/);

  const entrySource = readFileSync(uploaderEntry, "utf8");
  assert.match(entrySource, /createDriveUploaderClient/);
  assert.match(entrySource, /createInMemoryUploaderStateStore/);
});

test("sdkwork-drive-app-sdk generated TypeScript does not carry ownership metadata", () => {
  const source = readFileSync(
    path.join(
      sdkRoot,
      "sdkwork-drive-app-sdk-typescript",
      "generated/server-openapi/src/index.ts",
    ),
    "utf8",
  );
  assert.doesNotMatch(source, /sdkMetadata/);
  assert.doesNotMatch(source, /sdkDependencies/);
});

test("sdkwork-drive-app-sdk generated TypeScript exposes a real client surface", () => {
  const source = readFileSync(
    path.join(
      sdkRoot,
      "sdkwork-drive-app-sdk-typescript",
      "generated/server-openapi/src/index.ts",
    ),
    "utf8",
  );
  assert.match(source, /createClient/);
  assert.match(source, /export \* from ['"]\.\/api['"]/);
});

test("sdkwork-drive-app-sdk generated TypeScript exposes DriveNode usage context", () => {
  const source = readFileSync(
    path.join(
      sdkRoot,
      "sdkwork-drive-app-sdk-typescript",
      "generated/server-openapi/src/types/drive-node.ts",
    ),
    "utf8",
  );
  assert.match(source, /scene\?: string/);
  assert.match(source, /source\?: string/);
});

test("sdkwork-drive-app-sdk generated TypeScript exposes typed recent node metadata", () => {
  const driveApiSource = readFileSync(
    path.join(
      sdkRoot,
      "sdkwork-drive-app-sdk-typescript",
      "generated/server-openapi/src/api/drive.ts",
    ),
    "utf8",
  );
  const driveNodeSource = readFileSync(
    path.join(
      sdkRoot,
      "sdkwork-drive-app-sdk-typescript",
      "generated/server-openapi/src/types/drive-node.ts",
    ),
    "utf8",
  );
  const listDataSource = readFileSync(
    path.join(
      sdkRoot,
      "sdkwork-drive-app-sdk-typescript",
      "generated/server-openapi/src/types/drive-node-list-data.ts",
    ),
    "utf8",
  );

  assert.match(
    driveApiSource,
    /async list\(params\?: DriveRecentListParams\): Promise<DriveNodeListData>/,
  );
  assert.match(driveApiSource, /appApiPath\(`\/drive\/recent`\)/);
  assert.match(driveNodeSource, /createdAt: string/);
  assert.match(driveNodeSource, /updatedAt: string/);
  assert.match(listDataSource, /items: DriveNode\[\]/);
  assert.match(listDataSource, /incompletePage\?: boolean/);
});

test("sdkwork-drive-app-sdk generated TypeScript does not export appbase dependency models", () => {
  const typeIndex = readFileSync(
    path.join(
      sdkRoot,
      "sdkwork-drive-app-sdk-typescript",
      "generated/server-openapi/src/types/index.ts",
    ),
    "utf8",
  );
  for (const schemaName of forbiddenAppbaseSchemas) {
    assert.doesNotMatch(typeIndex, new RegExp(schemaName));
  }
  for (const schemaName of forbiddenStorageAdminSchemas) {
    assert.doesNotMatch(typeIndex, new RegExp(schemaName));
  }
});
