import type { PositiveInt64String } from './positive-int64-string';

export interface WebsiteRoot {
  uuid: string;
  spaceId: string;
  sourceRootMode: 'SPACE_ROOT' | 'FOLDER';
  contentMode: 'LIVE_TREE' | 'ATOMIC_GENERATION';
  activeGeneration: PositiveInt64String;
  rootStatus: 'ACTIVE' | 'SUSPENDED' | 'INVALID' | 'ARCHIVED';
  capabilities: ('STATIC_CONTENT' | 'BYTE_RANGE' | 'CONDITIONAL_REQUESTS')[];
  version: PositiveInt64String;
  updatedAt: string;
}
