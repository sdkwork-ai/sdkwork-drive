#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const httpMethods = new Set(['get', 'post', 'put', 'patch', 'delete', 'head', 'options']);

const openapiFiles = [
  'apis/app-api/drive/drive-app-api.openapi.json',
  'apis/drive-app-api.openapi.json',
  'apis/backend-api/drive/drive-backend-api.openapi.json',
  'apis/drive-backend-api.openapi.json',
  'apis/backend-api/drive/drive-admin-storage-api.openapi.json',
  'apis/drive-admin-storage-api.openapi.json',
  'apis/open-api/drive/drive-open-api.openapi.json',
  'apis/drive-open-api.openapi.json',
];

function stripClientTenantId(document) {
  let removedQueryParams = 0;
  let removedRequestFields = 0;

  for (const pathItem of Object.values(document.paths ?? {})) {
    if (!pathItem || typeof pathItem !== 'object') {
      continue;
    }
    for (const method of httpMethods) {
      const operation = pathItem[method];
      if (!operation || !Array.isArray(operation.parameters)) {
        continue;
      }
      const before = operation.parameters.length;
      operation.parameters = operation.parameters.filter(
        (parameter) => !(parameter?.in === 'query' && parameter?.name === 'tenantId'),
      );
      if (operation.parameters.length !== before) {
        removedQueryParams += 1;
      }
    }
  }

  for (const [schemaName, schema] of Object.entries(document.components?.schemas ?? {})) {
    if (!schemaName.endsWith('Request') || !schema?.properties?.tenantId) {
      continue;
    }
    delete schema.properties.tenantId;
    if (Array.isArray(schema.required)) {
      schema.required = schema.required.filter((field) => field !== 'tenantId');
      if (schema.required.length === 0) {
        delete schema.required;
      }
    }
    removedRequestFields += 1;
  }

  return { removedQueryParams, removedRequestFields };
}

for (const relativeFile of openapiFiles) {
  const absolutePath = path.join(repoRoot, relativeFile);
  const document = JSON.parse(fs.readFileSync(absolutePath, 'utf8'));
  const summary = stripClientTenantId(document);
  fs.writeFileSync(absolutePath, `${JSON.stringify(document, null, 2)}\n`);
  process.stdout.write(
    `${relativeFile}: removed ${summary.removedQueryParams} tenantId query params and ${summary.removedRequestFields} request tenantId fields\n`,
  );
}
