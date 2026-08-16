export interface MediaResource {
  id?: string;
  kind: 'image' | 'video' | 'audio' | 'voice' | 'document' | 'archive' | 'model' | 'other';
  source: 'drive' | 'external_url' | 'data_url' | 'provider_asset' | 'generated';
  url?: string;
  publicUrl?: string;
  uri?: string;
  objectBlobId?: string;
  fileName?: string;
  mimeType?: string;
  sizeBytes?: string;
  checksum?: { algorithm?: string; value?: string; };
  width?: number;
  height?: number;
  durationSeconds?: number;
  altText?: string;
  title?: string;
  poster?: MediaResource;
  thumbnails?: MediaResource[];
  variants?: MediaResource[];
  access?: { visibility: 'private' | 'tenant' | 'organization' | 'public' | 'signed'; expiresAt?: string; };
  ai?: { provenance?: 'uploaded' | 'generated' | 'edited' | 'imported'; provider?: string; model?: string; promptId?: string; generationTaskId?: string; sourceMediaIds?: string[]; seed?: string; moderationStatus?: 'unknown' | 'pending' | 'approved' | 'rejected' | 'blocked'; safetyLabels?: string[]; };
  metadata?: Record<string, unknown>;
  /** Legacy alias for id. */
  mediaResourceId?: string;
  /** Legacy alias for kind. */
  mediaType?: string;
  /** Legacy alias for mimeType. */
  contentType?: string;
  /** Legacy duration field. Use durationSeconds. */
  durationMs?: string;
  /** Legacy checksum field. Use checksum. */
  checksumSha256?: string;
}
