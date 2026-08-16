export interface SdkworkDrivePcAdminSdkDescriptor {
  readonly composedFacade: string;
  readonly family: string;
  readonly surface: 'backend-api';
  readonly credentialMode: 'authenticated-backend-admin';
  readonly status: 'available';
}

const SDKWORK_DRIVE_PC_ADMIN_SDK_INVENTORY: readonly SdkworkDrivePcAdminSdkDescriptor[] = [
  {
    composedFacade: '@sdkwork/drive-admin-storage-sdk',
    family: 'sdkwork-drive-admin-storage-sdk',
    surface: 'backend-api',
    credentialMode: 'authenticated-backend-admin',
    status: 'available',
  },
  {
    composedFacade: '@sdkwork/drive-backend-sdk',
    family: 'sdkwork-drive-backend-sdk',
    surface: 'backend-api',
    credentialMode: 'authenticated-backend-admin',
    status: 'available',
  },
];

export function listSdkworkCoreSdkInventory(): readonly SdkworkDrivePcAdminSdkDescriptor[] {
  return SDKWORK_DRIVE_PC_ADMIN_SDK_INVENTORY;
}
