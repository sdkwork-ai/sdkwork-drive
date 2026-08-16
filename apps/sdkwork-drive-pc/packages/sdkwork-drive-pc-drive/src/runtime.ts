import type { DrivePcSdkPorts } from './sdkPorts';
import { configureDrivePcSdkPorts } from './sdkPorts';

export interface ConfigureDrivePcRuntimeOptions {
  sdkPorts: DrivePcSdkPorts;
}

export function configureDrivePcRuntime(options: ConfigureDrivePcRuntimeOptions): void {
  configureDrivePcSdkPorts(options.sdkPorts);
}

export { configureDrivePcRuntime as configureDrivePcRuntimeFromDrivePackage };
