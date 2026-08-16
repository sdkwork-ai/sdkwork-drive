import type { SandboxExplorerPort } from './contracts';

let configuredPort: SandboxExplorerPort | null = null;

export function configureDriveSandboxExplorerRuntime(input: { port: SandboxExplorerPort }): void {
  configuredPort = input.port;
}

export function requireDriveSandboxExplorerPort(): SandboxExplorerPort {
  if (!configuredPort) {
    throw new Error('Drive Sandbox Explorer must be configured with a SandboxExplorerPort before rendering.');
  }
  return configuredPort;
}
