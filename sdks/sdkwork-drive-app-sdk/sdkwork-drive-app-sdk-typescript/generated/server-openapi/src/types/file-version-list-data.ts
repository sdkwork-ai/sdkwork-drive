import type { FileVersion } from './file-version';
import type { PageInfo } from './page-info';

export interface FileVersionListData {
  items: FileVersion[];
  pageInfo: PageInfo;
}
