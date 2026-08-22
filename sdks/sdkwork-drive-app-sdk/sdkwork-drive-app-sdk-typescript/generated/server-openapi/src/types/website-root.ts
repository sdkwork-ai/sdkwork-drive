export interface WebsiteRoot {
  uuid: string;
  spaceId: string;
  rootKey: string;
  displayName: string;
  sourceRootMode: 'SPACE_ROOT' | 'FOLDER';
  selectedFolderNodeId: string | null;
  contentMode: 'LIVE_TREE' | 'ATOMIC_GENERATION';
  activeNodeId: string;
  activeGeneration: string;
  rootStatus: 'ACTIVE' | 'SUSPENDED' | 'INVALID' | 'ARCHIVED';
  version: string;
  createdAt: string;
  updatedAt: string;
}
