import type { PositiveInt64String } from './positive-int64-string';

export interface ActivateWebsiteGenerationRequest {
  expectedRootVersion: PositiveInt64String;
  expectedGeneration: PositiveInt64String;
}
