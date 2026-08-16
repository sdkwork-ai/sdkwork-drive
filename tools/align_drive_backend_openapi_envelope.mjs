#!/usr/bin/env node
/**
 * Align drive-backend-api.openapi.json list success responses with SdkWorkApiResponse envelopes.
 */
import { readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const backendPath = path.join(
  repoRoot,
  'apis/backend-api/drive/drive-backend-api.openapi.json',
);
const adminPath = path.join(
  repoRoot,
  'apis/backend-api/drive/drive-admin-storage-api.openapi.json',
);

const backend = JSON.parse(readFileSync(backendPath, 'utf8'));
const admin = JSON.parse(readFileSync(adminPath, 'utf8'));

const envelopeSchemas = [
  'SdkWorkApiResponse',
  'SdkWorkResourceData',
  'SdkWorkPageData',
  'SdkWorkCommandData',
  'PageInfo',
  'SdkWorkResourceResponse',
  'SdkWorkCommandResponse',
];

for (const schemaName of envelopeSchemas) {
  if (admin.components?.schemas?.[schemaName]) {
    backend.components.schemas[schemaName] = admin.components.schemas[schemaName];
  }
}

const listResponseMap = {
  AuditEventPage: { itemSchema: 'AuditEvent', wrapperName: 'AuditEventListHttpResponse' },
  LabelListResponse: { itemSchema: 'DriveLabel', wrapperName: 'DriveLabelListHttpResponse' },
  MaintenanceJobPage: { itemSchema: 'MaintenanceJob', wrapperName: 'MaintenanceJobListHttpResponse' },
  ListSpacesResponse: { itemSchema: 'DriveSpace', wrapperName: 'DriveSpaceListHttpResponse' },
  DownloadPackagePage: { itemSchema: 'DownloadPackage', wrapperName: 'DownloadPackageListHttpResponse' },
};

for (const { itemSchema, wrapperName } of Object.values(listResponseMap)) {
  backend.components.schemas[wrapperName] = {
    allOf: [
      { $ref: '#/components/schemas/SdkWorkApiResponse' },
      {
        type: 'object',
        required: ['data'],
        properties: {
          data: {
            type: 'object',
            required: ['items', 'pageInfo'],
            properties: {
              items: {
                type: 'array',
                items: { $ref: `#/components/schemas/${itemSchema}` },
              },
              pageInfo: { $ref: '#/components/schemas/PageInfo' },
            },
          },
        },
      },
    ],
  };
}

const legacyWrapperMap = Object.fromEntries(
  Object.entries(listResponseMap).map(([legacyPageSchema, { wrapperName }]) => [
    `${legacyPageSchema}SdkWorkEnvelope`,
    wrapperName,
  ]),
);

function replaceListResponse(schemaRef) {
  if (!schemaRef || typeof schemaRef !== 'object') {
    return schemaRef;
  }
  const legacy = schemaRef.$ref?.match(/#\/components\/schemas\/(.+)$/)?.[1];
  const wrapperName = legacy
    ? listResponseMap[legacy]?.wrapperName ?? legacyWrapperMap[legacy]
    : undefined;
  if (wrapperName) {
    return { $ref: `#/components/schemas/${wrapperName}` };
  }
  if (Array.isArray(schemaRef.allOf)) {
    schemaRef.allOf = schemaRef.allOf.map((item) => replaceListResponse(item));
  }
  return schemaRef;
}

for (const pathItem of Object.values(backend.paths ?? {})) {
  for (const operation of Object.values(pathItem ?? {})) {
    const success = operation?.responses?.['200']?.content?.['application/json']?.schema;
    if (success) {
      operation.responses['200'].content['application/json'].schema = replaceListResponse(success);
    }
    const created = operation?.responses?.['201']?.content?.['application/json']?.schema;
    if (created?.$ref === '#/components/schemas/DriveLabel') {
      operation.responses['201'].content['application/json'].schema = {
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
                  item: { $ref: '#/components/schemas/DriveLabel' },
                },
              },
            },
          },
        ],
      };
    }
  }
}

for (const schemaName of [
  'AuditEventPage',
  'AuditEventPageSdkWorkEnvelope',
  'DeleteLabelResponse',
  'DownloadPackagePage',
  'DownloadPackagePageSdkWorkEnvelope',
  'DriveSpaceListResponse',
  'LabelListResponse',
  'LabelListResponseSdkWorkEnvelope',
  'ListSpacesResponse',
  'ListSpacesResponseSdkWorkEnvelope',
  'MaintenanceJobPage',
  'MaintenanceJobPageSdkWorkEnvelope',
  'SdkWorkListResponse',
]) {
  delete backend.components.schemas[schemaName];
}

for (const schemaName of [
  'DeleteStorageProviderBindingResponse',
  'DeleteStorageProviderResponse',
  'ListStorageProvidersResponse',
  'ProviderBucketList',
  'ProviderObjectList',
  'SdkWorkListResponse',
  'StorageProviderBindingListResponse',
]) {
  delete admin.components.schemas[schemaName];
}

writeFileSync(backendPath, `${JSON.stringify(backend, null, 2)}\n`, 'utf8');
writeFileSync(adminPath, `${JSON.stringify(admin, null, 2)}\n`, 'utf8');
console.log('aligned drive-backend-api.openapi.json envelope schemas');
