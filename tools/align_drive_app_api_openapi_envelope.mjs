#!/usr/bin/env node
/**
 * Align drive-app-api.openapi.json success responses with SdkWorkApiResponse envelopes.
 */
import { readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const openApiPath = path.join(repoRoot, 'apis/app-api/drive/drive-app-api.openapi.json');
const doc = JSON.parse(readFileSync(openApiPath, 'utf8'));

function schemaRefName(schema) {
  if (!schema || typeof schema !== 'object' || typeof schema.$ref !== 'string') {
    return undefined;
  }
  return schema.$ref.match(/#\/components\/schemas\/(.+)$/)?.[1];
}

function schemaExtendsSdkWorkApiResponse(schemaName, visited = new Set()) {
  if (!schemaName || schemaName === 'SdkWorkApiResponse' || visited.has(schemaName)) {
    return schemaName === 'SdkWorkApiResponse';
  }
  visited.add(schemaName);
  const schema = doc.components?.schemas?.[schemaName];
  if (!schema || typeof schema !== 'object') {
    return false;
  }
  if (Array.isArray(schema.required) && schema.required.includes('code') && schema.required.includes('traceId')) {
    return true;
  }
  return Array.isArray(schema.allOf) && schema.allOf.some((part) => {
    const partName = schemaRefName(part);
    return partName === 'SdkWorkApiResponse'
      || schemaExtendsSdkWorkApiResponse(partName, visited);
  });
}

function collapseDuplicateEnvelopeSchema(schema) {
  if (!schema || typeof schema !== 'object' || !Array.isArray(schema.allOf)) {
    return undefined;
  }
  const hasBaseEnvelope = schema.allOf.some(
    (part) => schemaRefName(part) === 'SdkWorkApiResponse',
  );
  if (!hasBaseEnvelope) {
    return undefined;
  }
  for (const part of schema.allOf) {
    const schemaName = schemaRefName(part);
    if (schemaName && schemaName !== 'SdkWorkApiResponse' && schemaExtendsSdkWorkApiResponse(schemaName)) {
      return { $ref: `#/components/schemas/${schemaName}` };
    }
  }
  return undefined;
}

function resourceHttpResponse(name, itemSchema) {
  return {
    allOf: [
      { $ref: '#/components/schemas/SdkWorkApiResponse' },
      {
        type: 'object',
        required: ['data'],
        properties: {
          data: {
            type: 'object',
            required: ['item'],
            properties: {
              item: { $ref: `#/components/schemas/${itemSchema}` },
            },
          },
        },
      },
    ],
  };
}

function commandHttpResponse(name, dataSchema) {
  return {
    allOf: [
      { $ref: '#/components/schemas/SdkWorkApiResponse' },
      {
        type: 'object',
        required: ['data'],
        properties: {
          data: { $ref: `#/components/schemas/${dataSchema}` },
        },
      },
    ],
  };
}

function listDataSchema(itemSchema, extraDataProperties = {}) {
  return {
    type: 'object',
    additionalProperties: false,
    required: ['items', 'pageInfo'],
    properties: {
      items: {
        type: 'array',
        items: { $ref: `#/components/schemas/${itemSchema}` },
      },
      pageInfo: { $ref: '#/components/schemas/PageInfo' },
      ...extraDataProperties,
    },
  };
}

function listHttpResponse(dataSchema) {
  return {
    allOf: [
      { $ref: '#/components/schemas/SdkWorkApiResponse' },
      {
        type: 'object',
        required: ['data'],
        properties: {
          data: { $ref: `#/components/schemas/${dataSchema}` },
        },
      },
    ],
  };
}

/** legacy bare list schema -> enveloped wrapper */
const legacyListSchemaMap = {
  NodeListResponse: {
    wrapperName: 'DriveNodeListHttpResponse',
    dataSchemaName: 'DriveNodeListData',
    itemSchema: 'DriveNode',
    extraDataProperties: {
      incompletePage: {
        type: 'boolean',
        description: 'True when ACL pagination scan budget was exhausted before the requested page could be filled.',
      },
    },
  },
  ChangeListResponse: {
    wrapperName: 'ChangeListHttpResponse',
    dataSchemaName: 'ChangeListData',
    itemSchema: 'Change',
  },
  VersionListResponse: {
    wrapperName: 'FileVersionListHttpResponse',
    dataSchemaName: 'FileVersionListData',
    itemSchema: 'FileVersion',
  },
  DriveWatchChannelListResponse: {
    wrapperName: 'DriveWatchChannelListHttpResponse',
    dataSchemaName: 'DriveWatchChannelListData',
    itemSchema: 'DriveWatchChannel',
  },
  AssetPage: {
    wrapperName: 'AssetListHttpResponse',
    dataSchemaName: 'AssetListData',
    itemSchema: 'AssetItem',
  },
  AssetCollectionPage: {
    wrapperName: 'AssetCollectionListHttpResponse',
    dataSchemaName: 'AssetCollectionListData',
    itemSchema: 'AssetCollection',
  },
};

for (const binding of Object.values(legacyListSchemaMap)) {
  doc.components.schemas[binding.dataSchemaName] = listDataSchema(
    binding.itemSchema,
    binding.extraDataProperties,
  );
  doc.components.schemas[binding.wrapperName] = listHttpResponse(binding.dataSchemaName);
}

const noContentOperationIds = new Set([
  'versions.delete',
  'assetCollectionItems.delete',
  'assetRelations.delete',
]);

/** operationId -> { [httpStatus]: { wrapperName, kind: 'resource'|'command', schema } } */
const operationBindings = {
  'quotas.retrieve': { 200: { wrapperName: 'QuotaSummaryHttpResponse', kind: 'resource', schema: 'QuotaSummary' } },
  'downloadUrls.create': {
    201: { wrapperName: 'CreateDownloadUrlHttpResponse', kind: 'command', schema: 'CreateDownloadUrlResponse' },
  },
  'nodes.downloadUrls.retrieve': {
    200: { wrapperName: 'CreateDownloadUrlHttpResponse', kind: 'command', schema: 'CreateDownloadUrlResponse' },
  },
  'downloadGrants.create': {
    201: { wrapperName: 'CreateDownloadUrlHttpResponse', kind: 'command', schema: 'CreateDownloadUrlResponse' },
  },
  'uploadSessions.create': {
    201: { wrapperName: 'DriveUploadSessionHttpResponse', kind: 'resource', schema: 'DriveUploadSession' },
  },
  'uploadSessions.retrieve': {
    200: { wrapperName: 'DriveUploadSessionHttpResponse', kind: 'resource', schema: 'DriveUploadSession' },
  },
  'uploadSessions.complete': {
    200: { wrapperName: 'DriveUploadSessionHttpResponse', kind: 'resource', schema: 'DriveUploadSession' },
  },
  'uploadSessions.abort': {
    200: { wrapperName: 'DriveUploadSessionHttpResponse', kind: 'resource', schema: 'DriveUploadSession' },
  },
  'uploadSessions.parts.update': {
    200: { wrapperName: 'PresignedUploadPartHttpResponse', kind: 'command', schema: 'PresignedUploadPart' },
  },
  'uploader.uploads.create': {
    201: {
      wrapperName: 'PrepareUploaderUploadHttpResponse',
      kind: 'command',
      schema: 'PrepareUploaderUploadResponse',
    },
  },
  'uploader.uploads.parts.update': {
    200: { wrapperName: 'UploaderUploadPartHttpResponse', kind: 'resource', schema: 'UploaderUploadPart' },
  },
  'versions.retrieve': { 200: { wrapperName: 'FileVersionHttpResponse', kind: 'resource', schema: 'FileVersion' } },
  'favorites.update': {
    200: { wrapperName: 'FavoriteNodeHttpResponse', kind: 'command', schema: 'FavoriteNodeResponse' },
  },
  'favorites.delete': {
    200: { wrapperName: 'FavoriteNodeHttpResponse', kind: 'command', schema: 'FavoriteNodeResponse' },
  },
  'nodes.path.retrieve': { 200: { wrapperName: 'NodePathHttpResponse', kind: 'command', schema: 'NodePathResponse' } },
  'nodes.capabilities.list': {
    200: { wrapperName: 'NodeCapabilitiesHttpResponse', kind: 'resource', schema: 'NodeCapabilitiesResponse' },
  },
  'changes.startPageToken.retrieve': {
    200: { wrapperName: 'StartPageTokenHttpResponse', kind: 'command', schema: 'StartPageTokenResponse' },
  },
  'changes.watch': {
    201: { wrapperName: 'DriveWatchChannelHttpResponse', kind: 'resource', schema: 'DriveWatchChannel' },
  },
  'nodes.watch': {
    201: { wrapperName: 'DriveWatchChannelHttpResponse', kind: 'resource', schema: 'DriveWatchChannel' },
  },
  'watchChannels.retrieve': {
    200: { wrapperName: 'DriveWatchChannelHttpResponse', kind: 'resource', schema: 'DriveWatchChannel' },
  },
  'watchChannels.stop': {
    200: { wrapperName: 'StopWatchChannelHttpResponse', kind: 'command', schema: 'StopWatchChannelResponse' },
  },
  'downloadPackages.create': {
    201: { wrapperName: 'DownloadPackageHttpResponse', kind: 'command', schema: 'DownloadPackageResponse' },
  },
  'downloadPackages.downloadUrls.retrieve': {
    200: { wrapperName: 'DownloadPackageHttpResponse', kind: 'command', schema: 'DownloadPackageResponse' },
  },
  'assets.create': { 201: { wrapperName: 'AssetItemHttpResponse', kind: 'resource', schema: 'AssetItem' } },
  'assets.retrieve': { 200: { wrapperName: 'AssetItemHttpResponse', kind: 'resource', schema: 'AssetItem' } },
  'assets.update': { 200: { wrapperName: 'AssetItemHttpResponse', kind: 'resource', schema: 'AssetItem' } },
  'assets.archive': { 200: { wrapperName: 'AssetItemHttpResponse', kind: 'resource', schema: 'AssetItem' } },
  'assets.restore': { 200: { wrapperName: 'AssetItemHttpResponse', kind: 'resource', schema: 'AssetItem' } },
  'assetCollections.create': {
    201: { wrapperName: 'AssetCollectionHttpResponse', kind: 'resource', schema: 'AssetCollection' },
  },
  'assetCollectionItems.create': {
    201: { wrapperName: 'AssetCollectionItemHttpResponse', kind: 'resource', schema: 'AssetCollectionItem' },
  },
  'assetRelations.create': {
    201: { wrapperName: 'AssetRelationHttpResponse', kind: 'resource', schema: 'AssetRelation' },
  },
};

for (const bindings of Object.values(operationBindings)) {
  for (const binding of Object.values(bindings)) {
    const { wrapperName, kind, schema } = binding;
    if (doc.components.schemas[wrapperName]) {
      continue;
    }
    doc.components.schemas[wrapperName] =
      kind === 'resource' ? resourceHttpResponse(wrapperName, schema) : commandHttpResponse(wrapperName, schema);
  }
}

let updatedOperations = 0;
for (const pathItem of Object.values(doc.paths ?? {})) {
  for (const operation of Object.values(pathItem)) {
    if (!operation?.operationId) {
      continue;
    }

    for (const status of ['200', '201', '202']) {
      const response = operation.responses?.[status];
      const mediaType = response?.content?.['application/json'];
      const collapsedSchema = collapseDuplicateEnvelopeSchema(mediaType?.schema);
      if (collapsedSchema) {
        mediaType.schema = collapsedSchema;
        updatedOperations += 1;
      }
    }

    if (noContentOperationIds.has(operation.operationId)) {
      operation.responses = operation.responses ?? {};
      if (operation.responses['200']) {
        delete operation.responses['200'];
        updatedOperations += 1;
      }
      const response = operation.responses['204'] ?? { description: 'No Content' };
      if (response.content) {
        delete response.content;
        updatedOperations += 1;
      }
      operation.responses['204'] = response;
      continue;
    }

    const bindings = operationBindings[operation.operationId];
    if (bindings) {
      for (const [status, binding] of Object.entries(bindings)) {
        const response = operation.responses?.[status];
        const jsonSchema = response?.content?.['application/json']?.schema;
        if (!jsonSchema?.$ref) {
          continue;
        }
        const targetRef = `#/components/schemas/${binding.wrapperName}`;
        if (jsonSchema.$ref === targetRef) {
          continue;
        }
        jsonSchema.$ref = targetRef;
        updatedOperations += 1;
      }
    }

    for (const status of ['200', '201']) {
      const response = operation.responses?.[status];
      const jsonSchema = response?.content?.['application/json']?.schema;
      if (!jsonSchema?.$ref) {
        continue;
      }
      const legacySchema = jsonSchema.$ref.match(/#\/components\/schemas\/(.+)$/)?.[1];
      const listBinding = legacySchema ? legacyListSchemaMap[legacySchema] : undefined;
      if (!listBinding) {
        continue;
      }
      const targetRef = `#/components/schemas/${listBinding.wrapperName}`;
      if (jsonSchema.$ref === targetRef) {
        continue;
      }
      jsonSchema.$ref = targetRef;
      updatedOperations += 1;
    }
  }
}

for (const schemaName of [
  'ArchiveEntryListResponse',
  'AssetCollectionPage',
  'AssetPage',
  'ChangeListResponse',
  'CommentListResponse',
  'CommentReplyListResponse',
  'DeleteAssetCollectionItemHttpResponse',
  'DeleteAssetCollectionItemResponse',
  'DeleteAssetRelationHttpResponse',
  'DeleteAssetRelationResponse',
  'DeleteNodePropertyHttpResponse',
  'DeleteNodePropertyResponse',
  'DeleteNodeResponse',
  'DeleteSpaceHttpResponse',
  'DeleteSpaceResponse',
  'DeleteVersionHttpResponse',
  'DeleteVersionResponse',
  'DeletedCommandHttpResponse',
  'DriveWatchChannelListResponse',
  'EffectivePermissionListResponse',
  'ListSpacesResponse',
  'NodeLabelListResponse',
  'NodeListResponse',
  'NodePropertyListResponse',
  'PermissionListResponse',
  'RemoveNodeLabelResponse',
  'RevokeShareLinkResponse',
  'SdkWorkListResponse',
  'ShareLinkListResponse',
  'VersionListResponse',
]) {
  if (doc.components.schemas[schemaName]) {
    delete doc.components.schemas[schemaName];
    updatedOperations += 1;
  }
}

for (const pathItem of Object.values(doc.paths ?? {})) {
  for (const operation of Object.values(pathItem)) {
    if (!operation?.operationId) {
      continue;
    }
    for (const status of ['200', '201', '202']) {
      const schema = operation.responses?.[status]?.content?.['application/json']?.schema;
      if (collapseDuplicateEnvelopeSchema(schema)) {
        throw new Error(
          `${operation.operationId} ${status} still wraps an enveloped response inside SdkWorkApiResponse`,
        );
      }
    }
  }
}

writeFileSync(openApiPath, `${JSON.stringify(doc, null, 2)}\n`, 'utf8');
console.log(`align_drive_app_api_openapi_envelope: updated ${updatedOperations} operation response refs`);
