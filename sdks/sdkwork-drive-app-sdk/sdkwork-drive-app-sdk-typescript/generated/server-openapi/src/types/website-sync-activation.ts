import type { WebsiteRoot } from './website-root';
import type { WebsiteSync } from './website-sync';

export interface WebsiteSyncActivation {
  sync: WebsiteSync;
  websiteRoot: WebsiteRoot;
}
