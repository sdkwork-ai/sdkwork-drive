export interface DriveNodeContent {
  nodeId: string;
  fileName?: string;
  contentType?: string;
  encoding: 'utf8' | 'base64';
  content: string;
  sizeBytes: string;
  checksumSha256?: string;
  returnedRangeStart: string;
  returnedRangeLength: string;
  hasMore: boolean;
}
