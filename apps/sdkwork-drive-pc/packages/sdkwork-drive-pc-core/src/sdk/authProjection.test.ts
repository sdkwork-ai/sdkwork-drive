import { describe, expect, it } from 'vitest';
import { omitAuthProjectionBody, omitAuthProjectionQuery } from './authProjection';

describe('authProjection', () => {
  it('strips tenant, actor, and organization projection fields from query params', () => {
    expect(
      omitAuthProjectionQuery({
        tenantId: 'tenant-001',
        userId: 'user-001',
        appId: 'drive-pc',
        organizationId: 'org-001',
        operatorId: 'user-001',
        subjectType: 'user',
        subjectId: 'user-001',
        spaceId: 'space-001',
      }),
    ).toEqual({
      spaceId: 'space-001',
    });
  });

  it('strips tenant, user, app, organization, and actor projection fields from JSON bodies', () => {
    expect(
      omitAuthProjectionBody({
        tenantId: 'tenant-001',
        userId: 'user-001',
        appId: 'drive-pc',
        organizationId: 'org-001',
        operatorId: 'user-001',
        spaceId: 'space-001',
        nodeName: 'folder',
      }),
    ).toEqual({
      spaceId: 'space-001',
      nodeName: 'folder',
    });
  });
});
