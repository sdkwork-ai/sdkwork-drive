#!/usr/bin/env node
/**
 * Staging admin smoke for docs/guides/operator/pre-launch-checklist.md (Admin Operations Smoke).
 *
 * Requires dual-token IAM against a staging/pre-production backend. Skips with exit 0 when
 * SDKWORK_DRIVE_STAGING_BACKEND_BASE_URL is unset (local CI without secrets).
 *
 * Env:
 *   SDKWORK_DRIVE_STAGING_BACKEND_BASE_URL — e.g. https://api-staging.example.com/backend/v3/api/drive
 *   SDKWORK_DRIVE_STAGING_AUTH_TOKEN — organization-scoped auth JWT (drive.storage.admin or broader)
 *   SDKWORK_DRIVE_STAGING_ACCESS_TOKEN — matching access JWT
 *   SDKWORK_DRIVE_STAGING_AUDIT_AUTH_TOKEN / _ACCESS_TOKEN — optional RBAC audit-only pair
 *   SDKWORK_DRIVE_STAGING_QUOTA_AUTH_TOKEN / _ACCESS_TOKEN — optional RBAC quota-only pair
 *   SDKWORK_DRIVE_STAGING_OPERATOR_ID — operator id for maintenance sweeps (default smoke-operator)
 */

const DEFAULT_OPERATOR_ID = 'smoke-operator';

function env(name) {
  const value = process.env[name]?.trim();
  return value ? value : undefined;
}

function normalizeBaseUrl(raw) {
  return raw.replace(/\/+$/, '');
}

async function readJson(response) {
  const contentType = response.headers.get('content-type') ?? '';
  const text = await response.text();
  if (contentType.includes('application/problem+json')) {
    let problem;
    try {
      problem = JSON.parse(text);
    } catch {
      problem = { detail: text };
    }
    const code = problem.code ?? response.status;
    throw new Error(`HTTP ${response.status} problem ${code}: ${problem.detail ?? problem.title ?? text}`);
  }
  try {
    return JSON.parse(text);
  } catch {
    throw new Error(`expected JSON body, got: ${text.slice(0, 240)}`);
  }
}

function assertSdkWorkEnvelope(payload, label) {
  if (!payload || typeof payload !== 'object') {
    throw new Error(`${label}: response is not an object`);
  }
  if (payload.code !== 0) {
    throw new Error(`${label}: expected code 0, got ${String(payload.code)}`);
  }
  if (!('data' in payload)) {
    throw new Error(`${label}: missing data envelope`);
  }
  if (typeof payload.traceId !== 'string' || payload.traceId.length === 0) {
    throw new Error(`${label}: missing traceId`);
  }
  return payload.data;
}

async function backendRequest(baseUrl, path, { authToken, accessToken, method = 'GET', body } = {}) {
  const url = `${normalizeBaseUrl(baseUrl)}${path.startsWith('/') ? path : `/${path}`}`;
  const headers = {
    authorization: `Bearer ${authToken}`,
    'access-token': accessToken,
    accept: 'application/json',
  };
  const init = { method, headers };
  if (body !== undefined) {
    headers['content-type'] = 'application/json';
    init.body = JSON.stringify(body);
  }
  const response = await fetch(url, init);
  return { response, payload: await readJson(response) };
}

async function expectSuccess(baseUrl, path, tokens, options = {}) {
  const { response, payload } = await backendRequest(baseUrl, path, {
    authToken: tokens.auth,
    accessToken: tokens.access,
    ...options,
  });
  if (!response.ok) {
    throw new Error(`${path}: expected 2xx, got ${response.status}`);
  }
  return assertSdkWorkEnvelope(payload, path);
}

async function expectForbidden(baseUrl, path, tokens, options = {}) {
  const { response } = await backendRequest(baseUrl, path, {
    authToken: tokens.auth,
    accessToken: tokens.access,
    ...options,
  });
  if (response.status !== 403) {
    throw new Error(`${path}: expected 403 for RBAC denial, got ${response.status}`);
  }
}

async function runAdminSurfaceSmoke(baseUrl, tokens, operatorId) {
  await expectSuccess(baseUrl, '/audit_events?page_size=20', tokens);
  await expectSuccess(baseUrl, '/quotas', tokens);
  await expectSuccess(baseUrl, '/labels?page_size=20', tokens);
  await expectSuccess(baseUrl, '/spaces?page_size=20', tokens);
  await expectSuccess(baseUrl, '/download_packages?page_size=20', tokens);
  const sweep = await expectSuccess(baseUrl, '/maintenance/object_sweep', tokens, {
    method: 'POST',
    body: {
      dryRun: true,
      limit: 20,
      operatorId,
    },
  });
  if (!sweep || typeof sweep !== 'object') {
    throw new Error('maintenance/object_sweep: missing data payload');
  }
  await expectSuccess(baseUrl, '/maintenance/jobs?page_size=20', tokens);
}

async function runRbacSmoke(baseUrl, auditTokens, quotaTokens) {
  if (auditTokens) {
    await expectSuccess(baseUrl, '/audit_events?page_size=20', auditTokens);
    await expectForbidden(baseUrl, '/quotas', auditTokens);
    await expectForbidden(baseUrl, '/labels?page_size=20', auditTokens);
  }
  if (quotaTokens) {
    await expectSuccess(baseUrl, '/quotas', quotaTokens);
    await expectForbidden(baseUrl, '/audit_events?page_size=20', quotaTokens);
  }
}

async function main() {
  const baseUrl = env('SDKWORK_DRIVE_STAGING_BACKEND_BASE_URL');
  if (!baseUrl) {
    console.log('[staging-admin-smoke] skip: SDKWORK_DRIVE_STAGING_BACKEND_BASE_URL is not set');
    process.exit(0);
  }

  const auth = env('SDKWORK_DRIVE_STAGING_AUTH_TOKEN');
  const access = env('SDKWORK_DRIVE_STAGING_ACCESS_TOKEN');
  if (!auth || !access) {
    console.error('[staging-admin-smoke] error: SDKWORK_DRIVE_STAGING_AUTH_TOKEN and SDKWORK_DRIVE_STAGING_ACCESS_TOKEN are required');
    process.exit(1);
  }

  const operatorId = env('SDKWORK_DRIVE_STAGING_OPERATOR_ID') ?? DEFAULT_OPERATOR_ID;
  const adminTokens = { auth, access };

  const auditAuth = env('SDKWORK_DRIVE_STAGING_AUDIT_AUTH_TOKEN');
  const auditAccess = env('SDKWORK_DRIVE_STAGING_AUDIT_ACCESS_TOKEN');
  const quotaAuth = env('SDKWORK_DRIVE_STAGING_QUOTA_AUTH_TOKEN');
  const quotaAccess = env('SDKWORK_DRIVE_STAGING_QUOTA_ACCESS_TOKEN');

  const auditTokens =
    auditAuth && auditAccess ? { auth: auditAuth, access: auditAccess } : undefined;
  const quotaTokens =
    quotaAuth && quotaAccess ? { auth: quotaAuth, access: quotaAccess } : undefined;

  console.log('[staging-admin-smoke] running admin surface checks...');
  await runAdminSurfaceSmoke(baseUrl, adminTokens, operatorId);

  if (auditTokens || quotaTokens) {
    console.log('[staging-admin-smoke] running RBAC checks...');
    await runRbacSmoke(baseUrl, auditTokens, quotaTokens);
  } else {
    console.log('[staging-admin-smoke] note: RBAC token pairs not set; skipping scope denial checks');
  }

  console.log('[staging-admin-smoke] passed');
}

main().catch((error) => {
  console.error(`[staging-admin-smoke] failed: ${error instanceof Error ? error.message : String(error)}`);
  process.exit(1);
});
