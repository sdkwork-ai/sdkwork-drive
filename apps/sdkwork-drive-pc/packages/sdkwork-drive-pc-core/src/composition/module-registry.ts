export interface SdkworkDrivePcModuleDescriptor {
  readonly capability: string;
  readonly id: string;
  readonly packageName: string;
  readonly routes: readonly SdkworkDrivePcRouteDescriptor[];
}

export interface SdkworkDrivePcRouteDescriptor {
  readonly id: string;
  readonly path: string;
  readonly screen: string;
  readonly surface: 'app';
}

const DRIVE_FILE_MODULE_ROUTES: readonly SdkworkDrivePcRouteDescriptor[] = [
  { id: 'my-storage', path: '/my-storage', screen: 'file-browser', surface: 'app' },
  { id: 'recent', path: '/recent', screen: 'file-browser', surface: 'app' },
  { id: 'starred', path: '/starred', screen: 'file-browser', surface: 'app' },
  { id: 'shared', path: '/shared', screen: 'file-browser', surface: 'app' },
  { id: 'trash', path: '/trash', screen: 'file-browser', surface: 'app' },
  { id: 'share-claim', path: '/share/:token', screen: 'share-claim', surface: 'app' },
];

const DRIVE_TRANSFER_MODULE_ROUTES: readonly SdkworkDrivePcRouteDescriptor[] = [
  { id: 'transfer', path: '/transfer', screen: 'transfer-center', surface: 'app' },
];

export function createSdkworkCoreModuleRegistry(): readonly SdkworkDrivePcModuleDescriptor[] {
  return [
    {
      capability: 'drive-file',
      id: 'drive-file',
      packageName: 'sdkwork-drive-pc-file',
      routes: DRIVE_FILE_MODULE_ROUTES,
    },
    {
      capability: 'drive-transfer',
      id: 'drive-transfer',
      packageName: 'sdkwork-drive-pc-transfer',
      routes: DRIVE_TRANSFER_MODULE_ROUTES,
    },
    {
      capability: 'drive-shell',
      id: 'drive-shell',
      packageName: 'sdkwork-drive-pc-drive',
      routes: [{ id: 'drive-root', path: '/*', screen: 'drive-view', surface: 'app' }],
    },
  ];
}
