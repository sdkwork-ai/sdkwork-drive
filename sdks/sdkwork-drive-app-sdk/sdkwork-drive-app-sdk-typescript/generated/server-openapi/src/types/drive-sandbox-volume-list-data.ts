import type { DriveSandboxVolume } from './drive-sandbox-volume';
import type { PageInfo } from './page-info';

export interface DriveSandboxVolumeListData {
  items: DriveSandboxVolume[];
  pageInfo: PageInfo;
}
