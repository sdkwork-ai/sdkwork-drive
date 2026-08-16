export interface SdkworkDrivePcAdminModuleDescriptor {
  readonly capability: string;
  readonly id: string;
  readonly packageName: string;
  readonly routes: readonly SdkworkDrivePcAdminRouteDescriptor[];
}

export interface SdkworkDrivePcAdminRouteDescriptor {
  readonly id: string;
  readonly path: string;
  readonly screen: string;
  readonly surface: 'backend-admin';
}

const ADMIN_STORAGE_ROUTES: readonly SdkworkDrivePcAdminRouteDescriptor[] = [
  { id: 'admin-storage-providers', path: '/admin/storage-providers', screen: 'storage-providers', surface: 'backend-admin' },
  { id: 'admin-storage-bindings', path: '/admin/storage-bindings', screen: 'storage-bindings', surface: 'backend-admin' },
];

const ADMIN_OPERATIONS_ROUTES: readonly SdkworkDrivePcAdminRouteDescriptor[] = [
  { id: 'admin-audit', path: '/admin/audit', screen: 'audit', surface: 'backend-admin' },
  { id: 'admin-maintenance', path: '/admin/maintenance', screen: 'maintenance', surface: 'backend-admin' },
  { id: 'admin-quotas', path: '/admin/quotas', screen: 'quotas', surface: 'backend-admin' },
  { id: 'admin-labels', path: '/admin/labels', screen: 'labels', surface: 'backend-admin' },
  { id: 'admin-spaces', path: '/admin/spaces', screen: 'spaces', surface: 'backend-admin' },
  { id: 'admin-download-packages', path: '/admin/download-packages', screen: 'download-packages', surface: 'backend-admin' },
];

export function createSdkworkCoreModuleRegistry(): readonly SdkworkDrivePcAdminModuleDescriptor[] {
  return [
    {
      capability: 'drive-admin-storage',
      id: 'drive-admin-storage',
      packageName: 'sdkwork-drive-pc-admin-storage-providers',
      routes: ADMIN_STORAGE_ROUTES,
    },
    {
      capability: 'drive-admin-operations',
      id: 'drive-admin-operations',
      packageName: 'sdkwork-drive-pc-admin-operations',
      routes: ADMIN_OPERATIONS_ROUTES,
    },
  ];
}
