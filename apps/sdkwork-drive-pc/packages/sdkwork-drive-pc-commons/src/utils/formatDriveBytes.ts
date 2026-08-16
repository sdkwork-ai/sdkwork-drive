import { formatBytes } from '@sdkwork/utils';

export function formatDriveBytes(bytes?: number | null): string {
  if (bytes === undefined || bytes === null) {
    return '--';
  }
  if (bytes === 0) {
    return '0 B';
  }
  return formatBytes(bytes);
}
