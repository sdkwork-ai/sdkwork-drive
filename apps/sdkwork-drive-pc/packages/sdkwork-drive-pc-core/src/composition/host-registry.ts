export interface SdkworkDrivePcHostDescriptor {
  readonly appKey: string;
  readonly architecture: string;
  readonly capability: string;
  readonly domain: string;
  readonly routePrefix: string;
  readonly runtimeFamily: string;
  readonly surface: 'app';
}

export function createSdkworkCoreHostRegistry(): SdkworkDrivePcHostDescriptor {
  return {
    appKey: 'sdkwork-drive-pc',
    architecture: 'pc-react',
    capability: 'pc-core',
    domain: 'drive',
    routePrefix: '/',
    runtimeFamily: 'desktop',
    surface: 'app',
  };
}
