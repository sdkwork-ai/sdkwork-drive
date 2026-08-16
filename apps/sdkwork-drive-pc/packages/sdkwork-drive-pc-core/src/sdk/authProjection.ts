const AUTH_PROJECTION_QUERY_KEYS = new Set([
  'tenantId',
  'tenant_id',
  'userId',
  'user_id',
  'appId',
  'app_id',
  'organizationId',
  'organization_id',
  'operatorId',
  'operator_id',
  'subjectType',
  'subject_type',
  'subjectId',
  'subject_id',
]);

const AUTH_PROJECTION_BODY_KEYS = new Set([
  'tenantId',
  'tenant_id',
  'userId',
  'user_id',
  'appId',
  'app_id',
  'organizationId',
  'organization_id',
  'operatorId',
  'operator_id',
  'subjectType',
  'subject_type',
  'subjectId',
  'subject_id',
]);

export function omitAuthProjectionQuery(
  query?: Record<string, string | number | boolean | undefined>,
): Record<string, string | number | boolean | undefined> | undefined {
  if (!query) {
    return undefined;
  }

  const next: Record<string, string | number | boolean | undefined> = {};
  for (const [key, value] of Object.entries(query)) {
    if (!AUTH_PROJECTION_QUERY_KEYS.has(key)) {
      next[key] = value;
    }
  }
  return Object.keys(next).length > 0 ? next : undefined;
}

export function omitAuthProjectionBody(body: unknown): unknown {
  if (typeof body !== 'object' || body === null || Array.isArray(body)) {
    return body;
  }

  const record = body as Record<string, unknown>;
  const next: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(record)) {
    if (!AUTH_PROJECTION_BODY_KEYS.has(key)) {
      next[key] = value;
    }
  }
  return next;
}
