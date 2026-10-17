import { HttpClient, createHttpClient } from './http/client';
import type { SdkworkCustomConfig } from './types/common';

import { DriveInternalPublishingApi, createDriveInternalPublishingApi } from './api/drive-internal-publishing';

export class SdkworkCustomClient {
  private httpClient: HttpClient;

  public readonly driveInternalPublishing: DriveInternalPublishingApi;

  constructor(config: SdkworkCustomConfig) {
    this.httpClient = createHttpClient(config);
    this.driveInternalPublishing = createDriveInternalPublishingApi(this.httpClient);
  }

  setApiKey(apiKey: string): this {
    this.httpClient.setApiKey(apiKey);
    return this;
  }
  get http(): HttpClient {
    return this.httpClient;
  }
}

export function createClient(config: SdkworkCustomConfig): SdkworkCustomClient {
  return new SdkworkCustomClient(config);
}

export default SdkworkCustomClient;
