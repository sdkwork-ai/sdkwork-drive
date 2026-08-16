#!/usr/bin/env node

import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import {
  IAM_APPLICATION_BOOTSTRAP_ENV,
  loadEnvFile,
  loadProfile,
  mergeRuntimeEnv,
  REPO_ROOT,
  resolveIamDevEnv,
  resolveStandaloneGatewayConfigPath,
  resolveTemporaryDatabaseDriverEnv,
} from './lib/drive-topology.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = REPO_ROOT;

const DEFAULT_ENVIRONMENT = 'development';
function cargoCommand() {
  return process.platform === 'win32' ? 'cargo.exe' : 'cargo';
}

function normalizeText(value) {
  const normalized = String(value ?? '').trim();
  return normalized || undefined;
}

function loadDevEnvFile(envFile) {
  return loadEnvFile(envFile, repoRoot);
}

function parseArgs(argv) {
  const settings = {
    environment: DEFAULT_ENVIRONMENT,
    config: undefined,
    devEnvFile: '.env.postgres',
    release: false,
    dryRun: false,
    help: false,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--help' || arg === '-h') {
      settings.help = true;
      continue;
    }
    if (arg === '--dry-run') {
      settings.dryRun = true;
      continue;
    }
    if (arg === '--release') {
      settings.release = true;
      continue;
    }
    if (arg === '--environment') {
      settings.environment = argv[index + 1];
      index += 1;
      continue;
    }
    if (arg === '--config') {
      settings.config = argv[index + 1];
      index += 1;
      continue;
    }
    if (arg === '--dev-env-file') {
      settings.devEnvFile = argv[index + 1];
      index += 1;
    }
  }

  return settings;
}

function resolveConfigPath(settings) {
  if (settings.config) {
    return path.isAbsolute(settings.config)
      ? settings.config
      : path.resolve(repoRoot, settings.config);
  }
  return resolveStandaloneGatewayConfigPath(
    { SDKWORK_DRIVE_STANDALONE_GATEWAY_ENVIRONMENT: settings.environment },
    repoRoot,
  );
}

function printHelp() {
  console.log(`Usage: node scripts/gateway-standalone-run.mjs [options]

Drive standalone gateway embeds appbase IAM and the Drive application assembly.
Use this for local/standalone deployment. Cloud composition is platform-owned.

Options:
  --environment <development|production>  Config profile (default: development)
  --config <path>                       Explicit TOML config path
  --dev-env-file <path>                 Env file for IAM/database (default: .env.postgres)
  --release                             Build/run release profile
  --dry-run                             Print command without executing
  --help, -h                            Show this help

Environment overrides:
  SDKWORK_DRIVE_STANDALONE_GATEWAY_CONFIG
  SDKWORK_DRIVE_STANDALONE_GATEWAY_ENVIRONMENT
  SDKWORK_DRIVE_STANDALONE_GATEWAY_BIND
`);
}

function main() {
  const settings = parseArgs(process.argv.slice(2));
  if (settings.help) {
    printHelp();
    process.exit(0);
  }

  const configPath = resolveConfigPath(settings);
  if (!fs.existsSync(configPath)) {
    console.error(`[sdkwork-drive] standalone gateway config not found: ${configPath}`);
    process.exit(1);
  }

  const profileEnv = loadProfile(`standalone.${settings.environment}`);
  const fileEnv = loadDevEnvFile(settings.devEnvFile);
  const runtimeEnv = mergeRuntimeEnv(process.env, profileEnv, fileEnv);
  const gatewayEnv = {
    ...resolveIamDevEnv(runtimeEnv, repoRoot),
    ...resolveTemporaryDatabaseDriverEnv(runtimeEnv),
    ...IAM_APPLICATION_BOOTSTRAP_ENV,
    SDKWORK_DRIVE_STANDALONE_GATEWAY_CONFIG: configPath,
    SDKWORK_DRIVE_STANDALONE_GATEWAY_ENVIRONMENT: settings.environment,
  };

  const cargoArgs = [
    'run',
    '-p',
    'sdkwork-api-drive-standalone-gateway',
    '--bin',
    'sdkwork-api-drive-standalone-gateway',
    '--',
    '--config',
    configPath,
  ];
  if (settings.release) {
    cargoArgs.splice(1, 0, '--release');
  }

  if (settings.dryRun) {
    console.log(`[sdkwork-api-drive-standalone-gateway] ${cargoCommand()} ${cargoArgs.join(' ')}`);
    process.exit(0);
  }

  const child = spawn(cargoCommand(), cargoArgs, {
    cwd: repoRoot,
    env: gatewayEnv,
    stdio: 'inherit',
    shell: false,
  });

  const stop = () => {
    if (!child.killed) child.kill('SIGTERM');
  };
  process.once('SIGINT', stop);
  process.once('SIGTERM', stop);

  child.on('exit', (code) => {
    process.removeListener('SIGINT', stop);
    process.removeListener('SIGTERM', stop);
    process.exitCode = code ?? 1;
  });
}

main();
