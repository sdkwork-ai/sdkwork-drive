#!/usr/bin/env node

import { spawn } from 'node:child_process';
import process from 'node:process';

import {
  loadProfile,
  mergeRuntimeEnv,
  REPO_ROOT,
  resolveBuildProfileId,
} from './lib/drive-topology.mjs';

const repoRoot = REPO_ROOT;

function pnpmCommand() {
  return process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm';
}

function parseArgs(argv) {
  const settings = {
    deploymentProfile: 'cloud',
    debug: false,
    help: false,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--help' || arg === '-h') {
      settings.help = true;
      continue;
    }
    if (arg === '--debug') {
      settings.debug = true;
      continue;
    }
    if (arg === '--deployment-profile') {
      const value = argv[index + 1];
      if (!value || value.startsWith('--')) {
        throw new Error('--deployment-profile requires a value (cloud or standalone)');
      }
      if (value !== 'cloud' && value !== 'standalone') {
        throw new Error('--deployment-profile must be cloud or standalone');
      }
      settings.deploymentProfile = value;
      index += 1;
      continue;
    }
  }

  return settings;
}

function printHelp() {
  console.log(`Usage: node scripts/drive-build.mjs [options]

Build the Drive PC desktop app (Tauri).

Defaults:
  deploymentProfile cloud       Release desktop builds target the cloud production profile.
  deploymentProfile standalone  Standalone desktop builds target local application ingress URLs.

Profiles load from etc/topology according to deployment profile and production environment.

Options:
  --deployment-profile <cloud|standalone>  Deployment profile (default: cloud)
  --debug                               Build debug desktop bundle instead of release
  --help, -h                            Show this help
`);
}

function main() {
  const settings = parseArgs(process.argv.slice(2));
  if (settings.help) {
    printHelp();
    process.exit(0);
  }

  const deploymentProfile = settings.deploymentProfile;
  const profileId = resolveBuildProfileId(deploymentProfile);
  const profileEnv = loadProfile(profileId);
  const buildEnv = mergeRuntimeEnv(process.env, profileEnv, {
    SDKWORK_DRIVE_DEPLOYMENT_PROFILE: deploymentProfile,
    VITE_DRIVE_PC_DEPLOYMENT_PROFILE: deploymentProfile,
    SDKWORK_DRIVE_PROFILE_ID: profileId,
  });
  const buildScript = settings.debug ? 'build:desktop:local' : 'build:desktop';

  const child = spawn(
    pnpmCommand(),
    ['--dir', 'apps/sdkwork-drive-pc', buildScript],
    {
      cwd: repoRoot,
      env: buildEnv,
      stdio: 'inherit',
      shell: true,
    },
  );

  child.on('exit', (code) => {
    process.exitCode = code ?? 1;
  });
}

main();
