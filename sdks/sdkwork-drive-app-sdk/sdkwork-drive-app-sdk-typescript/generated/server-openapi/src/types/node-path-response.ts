import type { DriveNode } from './drive-node';

export interface NodePathResponse {
  items: DriveNode[];
  pathSegments: string[];
}
