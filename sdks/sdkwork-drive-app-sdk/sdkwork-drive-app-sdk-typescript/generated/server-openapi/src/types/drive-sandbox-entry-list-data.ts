import type { DriveSandboxEntry } from './drive-sandbox-entry';
import type { PageInfo } from './page-info';

export interface DriveSandboxEntryListData {
  items: DriveSandboxEntry[];
  pageInfo: PageInfo;
}
