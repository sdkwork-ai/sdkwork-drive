import type { DriveSection } from '../pages/DrivePage';

export function supportsServerSideFileBrowserSort(
  section: DriveSection,
  searchQuery: string,
  _parentId: string | null,
): boolean {
  if (searchQuery.trim().length > 0) {
    return false;
  }
  if (section === 'computers') {
    return false;
  }
  return true;
}
