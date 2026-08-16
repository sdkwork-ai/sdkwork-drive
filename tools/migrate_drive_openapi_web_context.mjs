#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const httpMethods = new Set(['get', 'post', 'put', 'patch', 'delete', 'head', 'options']);

const openapiFiles = [
  { file: 'apis/app-api/drive/drive-app-api.openapi.json', surface: 'app-api' },
  { file: 'apis/drive-app-api.openapi.json', surface: 'app-api' },
  { file: 'apis/backend-api/drive/drive-backend-api.openapi.json', surface: 'backend-api' },
  { file: 'apis/drive-backend-api.openapi.json', surface: 'backend-api' },
  { file: 'apis/open-api/drive/drive-open-api.openapi.json', surface: 'open-api' },
  { file: 'apis/drive-open-api.openapi.json', surface: 'open-api' },
  { file: 'apis/backend-api/drive/drive-admin-storage-api.openapi.json', surface: 'backend-api' },
  { file: 'apis/drive-admin-storage-api.openapi.json', surface: 'backend-api' },
];

function surfaceForPath(routePath, fallback) {
  if (routePath.startsWith('/app/v3/api')) {
    return 'app-api';
  }
  if (routePath.startsWith('/backend/v3/api')) {
    return 'backend-api';
  }
  if (routePath.startsWith('/open/v3/api')) {
    return 'open-api';
  }
  if (routePath.startsWith('/admin/v3/api') || routePath.startsWith('/backend/v3/api/drive/storage')) {
    return 'backend-api';
  }
  return fallback;
}

for (const { file, surface: defaultSurface } of openapiFiles) {
  const absolutePath = path.join(repoRoot, file);
  const document = JSON.parse(fs.readFileSync(absolutePath, 'utf8'));
  let updatedOperations = 0;

  for (const [routePath, pathItem] of Object.entries(document.paths ?? {})) {
    if (!pathItem || typeof pathItem !== 'object') {
      continue;
    }
    const surface = surfaceForPath(routePath, defaultSurface);
    for (const [method, operation] of Object.entries(pathItem)) {
      if (!httpMethods.has(method) || !operation || typeof operation !== 'object') {
        continue;
      }
      if (operation['x-sdkwork-request-context'] === 'AppRequestContext') {
        operation['x-sdkwork-request-context'] = 'WebRequestContext';
      }
      if (!operation['x-sdkwork-request-context']) {
        operation['x-sdkwork-request-context'] = 'WebRequestContext';
      }
      if (!operation['x-sdkwork-api-surface']) {
        operation['x-sdkwork-api-surface'] = surface;
      }
      const isPublic = Array.isArray(operation.security) && operation.security.length === 0;
      if (isPublic && !operation['x-sdkwork-auth-mode']) {
        operation['x-sdkwork-auth-mode'] = 'anonymous';
      }
      updatedOperations += 1;
    }
  }

  fs.writeFileSync(absolutePath, `${JSON.stringify(document, null, 2)}\n`);
  process.stdout.write(`${file}: updated ${updatedOperations} operations\n`);
}
