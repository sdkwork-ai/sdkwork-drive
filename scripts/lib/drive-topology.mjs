import { existsSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  buildProfileId,
  createTopologyRuntime,
  isTcpPortReachable,
  loadTopologySpec,
  normalizeText,
} from '@sdkwork/app-topology';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export const REPO_ROOT = path.resolve(__dirname, '..', '..');
export const SPEC_PATH = path.join(REPO_ROOT, 'specs/topology.spec.json');
export const IAM_REPO_ROOT = path.resolve(REPO_ROOT, '..', 'sdkwork-iam');

const spec = loadTopologySpec(SPEC_PATH);
const runtime = createTopologyRuntime(spec, REPO_ROOT);

export const VALID_DEPLOYMENT_PROFILES = runtime.deploymentProfileValues;
export const VALID_ENVIRONMENTS = runtime.environmentValues;
export const DEFAULT_DEV_PROFILE_ID = runtime.defaults.developmentProfileId;
export const DEFAULT_BUILD_PROFILE_ID = runtime.defaults.productionProfileId;
export const DEFAULT_STANDALONE_BUILD_PROFILE_ID = 'standalone.production';
export const DEFAULT_GATEWAY_BIND = runtime.defaults.gatewayBind;
export const POSTGRES_REACHABILITY_TIMEOUT_MS = 2000;

export const APPLICATION_PUBLIC_INGRESS_PACKAGE_PROFILE = 'application-public-ingress';
export const PLATFORM_CONFIG_BUNDLE_PROFILE = 'platform-config-bundle';

export const GATEWAY_PACKAGE_TARGETS = runtime.listPackageTargets();
export const APPLICATION_PUBLIC_INGRESS_PACKAGE_TARGETS = runtime.listPackageTargetsByProfile(
  APPLICATION_PUBLIC_INGRESS_PACKAGE_PROFILE,
);
export const PLATFORM_CONFIG_BUNDLE_TARGET = runtime.findPackageTarget('platform-config-bundle-tar-gz');
export const DRIVE_CLOUD_GATEWAY_CONFIGS = spec.packaging?.cloudConfigFiles ?? [];

export const IAM_APPLICATION_BOOTSTRAP_ENV = {
  SDKWORK_APP_ROOT: REPO_ROOT,
  SDKWORK_DRIVE_APP_ROOT: REPO_ROOT,
  SDKWORK_IAM_APP_ROOT: IAM_REPO_ROOT,
};

const TEMPORARY_DATABASE_DRIVER_ENV_KEYS = [
  'SDKWORK_DATABASE_TEMPORARY_ANY_POOL_EXCEPTION',
  'SDKWORK_DATABASE_TEMPORARY_DRIVER_POOL_COUNT',
];

export function resolveDevProfileId(deploymentProfile) {
  runtime.assertDeploymentProfile(deploymentProfile);
  return buildProfileId(deploymentProfile, 'development');
}

export function resolveBuildProfileId(deploymentProfile) {
  runtime.assertDeploymentProfile(deploymentProfile);
  if (deploymentProfile === 'standalone') {
    return DEFAULT_STANDALONE_BUILD_PROFILE_ID;
  }
  return DEFAULT_BUILD_PROFILE_ID;
}

export const loadEnvFile = runtime.loadEnvFile;
export const loadProfile = runtime.loadProfile;
export const applyProfileEnv = runtime.applyProfileEnv;
export const mergeRuntimeEnv = runtime.mergeRuntimeEnv;
export const assertDeploymentProfile = runtime.assertDeploymentProfile;
export const assertProfileId = runtime.assertProfileId;
export const profilePath = runtime.profilePath;
export const shouldAutostartGateway = runtime.shouldAutostartGateway;
export const resolveGatewayBind = runtime.resolveGatewayBind;
export const resolveGatewayBaseUrl = runtime.resolveGatewayBaseUrl;
export const resolveIamDatabaseEnv = runtime.resolveIamDatabaseEnv;
export const resolveIamDevEnv = runtime.resolveIamDevEnv;
export const describeIamDatabaseTarget = runtime.describeIamDatabaseTarget;
export const assertPostgresReachableForIam = runtime.assertPostgresReachableForIam;
export const resolveCloudGatewayConfigPath = runtime.resolveCloudGatewayConfigPath;
export const resolveSurfaceHttpUrl = runtime.resolveSurfaceHttpUrl.bind(runtime);
export const resolveSurfaceBind = runtime.resolveSurfaceBind.bind(runtime);

export function resolveTemporaryDatabaseDriverEnv(env = process.env) {
  return Object.fromEntries(
    TEMPORARY_DATABASE_DRIVER_ENV_KEYS.flatMap((key) => {
      const value = normalizeText(env[key]);
      return value ? [[key, value]] : [];
    }),
  );
}

export function resolveStandaloneGatewayConfigPath(env = process.env) {
  const configPath = runtime.resolveStandaloneGatewayConfigPath(env, REPO_ROOT);
  const developmentExamplePath = `${configPath}.example`;
  if (!existsSync(configPath) && existsSync(developmentExamplePath)) {
    return developmentExamplePath;
  }
  return configPath;
}

export function findGatewayPackageTarget(targetId) {
  return runtime.findPackageTarget(targetId);
}

export function listGatewayPackageTargets(profile) {
  return runtime.listPackageTargetsByProfile(profile);
}

export { buildProfileId, normalizeText, isTcpPortReachable, spec, runtime };
