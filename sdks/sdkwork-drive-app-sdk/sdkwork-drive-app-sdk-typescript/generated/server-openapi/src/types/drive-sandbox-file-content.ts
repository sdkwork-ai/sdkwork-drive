import type { DriveSandboxEntry } from './drive-sandbox-entry';

export interface DriveSandboxFileContent {
  entry: DriveSandboxEntry;
  encoding: 'utf8' | 'base64';
  content: string;
  sizeBytes: string;
  checksumSha256: string;
}
