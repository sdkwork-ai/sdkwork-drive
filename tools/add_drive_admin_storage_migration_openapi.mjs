#!/usr/bin/env node
/**
 * Add the cross-provider storage migration operations to
 * `apis/backend-api/drive/drive-admin-storage-api.openapi.json`.
 *
 * Run once, after which the file is the source of truth and this script is
 * retained only as the record of how the block was shaped. Re-running it is
 * idempotent: every path and schema is replaced wholesale.
 *
 * Shape rules this script enforces, matching the rest of the file:
 * - single resource  -> 200 allOf [SdkWorkApiResponse, {data: {item: <Schema>}}]
 * - collection       -> 200 allOf [SdkWorkApiResponse, {data: {items: [...], pageInfo}}]
 * - failures         -> application/problem+json + application/json ProblemDetail
 * - every operation carries the five x-sdkwork-* extensions verbatim
 */
import { readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const apiPath = path.join(
  repoRoot,
  'apis/backend-api/drive/drive-admin-storage-api.openapi.json',
);

const api = JSON.parse(readFileSync(apiPath, 'utf8'));

const BASE = '/backend/v3/api/drive/storage';

/** The five extensions every operation in this file carries. */
const SDKWORK_EXTENSIONS = {
  'x-sdkwork-owner': 'sdkwork-drive',
  'x-sdkwork-api-authority': 'sdkwork-drive.admin.storage',
  'x-sdkwork-request-context': 'WebRequestContext',
  'x-sdkwork-api-surface': 'backend-api',
  'x-sdkwork-permission': 'drive.storage.admin',
};

const SECURITY = [{ AuthToken: [], AccessToken: [] }];

const PROBLEM_SCHEMA = { $ref: '#/components/schemas/ProblemDetail' };

function problemResponse(description) {
  return {
    description,
    content: {
      'application/problem+json': { schema: PROBLEM_SCHEMA },
      'application/json': { schema: PROBLEM_SCHEMA },
    },
  };
}

/** `200` wrapping a single typed resource under `data.item`. */
function itemResponse(schemaName) {
  return {
    description: 'OK',
    content: {
      'application/json': {
        schema: {
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
                    item: { $ref: `#/components/schemas/${schemaName}` },
                  },
                },
              },
            },
          ],
        },
      },
    },
  };
}

/** `200` wrapping a cursor page under `data.items` + `data.pageInfo`. */
function pageResponse(schemaName) {
  return {
    description: 'OK',
    content: {
      'application/json': {
        schema: {
          allOf: [
            { $ref: '#/components/schemas/SdkWorkApiResponse' },
            {
              type: 'object',
              required: ['data'],
              properties: {
                data: {
                  type: 'object',
                  additionalProperties: false,
                  required: ['items', 'pageInfo'],
                  properties: {
                    items: {
                      type: 'array',
                      items: { $ref: `#/components/schemas/${schemaName}` },
                    },
                    pageInfo: { $ref: '#/components/schemas/PageInfo' },
                  },
                },
              },
            },
          ],
        },
      },
    },
  };
}

function pathParam(name) {
  return { name, in: 'path', required: true, schema: { type: 'string' } };
}

function queryParam(name, schema) {
  return { name, in: 'query', required: false, schema };
}

function operation({ operationId, parameters = [], responses, requestBody }) {
  const op = {
    tags: ['storageAdmin'],
    operationId,
    ...(parameters.length ? { parameters } : {}),
    ...(requestBody ? { requestBody } : {}),
    responses,
    security: SECURITY,
    ...SDKWORK_EXTENSIONS,
  };
  return op;
}

function jsonBody(schemaRef, required = true) {
  return {
    required,
    content: {
      'application/json': {
        schema: { $ref: `#/components/schemas/${schemaRef}` },
      },
    },
  };
}

// ---------------------------------------------------------------------------
// Schemas
// ---------------------------------------------------------------------------

const MIGRATION_STATUSES = ['pending', 'running', 'succeeded', 'failed', 'cancelled'];
const MIGRATION_ITEM_STATUSES = ['pending', 'copied', 'failed'];

const int64String = (description) => ({
  type: 'string',
  format: 'int64',
  'x-sdkwork-int64-string': true,
  ...(description ? { description } : {}),
});

api.components.schemas.StorageMigration = {
  type: 'object',
  additionalProperties: false,
  required: [
    'id',
    'name',
    'sourceProviderId',
    'targetProviderId',
    'status',
    'applyBindingSwitch',
    'objectsTotal',
    'objectsCopied',
    'objectsFailed',
    'objectsOutstanding',
    'bytesCopied',
    'progressRatio',
    'createdBy',
    'updatedBy',
  ],
  properties: {
    id: { type: 'string' },
    name: { type: 'string' },
    sourceProviderId: { type: 'string' },
    targetProviderId: { type: 'string' },
    targetBucket: { type: ['string', 'null'] },
    status: { type: 'string', enum: MIGRATION_STATUSES },
    applyBindingSwitch: {
      type: 'boolean',
      description:
        'When true, every migrated object is re-pointed at the target provider once the run completes without failures. When false the bytes land on the target but the registry keeps reading from the source.',
    },
    objectsTotal: int64String('Objects queued when the run was seeded.'),
    objectsCopied: int64String('Objects copied and whose checksum was verified.'),
    objectsFailed: int64String('Objects that failed verification.'),
    objectsOutstanding: int64String('Objects still pending, derived from the queue.'),
    bytesCopied: int64String(),
    progressRatio: { type: 'number', format: 'double', minimum: 0, maximum: 1 },
    failureMessage: { type: ['string', 'null'] },
    createdBy: { type: 'string' },
    updatedBy: { type: 'string' },
  },
};

api.components.schemas.StorageMigrationItem = {
  type: 'object',
  additionalProperties: false,
  required: [
    'id',
    'storageObjectId',
    'sourceProviderId',
    'sourceBucket',
    'sourceObjectKey',
    'targetProviderId',
    'targetBucket',
    'targetObjectKey',
    'contentType',
    'contentLength',
    'checksumSha256Hex',
    'status',
  ],
  properties: {
    id: { type: 'string' },
    storageObjectId: { type: 'string' },
    sourceProviderId: { type: 'string' },
    sourceBucket: { type: 'string' },
    sourceObjectKey: { type: 'string' },
    targetProviderId: { type: 'string' },
    targetBucket: { type: 'string' },
    targetObjectKey: { type: 'string' },
    contentType: { type: 'string' },
    contentLength: int64String(),
    checksumSha256Hex: {
      type: 'string',
      description: 'SHA-256 recorded on the source registry row, in lowercase hex.',
    },
    status: { type: 'string', enum: MIGRATION_ITEM_STATUSES },
    verifiedChecksumSha256Hex: {
      type: ['string', 'null'],
      description: 'SHA-256 observed on the target after the copy, present only on success.',
    },
    failureMessage: { type: ['string', 'null'] },
  },
};

api.components.schemas.CreateStorageMigrationRequest = {
  type: 'object',
  additionalProperties: false,
  required: ['id', 'name', 'sourceProviderId', 'targetProviderId'],
  properties: {
    id: {
      type: 'string',
      description:
        'Caller-owned id. Supplying it makes a retried plan idempotent instead of opening a second run for the same intended migration.',
    },
    name: { type: 'string' },
    sourceProviderId: {
      type: 'string',
      description: 'Provider to drain. May be disabled; a deleted provider is rejected.',
    },
    targetProviderId: { type: 'string', description: 'Provider to copy onto. Must be active.' },
    targetBucket: {
      type: ['string', 'null'],
      description: "Target bucket. Defaults to the target provider's own bucket.",
    },
    applyBindingSwitch: {
      type: 'boolean',
      default: false,
      description:
        'Re-point objects and bindings at the target once every object is copied and verified.',
    },
  },
};

api.components.schemas.RunStorageMigrationRequest = {
  type: 'object',
  additionalProperties: false,
  properties: {
    batchSize: {
      type: ['integer', 'null'],
      format: 'int64',
      minimum: 1,
      description:
        'Objects to copy in this call. Clamped into the supported range rather than rejected, because it is a throughput knob and not a contract.',
    },
  },
};

api.components.schemas.StorageMigrationRunReport = {
  type: 'object',
  additionalProperties: false,
  required: ['migration', 'copiedThisBatch', 'failedThisBatch', 'completed'],
  properties: {
    migration: { $ref: '#/components/schemas/StorageMigration' },
    copiedThisBatch: int64String(),
    failedThisBatch: int64String(),
    completed: {
      type: 'boolean',
      description: 'True once the run reached `succeeded` on this call.',
    },
  },
};

// ---------------------------------------------------------------------------
// Paths
// ---------------------------------------------------------------------------

const pagedQuery = (extra = []) => [
  ...extra,
  queryParam('pageSize', { type: 'integer', minimum: 1, maximum: 200 }),
  queryParam('pageToken', { type: 'string' }),
];

api.paths[`${BASE}/migrations`] = {
  get: operation({
    operationId: 'storageMigrations.list',
    parameters: pagedQuery([queryParam('status', { type: 'string' })]),
    responses: {
      200: pageResponse('StorageMigration'),
      400: problemResponse('Bad Request'),
      500: problemResponse('Internal Server Error'),
    },
  }),
  post: operation({
    operationId: 'storageMigrations.create',
    requestBody: jsonBody('CreateStorageMigrationRequest'),
    responses: {
      200: itemResponse('StorageMigration'),
      400: problemResponse('Bad Request'),
      403: problemResponse('Forbidden'),
      404: problemResponse('Not Found'),
      409: problemResponse('Conflict'),
      500: problemResponse('Internal Server Error'),
    },
  }),
};

api.paths[`${BASE}/migrations/{migrationId}`] = {
  get: operation({
    operationId: 'storageMigrations.retrieve',
    parameters: [pathParam('migrationId')],
    responses: {
      200: itemResponse('StorageMigration'),
      404: problemResponse('Not Found'),
      500: problemResponse('Internal Server Error'),
    },
  }),
};

api.paths[`${BASE}/migrations/{migrationId}/run`] = {
  post: operation({
    operationId: 'storageMigrations.run',
    parameters: [pathParam('migrationId')],
    requestBody: jsonBody('RunStorageMigrationRequest', false),
    responses: {
      200: itemResponse('StorageMigrationRunReport'),
      400: problemResponse('Bad Request'),
      404: problemResponse('Not Found'),
      409: problemResponse('Conflict'),
      500: problemResponse('Internal Server Error'),
    },
  }),
};

api.paths[`${BASE}/migrations/{migrationId}/items`] = {
  get: operation({
    operationId: 'storageMigrations.items.list',
    parameters: pagedQuery([pathParam('migrationId'), queryParam('status', { type: 'string' })]),
    responses: {
      200: pageResponse('StorageMigrationItem'),
      400: problemResponse('Bad Request'),
      404: problemResponse('Not Found'),
      500: problemResponse('Internal Server Error'),
    },
  }),
};

api.paths[`${BASE}/migrations/{migrationId}/cancel`] = {
  post: operation({
    operationId: 'storageMigrations.cancel',
    parameters: [pathParam('migrationId')],
    // No request body: the operator is a request-context field and must not be
    // client-writable, so accepting one would let a caller name someone else in
    // the audit trail.
    responses: {
      200: itemResponse('StorageMigration'),
      404: problemResponse('Not Found'),
      409: problemResponse('Conflict'),
      500: problemResponse('Internal Server Error'),
    },
  }),
};

// Paths are left in insertion order. The manifest generator sorts by
// (path, method) itself, so re-ordering here would only produce diff churn in a
// hand-maintained file.

writeFileSync(apiPath, `${JSON.stringify(api, null, 2)}\n`, 'utf8');
console.log(`added ${Object.keys(api.paths).length} paths -> ${path.relative(repoRoot, apiPath)}`);
