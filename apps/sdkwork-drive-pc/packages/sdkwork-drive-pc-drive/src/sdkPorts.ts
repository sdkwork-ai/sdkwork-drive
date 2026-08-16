import type { SessionSnapshot } from 'sdkwork-drive-pc-core';

export interface DrivePcSdkPorts {
  getDriveClient: () => unknown;
  readHostSession: () => SessionSnapshot | null;
  subscribeHostSession?: (listener: () => void) => () => void;
  resolveHostLanguage?: () => string;
  subscribeHostLanguage?: (listener: (language: string) => void) => () => void;
}

let sdkPorts: DrivePcSdkPorts | null = null;

export function configureDrivePcSdkPorts(ports: DrivePcSdkPorts): void {
  sdkPorts = ports;
}

export function getDrivePcSdkPorts(): DrivePcSdkPorts {
  if (!sdkPorts) {
    throw new Error('Drive PC SDK ports are not configured. Call configureDrivePcSdkPorts first.');
  }
  return sdkPorts;
}

export function tryGetDrivePcSdkPorts(): DrivePcSdkPorts | null {
  return sdkPorts;
}

export function getConfiguredDriveAppSdkClient(): unknown {
  return getDrivePcSdkPorts().getDriveClient();
}
