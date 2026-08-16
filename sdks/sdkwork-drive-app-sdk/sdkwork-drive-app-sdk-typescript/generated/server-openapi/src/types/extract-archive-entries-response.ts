import type { DriveNode } from './drive-node';

export interface ExtractArchiveEntriesResponse {
  items: DriveNode[];
  extractedCount: string;
}
