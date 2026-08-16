import type { DriveSection } from '../pages/DrivePage';

const BUILTIN_SECTION_PATHS: Record<string, string> = {
  'my-storage': '/my-storage',
  recent: '/recent',
  starred: '/starred',
  shared: '/shared',
  computers: '/computers',
  transfer: '/transfer',
  trash: '/trash',
  apps: '/apps',
  'admin-storage-providers': '/admin/storage-providers',
  'admin-storage-bindings': '/admin/storage-bindings',
  'admin-storage-kinds': '/admin/storage-kinds',
  'admin-storage-buckets': '/admin/storage-buckets',
  'admin-audit': '/admin/audit',
  'admin-maintenance': '/admin/maintenance',
  'admin-quotas': '/admin/quotas',
  'admin-labels': '/admin/labels',
  'admin-spaces': '/admin/spaces',
  'admin-download-packages': '/admin/download-packages',
};

const DEFAULT_SECTION: DriveSection = 'my-storage';
const DEFAULT_PATH = BUILTIN_SECTION_PATHS[DEFAULT_SECTION];

export function driveSectionToPath(section: DriveSection): string {
  const builtinPath = BUILTIN_SECTION_PATHS[section];
  if (builtinPath) {
    return builtinPath;
  }
  return `/spaces/${encodeURIComponent(section)}`;
}

export function drivePathToSection(pathname: string): DriveSection {
  const normalized = normalizePathname(pathname);
  if (isShareLinkClaimPath(normalized)) {
    return 'shared';
  }
  for (const [section, path] of Object.entries(BUILTIN_SECTION_PATHS)) {
    if (normalized === path) {
      return section;
    }
  }

  const spaceMatch = normalized.match(/^\/spaces\/([^/]+)$/);
  if (spaceMatch?.[1]) {
    return decodeURIComponent(spaceMatch[1]);
  }

  return DEFAULT_SECTION;
}

export function resolveDriveSectionPath(pathname: string): string {
  if (isShareLinkClaimPath(pathname)) {
    return normalizePathname(pathname);
  }
  const section = drivePathToSection(pathname);
  return driveSectionToPath(section);
}

export function parseShareLinkClaimToken(pathname: string): string | null {
  const normalized = normalizePathname(pathname);
  const match = normalized.match(/^\/share\/([^/]+)$/);
  if (!match?.[1]) {
    return null;
  }
  try {
    const token = decodeURIComponent(match[1]).trim();
    return token || null;
  } catch {
    return null;
  }
}

export function buildShareLinkClaimPath(token: string): string {
  const trimmed = token.trim();
  if (!trimmed) {
    return '/share';
  }
  return `/share/${encodeURIComponent(trimmed)}`;
}

export function isShareLinkClaimPath(pathname: string): boolean {
  return parseShareLinkClaimToken(pathname) !== null;
}

function normalizePathname(pathname: string): string {
  if (!pathname || pathname === '/') {
    return DEFAULT_PATH;
  }
  const trimmed = pathname.replace(/\/+$/, '');
  return trimmed || DEFAULT_PATH;
}
