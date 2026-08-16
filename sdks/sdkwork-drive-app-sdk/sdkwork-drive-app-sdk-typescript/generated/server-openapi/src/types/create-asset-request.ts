import type { MediaResource } from './media-resource';

export interface CreateAssetRequest {
  /** Existing Drive node to expose through /assets. */
  driveNodeId?: string;
  virtualReference?: { driveSpaceId?: string; title?: string; sourceDomain?: string; sourceResourceType?: string; sourceResourceId?: string; resourceSnapshot?: MediaResource; };
  title?: string;
  description?: string;
  scene?: string;
  source?: string;
  tags?: string[];
}
