export interface SdkworkDrivePcSdkDescriptor {
  readonly composedFacade: string;
  readonly family: string;
  readonly surface: 'app-api' | 'backend-api' | 'open-api';
  readonly credentialMode: 'authenticated-app-api';
  readonly status: 'available';
}

const SDKWORK_DRIVE_PC_SDK_INVENTORY: readonly SdkworkDrivePcSdkDescriptor[] = [
  {
    composedFacade: '@sdkwork/drive-app-sdk',
    family: 'sdkwork-drive-app-sdk',
    surface: 'app-api',
    credentialMode: 'authenticated-app-api',
    status: 'available',
  },
  {
    composedFacade: '@sdkwork/iam-app-sdk',
    family: 'sdkwork-iam-app-sdk',
    surface: 'app-api',
    credentialMode: 'authenticated-app-api',
    status: 'available',
  },
];

export function listSdkworkCoreSdkInventory(): readonly SdkworkDrivePcSdkDescriptor[] {
  return SDKWORK_DRIVE_PC_SDK_INVENTORY;
}
