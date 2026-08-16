import { describe, expect, it } from 'vitest';
import {
  buildShareLinkClaimPath,
  drivePathToSection,
  driveSectionToPath,
  isShareLinkClaimPath,
  parseShareLinkClaimToken,
  resolveDriveSectionPath,
} from '../src/routing/driveSectionRoutes';

describe('drive section routes', () => {
  it('maps built-in sections to stable paths', () => {
    expect(driveSectionToPath('my-storage')).toBe('/my-storage');
    expect(driveSectionToPath('recent')).toBe('/recent');
    expect(driveSectionToPath('admin-storage-providers')).toBe('/admin/storage-providers');
    expect(driveSectionToPath('admin-audit')).toBe('/admin/audit');
    expect(driveSectionToPath('admin-maintenance')).toBe('/admin/maintenance');
    expect(driveSectionToPath('admin-quotas')).toBe('/admin/quotas');
    expect(driveSectionToPath('admin-labels')).toBe('/admin/labels');
    expect(driveSectionToPath('admin-spaces')).toBe('/admin/spaces');
    expect(driveSectionToPath('admin-download-packages')).toBe('/admin/download-packages');
  });

  it('maps dynamic space sections to /spaces/:id paths', () => {
    expect(driveSectionToPath('space-kb-engineering')).toBe('/spaces/space-kb-engineering');
    expect(driveSectionToPath('space with spaces')).toBe('/spaces/space%20with%20spaces');
  });

  it('resolves paths back to section ids', () => {
    expect(drivePathToSection('/recent')).toBe('recent');
    expect(drivePathToSection('/admin/storage-bindings')).toBe('admin-storage-bindings');
    expect(drivePathToSection('/admin/audit')).toBe('admin-audit');
    expect(drivePathToSection('/admin/maintenance')).toBe('admin-maintenance');
    expect(drivePathToSection('/admin/quotas')).toBe('admin-quotas');
    expect(drivePathToSection('/admin/labels')).toBe('admin-labels');
    expect(drivePathToSection('/admin/spaces')).toBe('admin-spaces');
    expect(drivePathToSection('/admin/download-packages')).toBe('admin-download-packages');
    expect(drivePathToSection('/spaces/space-kb-engineering')).toBe('space-kb-engineering');
    expect(drivePathToSection('/')).toBe('my-storage');
  });

  it('normalizes unknown paths to the default section path', () => {
    expect(resolveDriveSectionPath('/unknown')).toBe('/my-storage');
  });

  it('preserves share link claim paths for authenticated deep links', () => {
    expect(buildShareLinkClaimPath('claim-share-token')).toBe('/share/claim-share-token');
    expect(parseShareLinkClaimToken('/share/claim-share-token')).toBe('claim-share-token');
    expect(isShareLinkClaimPath('/share/claim-share-token')).toBe(true);
    expect(resolveDriveSectionPath('/share/claim-share-token')).toBe('/share/claim-share-token');
    expect(drivePathToSection('/share/claim-share-token')).toBe('shared');
  });

  it('encodes share tokens with reserved path characters', () => {
    const token = 'token/with spaces+plus';
    expect(buildShareLinkClaimPath(token)).toBe(`/share/${encodeURIComponent(token)}`);
    expect(parseShareLinkClaimToken(buildShareLinkClaimPath(token))).toBe(token);
  });

  it('rejects empty or malformed share claim paths', () => {
    expect(buildShareLinkClaimPath('')).toBe('/share');
    expect(parseShareLinkClaimToken('/share/')).toBe(null);
    expect(parseShareLinkClaimToken('/share')).toBe(null);
    expect(isShareLinkClaimPath('/shared')).toBe(false);
  });
});
