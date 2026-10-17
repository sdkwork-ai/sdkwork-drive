import type { PositiveInt64String } from './positive-int64-string';

export interface RootScopeSubscription {
  uuid: string;
  spaceId: string;
  consumerKind: 'KNOWLEDGEBASE_RAW';
  consumerResourceId: string;
  rootNodeId: string;
  scopeStatus: 'ACTIVE' | 'SUSPENDED' | 'REVOKED';
  version: PositiveInt64String;
  createdAt: string;
  updatedAt: string;
}
