import type { WebsiteGeneration } from './website-generation';
import type { WebsiteRoot } from './website-root';

export interface WebsiteGenerationActivation {
  sourceGeneration: WebsiteGeneration;
  websiteRoot: WebsiteRoot;
}
