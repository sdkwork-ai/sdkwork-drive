/** One drive node as seen by the tenant-scoped backend admin file list. Metadata only: object storage keys and credentials never enter this projection. */
export interface DriveFile {
  id: string;
  tenantId: string;
  spaceId: string;
  spaceName: string;
  spaceType: 'personal' | 'team' | 'knowledge_base' | 'ai_generated' | 'git_repository' | 'deployment' | 'app_upload' | 'im' | 'rtc' | 'notary' | 'website' | 'documents';
  ownerSubjectType: string;
  ownerSubjectId: string;
  parentNodeId?: string | null;
  nodeType: 'file' | 'folder' | 'shortcut' | 'virtual_reference';
  nodeName: string;
  /** Slash-separated logical path inside the space, resolved from the node ancestor chain. */
  filePath: string;
  fileExtension?: string | null;
  contentType?: string | null;
  /** Head media group: image, video, audio, text, document, archive, or binary. */
  contentTypeGroup?: string | null;
  /** Head content length in bytes; 0 for nodes without file content. */
  contentLength: string;
  lifecycleStatus: 'active' | 'trashed' | 'deleted';
  version: string;
  createdBy: string;
  updatedBy: string;
  createdAt: string;
  updatedAt: string;
}
