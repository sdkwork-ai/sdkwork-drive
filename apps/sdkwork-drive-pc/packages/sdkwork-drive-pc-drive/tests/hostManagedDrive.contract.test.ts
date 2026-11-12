import { describe, expect, it } from 'vitest';
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import path from 'node:path';

import { configureDrivePcSdkPorts, getDrivePcSdkPorts } from '../src/sdkPorts';

const packageRoot = path.resolve(__dirname, '..');

function listSourceFiles(root: string): string[] {
  const files: string[] = [];
  for (const entry of readdirSync(root)) {
    const absolute = path.join(root, entry);
    const stat = statSync(absolute);
    if (stat.isDirectory()) {
      files.push(...listSourceFiles(absolute));
      continue;
    }
    if (entry.endsWith('.ts') || entry.endsWith('.tsx')) {
      files.push(absolute);
    }
  }
  return files;
}

describe('sdkwork-drive-pc-drive host module contract', () => {
  it('uses canonical package identity and host SDK ports', () => {
    const packageJson = JSON.parse(
      readFileSync(path.join(packageRoot, 'package.json'), 'utf8'),
    ) as { name?: string };

    expect(packageJson.name).toBe('sdkwork-drive-pc-drive');
    expect(() => getDrivePcSdkPorts()).toThrow(/not configured/i);
    configureDrivePcSdkPorts({
      getDriveClient: () => ({ operationId: 'host-managed-drive-client' }),
      readHostSession: () => null,
    });
    expect(getDrivePcSdkPorts().getDriveClient()).toEqual({
      operationId: 'host-managed-drive-client',
    });
  });

  it('does not use raw HTTP in authored package sources', () => {
    const srcRoot = path.join(packageRoot, 'src');
    expect(existsSync(srcRoot)).toBe(true);
    const combined = listSourceFiles(srcRoot)
      .map((file) => readFileSync(file, 'utf8'))
      .join('\n');
    expect(combined).not.toMatch(/\bfetch\s*\(/);
    expect(combined).not.toContain('../../../src/index.css');
  });

  it('supports host-managed language ports for embedding', () => {
    const driveView = readFileSync(path.join(packageRoot, 'src', 'DriveView.tsx'), 'utf8');
    const sdkPortsSource = readFileSync(path.join(packageRoot, 'src', 'sdkPorts.ts'), 'utf8');

    expect(sdkPortsSource).toContain('resolveHostLanguage');
    expect(sdkPortsSource).toContain('subscribeHostLanguage');
    expect(driveView).toContain('subscribeHostLanguage');
  });

  it('accepts stable Drive node preview requests from an embedding host', () => {
    const driveView = readFileSync(path.join(packageRoot, 'src', 'DriveView.tsx'), 'utf8');
    const driveIndex = readFileSync(path.join(packageRoot, 'src', 'index.ts'), 'utf8');
    const filePackageRoot = path.resolve(packageRoot, '..', 'sdkwork-drive-pc-file');
    const drivePage = readFileSync(
      path.join(filePackageRoot, 'src', 'pages', 'DrivePage.tsx'),
      'utf8',
    );
    const fileBrowser = readFileSync(
      path.join(filePackageRoot, 'src', 'components', 'FileBrowser.tsx'),
      'utf8',
    );
    const openRequestType = readFileSync(
      path.join(filePackageRoot, 'src', 'types', 'driveOpenRequest.ts'),
      'utf8',
    );

    expect(openRequestType).toContain("section: 'recent'");
    expect(openRequestType).toContain("intent: 'preview'");
    expect(openRequestType).toContain('nodeId: string');
    expect(openRequestType).toContain('spaceId?: string');
    expect(driveIndex).toContain('DriveOpenRequest');
    expect(driveView).toContain('openRequest={openRequest}');
    expect(drivePage).toContain('setActiveSection(openRequest.section)');
    expect(fileBrowser).toContain('.getNodeDetails(openRequest.nodeId');
    expect(fileBrowser).toContain('setSelectedPreviewFile(file)');
    expect(fileBrowser).not.toContain('openRequest.downloadUrl');
    expect(fileBrowser).not.toContain('openRequest.signedSourceUrl');
  });

  it('resets account-scoped state when the host swaps the signed-in account', () => {
    const driveView = readFileSync(path.join(packageRoot, 'src', 'DriveView.tsx'), 'utf8');
    const filePackageRoot = path.resolve(packageRoot, '..', 'sdkwork-drive-pc-file');
    const drivePage = readFileSync(
      path.join(filePackageRoot, 'src', 'pages', 'DrivePage.tsx'),
      'utf8',
    );
    const fileBrowser = readFileSync(
      path.join(filePackageRoot, 'src', 'components', 'FileBrowser.tsx'),
      'utf8',
    );
    const accountScope = readFileSync(
      path.join(filePackageRoot, 'src', 'state', 'useDriveAccountScope.ts'),
      'utf8',
    );
    const cutClipboard = readFileSync(
      path.join(filePackageRoot, 'src', 'state', 'driveCutClipboard.ts'),
      'utf8',
    );
    const hostRuntime = readFileSync(
      path.join(packageRoot, 'src', 'createHostManagedDriveRuntime.ts'),
      'utf8',
    );

    // The host can swap accounts without remounting this view, so the workspace
    // is keyed by the account and the file service invalidates its caches.
    expect(driveView).toContain('resolveDriveStorageScope');
    expect(driveView).toContain('key={accountScopeKey}');
    expect(accountScope).toContain('fileService.syncAccountScope(scopeKey)');
    expect(accountScope).toContain('setDriveCutClipboardScope(scopeKey)');
    expect(accountScope).toContain('canPersist: signedIn && scopeKey !== \'\'');
    expect(cutClipboard).toContain('export function setDriveCutClipboardScope');
    expect(hostRuntime).toContain('resolveUploaderStorageScope');

    // DrivePage persists only for an account that owns the data, and the browser
    // reloads when the account changes.
    expect(drivePage).toContain('useDriveAccountScope');
    expect(drivePage).toContain('if (!canPersistTransferJobs)');
    expect(drivePage).toContain('accountScopeKey={storageScopeKey}');
    expect(drivePage).toContain('[fileService, storageScopeKey]');
    expect(fileBrowser).toContain('accountScopeKey');
  });

  it('ships workspace chrome layout rules through driveSurface.css', () => {
    const driveSurface = readFileSync(
      path.join(packageRoot, 'src', 'driveSurface.css'),
      'utf8',
    );
    const workspaceChrome = readFileSync(
      path.join(packageRoot, 'src', 'driveWorkspaceChrome.css'),
      'utf8',
    );

    expect(driveSurface).toContain('@import "./driveWorkspaceChrome.css"');
    expect(workspaceChrome).toContain('.sdkwork-drive-file-list-header');
    expect(workspaceChrome).toContain('.sdkwork-drive-file-header');
  });
});
