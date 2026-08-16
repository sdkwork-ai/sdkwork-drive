export interface DriveNode {
  id: string;
  tenantId?: string;
  spaceId: string;
  parentNodeId?: string;
  nodeType: 'file' | 'folder' | 'shortcut' | 'virtual_reference';
  nodeName: string;
  lifecycleStatus: 'active' | 'trashed' | 'deleted';
  version: string;
  /** Target node id when nodeType is shortcut. */
  shortcutTargetNodeId?: string | null;
  /** Drive uploader usage context identifier. Optional semantic context for idempotency, ownership, and cleanup scoping. */
  scene?: string;
  /** Drive uploader usage context identifier. Optional semantic context for idempotency, ownership, and cleanup scoping. */
  source?: string;
  spaceType: 'personal' | 'team' | 'knowledge_base' | 'ai_generated' | 'git_repository' | 'deployment' | 'app_upload' | 'im' | 'rtc' | 'notary' | 'website';
  /** File content lifecycle on the node. Folders and shortcuts remain empty. */
  contentState?: 'empty' | 'uploading' | 'ready' | 'failed';
  /** Normalized file extension without a leading dot. */
  fileExtension?: string;
  /** MIME type of the latest active file version. */
  contentType?: string;
  /** Coarse file category derived from contentType for list filtering and icons. */
  contentTypeGroup?: 'image' | 'video' | 'audio' | 'text' | 'document' | 'archive' | 'binary';
  /** Byte size of the latest active file version. */
  contentLength?: string;
  /** Optional UI folder color from node property ui.folderColor. */
  folderColor?: string;
  /** Server-owned node creation timestamp. */
  createdAt: string;
  /** Server-owned node last modification timestamp. */
  updatedAt: string;
}
