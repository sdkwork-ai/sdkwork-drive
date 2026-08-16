#!/usr/bin/env node

import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import net from 'node:net';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import {
  assertPostgresReachableForIam,
  describeIamDatabaseTarget,
  loadEnvFile as loadTopologyEnvFile,
  loadProfile,
  mergeRuntimeEnv,
  REPO_ROOT,
  resolveDevProfileId,
  resolveGatewayBind,
  resolveIamDevEnv,
  resolveTemporaryDatabaseDriverEnv,
  IAM_APPLICATION_BOOTSTRAP_ENV,
  shouldAutostartGateway,
} from './lib/drive-topology.mjs';
import { mergeRepoDevBootstrapAccessTokenEnv } from '../../sdkwork-iam/scripts/dev/create-dev-bootstrap-access-token-env.mjs';

const require = createRequire(import.meta.url);
const http = require('http');

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = REPO_ROOT;

const DEFAULT_SDKWORK_API_CLOUD_GATEWAY_BIND = '127.0.0.1:3900';
const DEFAULT_SDKWORK_API_CLOUD_GATEWAY_BASE_URL = `http://${DEFAULT_SDKWORK_API_CLOUD_GATEWAY_BIND}`;
const DEFAULT_DRIVE_APP_API_BIND = '127.0.0.1:18080';
const DEFAULT_DRIVE_APP_API_BASE_URL = `http://${DEFAULT_DRIVE_APP_API_BIND}`;
const DEFAULT_DRIVE_ADMIN_STORAGE_API_BIND = '127.0.0.1:18083';
const DEFAULT_DRIVE_ADMIN_STORAGE_API_BASE_URL =
  `http://${DEFAULT_DRIVE_ADMIN_STORAGE_API_BIND}`;
const DRIVE_PC_DEV_PORT = 5183;
const GATEWAY_HEALTH_PATH = '/healthz';
const GATEWAY_CHECK_TIMEOUT_MS = 2000;
const GATEWAY_STARTUP_WAIT_MS = 1000;
const GATEWAY_MAX_STARTUP_ATTEMPTS = 90;
const POSTGRES_REACHABILITY_TIMEOUT_MS = 2000;

const SDKWORK_API_CLOUD_GATEWAY_BASE_URL_ENV_KEYS = [
  'SDKWORK_API_CLOUD_GATEWAY_BASE_URL',
  'SDKWORK_DRIVE_API_GATEWAY_BASE_URL',
];

function pnpmCommand() {
  return process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm';
}

function cargoCommand() {
  return process.platform === 'win32' ? 'cargo.exe' : 'cargo';
}

function normalizeText(value) {
  const normalized = String(value ?? '').trim();
  return normalized || undefined;
}

function normalizeUpstreamBaseUrl(value, label) {
  const normalized = normalizeText(value);
  if (!normalized) {
    return undefined;
  }
  let parsedUrl;
  try {
    parsedUrl = new URL(normalized);
  } catch {
    throw new Error(`${label} must be a valid absolute http(s) URL`);
  }
  if (parsedUrl.protocol !== 'http:' && parsedUrl.protocol !== 'https:') {
    throw new Error(`${label} must be a valid absolute http(s) URL`);
  }
  return normalized.replace(/\/+$/u, '');
}

function normalizeGatewayBind(value, label = 'SDKWORK_API_CLOUD_GATEWAY_BIND') {
  const normalized = normalizeText(value);
  if (!normalized) {
    return undefined;
  }
  if (normalized.startsWith('http://') || normalized.startsWith('https://')) {
    throw new Error(`${label} must be a host:port bind address, not a URL`);
  }
  return normalized;
}

function resolveSdkworkApiGatewayBind(env = process.env) {
  const deploymentProfile = normalizeText(env.SDKWORK_DRIVE_DEPLOYMENT_PROFILE) || 'standalone';
  return resolveGatewayBind(env, deploymentProfile);
}

function resolveSdkworkApiGatewayBaseUrl(env = process.env) {
  for (const key of SDKWORK_API_CLOUD_GATEWAY_BASE_URL_ENV_KEYS) {
    const baseUrl = normalizeUpstreamBaseUrl(env[key], key);
    if (baseUrl) {
      return baseUrl;
    }
  }
  return `http://${resolveSdkworkApiGatewayBind(env)}`;
}

function parseBooleanEnv(value, defaultValue) {
  const normalized = normalizeText(value);
  if (!normalized) {
    return defaultValue;
  }
  if (['1', 'true', 'on', 'yes'].includes(normalized.toLowerCase())) {
    return true;
  }
  if (['0', 'false', 'off', 'no'].includes(normalized.toLowerCase())) {
    return false;
  }
  return defaultValue;
}

function shouldAutostartSdkworkApiGateway(env) {
  return shouldAutostartGateway(env);
}

function parseHostPort(bind, fallbackHost = '127.0.0.1') {
  const normalized = normalizeText(bind);
  if (!normalized) {
    return { host: fallbackHost, port: undefined };
  }
  const separator = normalized.lastIndexOf(':');
  if (separator <= 0) {
    return { host: fallbackHost, port: Number(normalized) };
  }
  return {
    host: normalized.slice(0, separator) || fallbackHost,
    port: Number(normalized.slice(separator + 1)),
  };
}

function findListeningProcessId(port, host = '127.0.0.1') {
  if (!Number.isFinite(port)) {
    return undefined;
  }

  if (process.platform === 'win32') {
    const result = spawnSync('netstat.exe', ['-ano'], {
      encoding: 'utf8',
      windowsHide: true,
    });
    const hostPort = `${host}:${port}`;
    for (const line of result.stdout.split(/\r?\n/u)) {
      if (!line.includes('LISTENING') || !line.includes(hostPort)) {
        continue;
      }
      const pid = Number(line.trim().split(/\s+/u).at(-1));
      if (Number.isFinite(pid) && pid > 0) {
        return pid;
      }
    }
    return undefined;
  }

  const result = spawnSync('lsof', ['-nP', `-iTCP:${port}`, '-sTCP:LISTEN', '-t'], {
    encoding: 'utf8',
  });
  const pid = Number(String(result.stdout).trim().split(/\r?\n/u)[0]);
  return Number.isFinite(pid) && pid > 0 ? pid : undefined;
}

async function ensureDrivePcDevPortAvailable(env, target) {
  if (target !== 'desktop') {
    return;
  }
  if (!parseBooleanEnv(env.SDKWORK_DRIVE_PC_DEV_RECLAIM_PORT, true)) {
    return;
  }

  if (await isTcpPortAvailable(DRIVE_PC_DEV_PORT)) {
    return;
  }

  const pid = findListeningProcessId(DRIVE_PC_DEV_PORT);
  if (pid) {
    console.warn(
      `[sdkwork-drive] port ${DRIVE_PC_DEV_PORT} is in use by pid ${pid}; stopping stale dev server`,
    );
    terminateProcessTree({ pid });
    await new Promise((resolve) => {
      setTimeout(resolve, 500);
    });
  }

  if (!(await isTcpPortAvailable(DRIVE_PC_DEV_PORT))) {
    throw new Error(
      `Port ${DRIVE_PC_DEV_PORT} is already in use. Stop the process on that port or set SDKWORK_DRIVE_PC_DEV_RECLAIM_PORT=false.`,
    );
  }
}

async function resolveGatewayAutostart(env) {
  if (!shouldAutostartSdkworkApiGateway(env)) {
    return false;
  }

  const { host, port } = parseHostPort(resolveSdkworkApiGatewayBind(env));
  if (!Number.isFinite(port)) {
    return true;
  }

  if (!(await isTcpPortAvailable(port, host))) {
    console.warn(
      `[sdkwork-drive] detected gateway on ${host}:${port}, skipping autostart`,
    );
    return false;
  }

  return true;
}

function resolveExplicitViteAppApiBaseUrl(env) {
  return normalizeUpstreamBaseUrl(env.VITE_DRIVE_PC_APP_API_BASE_URL, 'VITE_DRIVE_PC_APP_API_BASE_URL')
    || normalizeUpstreamBaseUrl(
      env.VITE_DRIVE_PC_DRIVE_APP_API_BASE_URL,
      'VITE_DRIVE_PC_DRIVE_APP_API_BASE_URL',
    );
}

function resolveDevAdminStorageApiBaseUrl(env, apiGatewayBaseUrl) {
  return normalizeUpstreamBaseUrl(
    env.VITE_DRIVE_PC_DRIVE_ADMIN_STORAGE_API_BASE_URL,
    'VITE_DRIVE_PC_DRIVE_ADMIN_STORAGE_API_BASE_URL',
  )
    || normalizeUpstreamBaseUrl(
      env.VITE_DRIVE_PC_BACKEND_API_BASE_URL,
      'VITE_DRIVE_PC_BACKEND_API_BASE_URL',
    )
    || normalizeUpstreamBaseUrl(
      env.SDKWORK_DRIVE_ADMIN_STORAGE_API_BASE_URL,
      'SDKWORK_DRIVE_ADMIN_STORAGE_API_BASE_URL',
    )
    || apiGatewayBaseUrl
    || DEFAULT_DRIVE_ADMIN_STORAGE_API_BASE_URL;
}

async function resolveDevApiBaseUrls(env, gatewayWillStart) {
  const explicitAppApiBaseUrl = resolveExplicitViteAppApiBaseUrl(env);
  if (explicitAppApiBaseUrl) {
    const apiGatewayBaseUrl = normalizeUpstreamBaseUrl(
      env.VITE_DRIVE_PC_PLATFORM_API_GATEWAY_HTTP_URL,
      'VITE_DRIVE_PC_PLATFORM_API_GATEWAY_HTTP_URL',
    ) || explicitAppApiBaseUrl;
    return {
      appApiBaseUrl: explicitAppApiBaseUrl,
      apiGatewayBaseUrl,
      adminStorageApiBaseUrl: resolveDevAdminStorageApiBaseUrl(env, apiGatewayBaseUrl),
    };
  }

  const { host, port } = parseHostPort(resolveSdkworkApiGatewayBind(env));
  const gatewayAlreadyRunning = Number.isFinite(port)
    ? !(await isTcpPortAvailable(port, host))
    : false;

  if (gatewayWillStart || gatewayAlreadyRunning) {
    const apiGatewayBaseUrl = resolveSdkworkApiGatewayBaseUrl(env);
    return {
      appApiBaseUrl: apiGatewayBaseUrl,
      apiGatewayBaseUrl,
      adminStorageApiBaseUrl: resolveDevAdminStorageApiBaseUrl(env, apiGatewayBaseUrl),
    };
  }

  const appApiBaseUrl = normalizeUpstreamBaseUrl(
    env.SDKWORK_DRIVE_APP_API_BASE_URL,
    'SDKWORK_DRIVE_APP_API_BASE_URL',
  ) || DEFAULT_DRIVE_APP_API_BASE_URL;

  return {
    appApiBaseUrl,
    apiGatewayBaseUrl: appApiBaseUrl,
    adminStorageApiBaseUrl: resolveDevAdminStorageApiBaseUrl(env, appApiBaseUrl),
  };
}

function isTcpPortAvailable(port, host = '127.0.0.1') {
  return new Promise((resolve) => {
    const server = net.createServer();
    server.unref();
    server.once('error', () => resolve(false));
    server.listen({ host, port }, () => {
      server.close(() => resolve(true));
    });
  });
}

function isTcpPortReachable(port, host = '127.0.0.1', timeoutMs = POSTGRES_REACHABILITY_TIMEOUT_MS) {
  if (!Number.isFinite(port)) {
    return Promise.resolve(false);
  }

  return new Promise((resolve) => {
    const socket = net.connect({ host, port, timeout: timeoutMs });
    const finish = (reachable) => {
      socket.destroy();
      resolve(reachable);
    };
    socket.once('connect', () => finish(true));
    socket.once('error', () => finish(false));
    socket.once('timeout', () => finish(false));
  });
}

function checkGatewayHealth(gatewayUrl, timeoutMs = GATEWAY_CHECK_TIMEOUT_MS) {
  return new Promise((resolve) => {
    const url = new URL(GATEWAY_HEALTH_PATH, gatewayUrl);
    const request = http.request(
      {
        hostname: url.hostname,
        port: url.port || 80,
        path: url.pathname,
        method: 'GET',
        timeout: timeoutMs,
      },
      (response) => {
        resolve(response.statusCode >= 200 && response.statusCode < 300);
      },
    );

    request.on('error', () => {
      resolve(false);
    });

    request.on('timeout', () => {
      request.destroy();
      resolve(false);
    });

    request.end();
  });
}

function delay(ms) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

function formatGatewayStartupError(exitCode, iamDatabaseTarget) {
  const exitDetail = exitCode == null
    ? 'The dev gateway did not become healthy before the startup timeout.'
    : `The dev gateway exited with code ${exitCode}.`;
  return new Error(
    `${exitDetail}\n`
      + `IAM database target: ${iamDatabaseTarget}\n`
      + 'Ensure PostgreSQL is running and .env.postgres matches your local instance '
      + '(copy from .env.postgres.example if needed).',
  );
}

function loadEnvFile(envFile) {
  return loadTopologyEnvFile(envFile, repoRoot);
}

function parseArgs(argv) {
  const settings = {
    target: 'browser',
    database: undefined,
    devEnvFile: null,
    deploymentProfile: 'standalone',
    clientOnly: false,
    dryRun: false,
    help: false,
  };
  let forwardOnly = false;
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (forwardOnly) {
      continue;
    }
    if (arg === '--') {
      forwardOnly = true;
      continue;
    }
    if (arg === '--help' || arg === '-h') {
      settings.help = true;
      continue;
    }
    if (arg === '--dry-run') {
      settings.dryRun = true;
      continue;
    }
    if (arg === '--client-only') {
      settings.clientOnly = true;
      continue;
    }
    if (arg === '--target') {
      const value = argv[index + 1];
      if (!value || value.startsWith('--')) {
        throw new Error('--target requires a value (browser or desktop)');
      }
      settings.target = value;
      index += 1;
      continue;
    }
    if (arg === '--dev-env-file') {
      const value = argv[index + 1];
      if (!value || value.startsWith('--')) {
        throw new Error('--dev-env-file requires a path');
      }
      settings.devEnvFile = value;
      index += 1;
      continue;
    }
    if (arg === '--deployment-profile') {
      const value = argv[index + 1];
      if (!value || value.startsWith('--')) {
        throw new Error('--deployment-profile requires a value (standalone or cloud)');
      }
      settings.deploymentProfile = value;
      index += 1;
      continue;
    }
  }
  return settings;
}

function resolveStandaloneGatewayConfigPath(env) {
  const explicit = normalizeText(env.SDKWORK_DRIVE_STANDALONE_GATEWAY_CONFIG);
  if (explicit) {
    return path.isAbsolute(explicit) ? explicit : path.resolve(repoRoot, explicit);
  }

  const environment = normalizeText(env.SDKWORK_DRIVE_STANDALONE_GATEWAY_ENVIRONMENT) || 'development';
  return path.resolve(
    repoRoot,
    `etc/sdkwork-api-drive-standalone-gateway.${environment}.toml`,
  );
}

function createManagedGatewayProcess({ env, gatewayWillStart, deploymentProfile }) {
  if (deploymentProfile === 'cloud') {
    return undefined;
  }

  return createStandaloneGatewayProcess({ env, gatewayWillStart });
}

function createStandaloneGatewayProcess({ env, gatewayWillStart }) {
  if (!gatewayWillStart) {
    return undefined;
  }

  const configPath = resolveStandaloneGatewayConfigPath(env);
  const gatewayEnv = {
    ...resolveIamDevEnv(env, repoRoot),
    ...resolveTemporaryDatabaseDriverEnv(env),
    SDKWORK_DRIVE_STANDALONE_GATEWAY_BIND: resolveSdkworkApiGatewayBind(env),
    SDKWORK_DRIVE_STANDALONE_GATEWAY_CONFIG: configPath,
    SDKWORK_DRIVE_STANDALONE_GATEWAY_ENVIRONMENT:
      normalizeText(env.SDKWORK_DRIVE_STANDALONE_GATEWAY_ENVIRONMENT) || 'development',
  };

  return {
    args: [
      'run',
      '-p',
      'sdkwork-api-drive-standalone-gateway',
      '--bin',
      'sdkwork-api-drive-standalone-gateway',
      '--',
      '--config',
      configPath,
    ],
    command: cargoCommand(),
    cwd: repoRoot,
    env: gatewayEnv,
    label: 'sdkwork-api-drive-standalone-gateway',
    shell: false,
  };
}

function createDevServerProcess({ env, target }) {
  const command = pnpmCommand();
  const isDesktop = target === 'desktop';

  if (isDesktop) {
    return {
      args: ['--dir', 'apps/sdkwork-drive-pc', 'dev:desktop'],
      command,
      cwd: repoRoot,
      env,
      label: 'drive-pc-desktop',
      shell: true,
    };
  }

  return {
    args: ['--dir', 'apps/sdkwork-drive-pc', 'dev'],
    command,
    cwd: repoRoot,
    env,
    label: 'drive-pc-browser',
    shell: true,
  };
}

function prefixOutput(label, stream, chunk) {
  const text = String(chunk ?? '');
  for (const line of text.split(/\r?\n/u)) {
    if (line.length > 0) {
      stream.write(`[${label}] ${line}\n`);
    }
  }
}

function spawnProcessEntry(entry, { onOutput = prefixOutput } = {}) {
  const child = spawn(entry.command, entry.args, {
    cwd: entry.cwd,
    env: entry.env,
    shell: entry.shell,
    stdio: ['ignore', 'pipe', 'pipe'],
  });

  child.stdout?.on('data', (chunk) => onOutput(entry.label, process.stdout, chunk));
  child.stderr?.on('data', (chunk) => onOutput(entry.label, process.stderr, chunk));
  return child;
}

async function ensureDevGatewayHealthy({
  gatewayEntry,
  gatewayUrl,
  iamDatabaseTarget,
}) {
  const alreadyHealthy = await checkGatewayHealth(gatewayUrl);
  if (alreadyHealthy) {
    console.log(`[sdkwork-drive] dev gateway already healthy at ${gatewayUrl}`);
    return null;
  }

  if (!gatewayEntry) {
    throw new Error(
      `Dev gateway is not healthy at ${gatewayUrl} and autostart is disabled.\n`
        + 'Start the dev gateway manually or set SDKWORK_DRIVE_GATEWAY_AUTOSTART=false.',
    );
  }

  console.log(
    `[sdkwork-drive] starting sdkwork-api-drive-standalone-gateway (IAM database: ${iamDatabaseTarget})...`,
  );
  const child = spawnProcessEntry(gatewayEntry);
  let exitCode = null;

  child.on('exit', (code) => {
    if (code && code !== 0) {
      exitCode = code;
    }
  });

  console.log(`[sdkwork-drive] waiting for dev gateway at ${gatewayUrl}...`);
  for (let attempt = 1; attempt <= GATEWAY_MAX_STARTUP_ATTEMPTS; attempt += 1) {
    if (exitCode != null) {
      terminateProcessTree(child);
      throw formatGatewayStartupError(exitCode, iamDatabaseTarget);
    }

    const healthy = await checkGatewayHealth(gatewayUrl);
    if (healthy) {
      console.log(`[sdkwork-drive] dev gateway is healthy (attempt ${attempt})`);
      return child;
    }

    if (attempt < GATEWAY_MAX_STARTUP_ATTEMPTS) {
      await delay(GATEWAY_STARTUP_WAIT_MS);
    }
  }

  terminateProcessTree(child);
  throw formatGatewayStartupError(exitCode, iamDatabaseTarget);
}

function terminateProcessTree(child) {
  if (!child?.pid) {
    return;
  }

  if (process.platform === 'win32') {
    spawnSync('taskkill.exe', ['/PID', String(child.pid), '/T', '/F'], {
      stdio: 'ignore',
      windowsHide: true,
    });
    return;
  }

  child.kill();
}

function printHelp() {
  console.log(`Usage: node scripts/drive-dev.mjs [options]

Options:
  --target <browser|desktop>       Target platform (default: browser)
  --deployment-profile <standalone|cloud> Deployment profile (default: standalone)
  --dev-env-file <path>            Path to env file
  --dry-run                        Print plan without executing
  --client-only                    Start only the selected local client
  --help, -h                       Show this help

Deployment policy (standalone default for dev):
  Profiles load from etc/topology according to deployment profile and environment.
  Standalone dev uses the local Drive standalone gateway on ${DEFAULT_SDKWORK_API_CLOUD_GATEWAY_BIND}.
  Cloud dev consumes the deployed platform.api-gateway surface and starts no local API process.
  IAM login requires PostgreSQL (copy .env.postgres.example to .env.postgres and start PostgreSQL).
  Vite/desktop starts only after the gateway health check passes.
  Set SDKWORK_DRIVE_GATEWAY_AUTOSTART=false to skip gateway autostart.

Desktop packaging:
  pnpm build defaults to the cloud production profile.
  pnpm build:standalone targets local application ingress URLs.

Desktop dev port policy:
  Desktop dev reclaims ${DRIVE_PC_DEV_PORT} from stale Vite processes by default.
  Set SDKWORK_DRIVE_PC_DEV_RECLAIM_PORT=false to disable automatic cleanup.
`);
}

async function main() {
  try {
    const settings = parseArgs(process.argv.slice(2));
    if (settings.help) {
      printHelp();
      process.exit(0);
    }

    const postgresEnvFile = '.env.postgres';
    const envFile = settings.devEnvFile || postgresEnvFile;
    const profileId = resolveDevProfileId(settings.deploymentProfile);
    const profileEnv = loadProfile(profileId);
    const postgresEnv = loadEnvFile(postgresEnvFile);
    const fileEnv = loadEnvFile(envFile);
    const baseEnv = mergeRuntimeEnv(process.env, profileEnv, postgresEnv, fileEnv, {
      SDKWORK_DRIVE_DEPLOYMENT_PROFILE: settings.deploymentProfile,
      VITE_DRIVE_PC_DEPLOYMENT_PROFILE: settings.deploymentProfile,
      SDKWORK_DRIVE_PROFILE_ID: profileId,
      ...IAM_APPLICATION_BOOTSTRAP_ENV,
    });

    if (!settings.dryRun) {
      await ensureDrivePcDevPortAvailable(baseEnv, settings.target);
    }

    const gatewayWillStart = settings.clientOnly
      ? false
      : await resolveGatewayAutostart(baseEnv);
    const { appApiBaseUrl, apiGatewayBaseUrl, adminStorageApiBaseUrl } =
      await resolveDevApiBaseUrls(baseEnv, gatewayWillStart);

    const gatewayProcess = settings.clientOnly
      ? undefined
      : createManagedGatewayProcess({
        env: baseEnv,
        gatewayWillStart,
        deploymentProfile: settings.deploymentProfile,
      });
    const iamDevEnv = resolveIamDevEnv(baseEnv, repoRoot);
    const iamDatabaseTarget = describeIamDatabaseTarget(iamDevEnv);
    const devServerProcess = createDevServerProcess({
      env: mergeRepoDevBootstrapAccessTokenEnv({
        repoRoot,
        manifestPath: 'apps/sdkwork-drive-pc/sdkwork.app.config.json',
        env: {
          ...baseEnv,
          VITE_DRIVE_PC_DEV_SAME_ORIGIN_API: 'true',
          SDKWORK_DRIVE_DEV_APP_API_PROXY_TARGET: apiGatewayBaseUrl,
          SDKWORK_DRIVE_DEV_ADMIN_API_PROXY_TARGET: adminStorageApiBaseUrl,
          VITE_DRIVE_PC_PLATFORM_API_GATEWAY_HTTP_URL: apiGatewayBaseUrl,
          VITE_DRIVE_PC_APP_API_BASE_URL: appApiBaseUrl,
          VITE_DRIVE_PC_APPBASE_APP_API_BASE_URL: apiGatewayBaseUrl,
          VITE_DRIVE_PC_DRIVE_APP_API_BASE_URL: appApiBaseUrl,
          VITE_DRIVE_PC_BACKEND_API_BASE_URL: adminStorageApiBaseUrl,
          VITE_DRIVE_PC_DRIVE_ADMIN_STORAGE_API_BASE_URL: adminStorageApiBaseUrl,
        },
      }),
      target: settings.target,
    });

    const processes = [];
    if (gatewayProcess) {
      processes.push(gatewayProcess);
    }
    processes.push(devServerProcess);

    if (settings.dryRun) {
      console.log(`[sdkwork-drive] deploymentProfile=${settings.deploymentProfile}`);
      console.log('[sdkwork-drive] environment=development');
      console.log(`[sdkwork-drive] SDKWORK_DRIVE_PROFILE_ID=${profileId}`);
      console.log(`[sdkwork-drive] IAM database target: ${iamDatabaseTarget}`);
      for (const entry of processes) {
        console.log(`[${entry.label}] ${entry.command} ${entry.args.join(' ')}`);
      }
      process.exit(0);
    }

    const children = [];
    let shuttingDown = false;

    function shutdown(exceptChild) {
      if (shuttingDown) {
        return;
      }
      shuttingDown = true;
      for (const child of children) {
        if (child !== exceptChild && child.exitCode == null && child.signalCode == null) {
          terminateProcessTree(child);
        }
      }
    }

    function attachProcessLifecycle(entry, child) {
      child.on('error', (error) => {
        process.stderr.write(
          `[${entry.label}] ${error instanceof Error ? error.message : String(error)}\n`,
        );
        shutdown(child);
        process.exitCode = 1;
      });
      child.on('exit', (code, signal) => {
        if (shuttingDown) {
          return;
        }
        shutdown(child);
        if (code && code !== 0) {
          process.stderr.write(`[${entry.label}] exited with code ${code}\n`);
          process.exitCode = code;
          return;
        }
        if (signal) {
          process.stderr.write(`[${entry.label}] exited with signal ${signal}\n`);
          process.exitCode = 1;
        }
      });
    }

    if (settings.clientOnly) {
      const child = spawnProcessEntry(devServerProcess);
      children.push(child);
      attachProcessLifecycle(devServerProcess, child);
      const stop = () => shutdown();
      process.once('SIGINT', stop);
      process.once('SIGTERM', stop);
      return;
    }

    const needsGateway = shouldAutostartSdkworkApiGateway(baseEnv)
      || !(await checkGatewayHealth(apiGatewayBaseUrl));
    if (needsGateway) {
      await assertPostgresReachableForIam(iamDevEnv);
    }

    const gatewayChild = await ensureDevGatewayHealthy({
      gatewayEntry: gatewayProcess,
      gatewayUrl: apiGatewayBaseUrl,
      iamDatabaseTarget,
    });
    if (gatewayChild) {
      children.push(gatewayChild);
      attachProcessLifecycle(gatewayProcess, gatewayChild);
    }

    for (const entry of [devServerProcess]) {
      const child = spawnProcessEntry(entry);
      children.push(child);
      attachProcessLifecycle(entry, child);
    }

    const stop = () => shutdown();
    process.once('SIGINT', stop);
    process.once('SIGTERM', stop);
  } catch (error) {
    console.error(`[sdkwork-drive] ${error.message}`);
    process.exit(1);
  }
}

main();
