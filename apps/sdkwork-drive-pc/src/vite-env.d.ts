/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_DRIVE_PC_ENVIRONMENT?: string;
  readonly VITE_DRIVE_PC_CONFIG_PROFILE?: string;
  readonly VITE_DRIVE_PC_BUILD_MODE?: string;
  readonly VITE_DRIVE_PC_RUNTIME_TARGET?: string;
  readonly VITE_DRIVE_PC_APP_API_BASE_URL?: string;
  readonly VITE_DRIVE_PC_BACKEND_API_BASE_URL?: string;
  readonly VITE_DRIVE_PC_APPBASE_APP_API_BASE_URL?: string;
  readonly VITE_DRIVE_PC_DRIVE_APP_API_BASE_URL?: string;
  readonly VITE_DRIVE_PC_DRIVE_ADMIN_STORAGE_API_BASE_URL?: string;
  readonly VITE_DRIVE_PC_PLATFORM_API_GATEWAY_HTTP_URL?: string;
  readonly VITE_DRIVE_PC_DEPLOYMENT_PROFILE?: 'standalone' | 'cloud';
  /** @deprecated use VITE_DRIVE_PC_DEPLOYMENT_PROFILE */
  readonly VITE_DRIVE_PC_HOSTING?: 'self-hosted' | 'cloud-hosted';
  /** @deprecated use VITE_DRIVE_PC_DEPLOYMENT_PROFILE */
  readonly VITE_DRIVE_PC_TOPOLOGY?: 'standalone' | 'cloud';
  readonly VITE_DRIVE_PC_DEV_SAME_ORIGIN_API?: string;
  readonly VITE_DRIVE_PC_TOKEN_MANAGER_MODE?: string;
  readonly VITE_DRIVE_PC_TOKEN_STORAGE?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
