export interface DownloadPackageItem {
  nodeId: string;
  nodeName: string;
  archivePath: string;
  bucket: string;
  objectKey: string;
  contentType: string;
  contentLength: string;
  checksumSha256Hex: string;
}
