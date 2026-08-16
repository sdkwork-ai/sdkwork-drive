export interface SdkworkDrivePcAdminHostDescriptor {
  readonly appKey: string;
  readonly architecture: string;
  readonly capability: string;
  readonly domain: string;
  readonly routePrefix: string;
  readonly runtimeFamily: string;
  readonly surface: 'backend-admin';
}

export function createSdkworkCoreHostRegistry(): SdkworkDrivePcAdminHostDescriptor {
  return {
    appKey: 'sdkwork-drive-pc',
    architecture: 'pc-react',
    capability: 'pc-admin-core',
    domain: 'drive',
    routePrefix: '/admin',
    runtimeFamily: 'desktop',
    surface: 'backend-admin',
  };
}
