import type { WebsiteRootSelector } from './website-root-selector';

export interface CreateWebsiteRootRequest {
  rootKey: string;
  displayName: string;
  sourceRoot: WebsiteRootSelector;
  contentMode: 'LIVE_TREE' | 'ATOMIC_GENERATION';
}
