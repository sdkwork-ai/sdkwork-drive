#!/usr/bin/env node
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdir, mkdtemp, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const toolPath = path.join(repoRoot, 'tools/check_drive_deployments.mjs');

function redisRateLimitEnvBlock() {
  return `            - name: SDKWORK_DRIVE_RATE_LIMIT_BACKEND
              value: redis
            - name: SDKWORK_DRIVE_RATE_LIMIT_REDIS_URL
              valueFrom:
                secretKeyRef:
                  name: sdkwork-drive-rate-limit
                  key: SDKWORK_DRIVE_RATE_LIMIT_REDIS_URL
            - name: SDKWORK_DRIVE_RATE_LIMIT_FAIL_CLOSED
              value: "true"
`;
}

function deploymentManifest(imageRefs, options = {}) {
  const includeRedisRateLimit = options.includeRedisRateLimit !== false;
  const redisRateLimit = includeRedisRateLimit ? redisRateLimitEnvBlock() : '';
  const imageByName = Object.fromEntries(imageRefs.map((imageRef) => [imageRef.name, imageRef.image]));
  return `apiVersion: apps/v1
kind: Deployment
metadata:
  name: sdkwork-drive-app-api
spec:
  template:
    spec:
      containers:
        - name: app-api
          image: ${imageByName['sdkwork-drive-app-api']}
          env:
            - name: OTEL_EXPORTER_OTLP_ENDPOINT
              value: http://otel:4318/v1/traces
            - name: OTEL_SERVICE_NAME
              value: sdkwork-drive-app-api
            - name: SDKWORK_DRIVE_APP_API_RATE_LIMIT_MAX_REQUESTS
              value: "600"
${redisRateLimit}            - name: SDKWORK_DRIVE_UPLOAD_CONTENT_POLICY_MODE
              value: enforce
          envFrom:
            - secretRef:
                name: sdkwork-drive-iam
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sdkwork-drive-backend-api
spec:
  template:
    spec:
      containers:
        - name: backend-api
          image: ${imageByName['sdkwork-drive-backend-api']}
          env:
            - name: OTEL_EXPORTER_OTLP_ENDPOINT
              value: http://otel:4318/v1/traces
            - name: OTEL_SERVICE_NAME
              value: sdkwork-drive-backend-api
            - name: SDKWORK_DRIVE_BACKEND_API_RATE_LIMIT_MAX_REQUESTS
              value: "300"
${redisRateLimit}          envFrom:
            - secretRef:
                name: sdkwork-drive-iam
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sdkwork-drive-open-api
spec:
  template:
    spec:
      containers:
        - name: open-api
          image: ${imageByName['sdkwork-drive-open-api']}
          env:
            - name: OTEL_EXPORTER_OTLP_ENDPOINT
              value: http://otel:4318/v1/traces
            - name: OTEL_SERVICE_NAME
              value: sdkwork-drive-open-api
            - name: SDKWORK_DRIVE_OPEN_API_RATE_LIMIT_MAX_REQUESTS
              value: "120"
${redisRateLimit}          envFrom:
            - secretRef:
                name: sdkwork-drive-iam
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sdkwork-drive-admin-storage-api
spec:
  template:
    spec:
      containers:
        - name: admin-storage-api
          image: ${imageByName['sdkwork-drive-admin-storage-api']}
          env:
            - name: SDKWORK_DRIVE_ADMIN_STORAGE_API_RATE_LIMIT_MAX_REQUESTS
              value: "300"
${redisRateLimit}          envFrom:
            - secretRef:
                name: sdkwork-drive-iam
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sdkwork-drive-install-worker
spec:
  template:
    spec:
      containers:
        - name: install-worker
          image: ${imageByName['sdkwork-drive-install-worker']}
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sdkwork-api-drive-standalone-gateway
spec:
  template:
    spec:
      containers:
        - name: standalone-gateway
          image: ${imageByName['sdkwork-api-drive-standalone-gateway']}
---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: sdkwork-drive
  annotations:
    nginx.ingress.kubernetes.io/limit-rps: "120"
spec: {}
`;
}

function legacyDeploymentManifestWithoutRedis(imageRefs) {
  const imageByName = Object.fromEntries(imageRefs.map((imageRef) => [imageRef.name, imageRef.image]));
  return `apiVersion: apps/v1
kind: Deployment
metadata:
  name: sdkwork-drive-app-api
spec:
  template:
    spec:
      containers:
        - name: app-api
          image: ${imageByName['sdkwork-drive-app-api']}
          env:
            - name: OTEL_EXPORTER_OTLP_ENDPOINT
              value: http://otel:4318/v1/traces
            - name: OTEL_SERVICE_NAME
              value: sdkwork-drive-app-api
            - name: SDKWORK_DRIVE_APP_API_RATE_LIMIT_MAX_REQUESTS
              value: "600"
            - name: SDKWORK_DRIVE_UPLOAD_CONTENT_POLICY_MODE
              value: enforce
          envFrom:
            - secretRef:
                name: sdkwork-drive-iam
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sdkwork-drive-backend-api
spec:
  template:
    spec:
      containers:
        - name: backend-api
          image: ${imageByName['sdkwork-drive-backend-api']}
          env:
            - name: OTEL_EXPORTER_OTLP_ENDPOINT
              value: http://otel:4318/v1/traces
            - name: OTEL_SERVICE_NAME
              value: sdkwork-drive-backend-api
            - name: SDKWORK_DRIVE_BACKEND_API_RATE_LIMIT_MAX_REQUESTS
              value: "300"
          envFrom:
            - secretRef:
                name: sdkwork-drive-iam
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sdkwork-drive-open-api
spec:
  template:
    spec:
      containers:
        - name: open-api
          image: ${imageByName['sdkwork-drive-open-api']}
          env:
            - name: OTEL_EXPORTER_OTLP_ENDPOINT
              value: http://otel:4318/v1/traces
            - name: OTEL_SERVICE_NAME
              value: sdkwork-drive-open-api
            - name: SDKWORK_DRIVE_OPEN_API_RATE_LIMIT_MAX_REQUESTS
              value: "120"
          envFrom:
            - secretRef:
                name: sdkwork-drive-iam
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sdkwork-drive-admin-storage-api
spec:
  template:
    spec:
      containers:
        - name: admin-storage-api
          image: ${imageByName['sdkwork-drive-admin-storage-api']}
          env:
            - name: SDKWORK_DRIVE_ADMIN_STORAGE_API_RATE_LIMIT_MAX_REQUESTS
              value: "300"
          envFrom:
            - secretRef:
                name: sdkwork-drive-iam
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sdkwork-drive-install-worker
spec:
  template:
    spec:
      containers:
        - name: install-worker
          image: ${imageByName['sdkwork-drive-install-worker']}
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sdkwork-api-drive-standalone-gateway
spec:
  template:
    spec:
      containers:
        - name: standalone-gateway
          image: ${imageByName['sdkwork-api-drive-standalone-gateway']}
---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: sdkwork-drive
  annotations:
    nginx.ingress.kubernetes.io/limit-rps: "120"
spec: {}
`;
}

function placeholderImages() {
  return [
    'sdkwork-drive-app-api',
    'sdkwork-drive-backend-api',
    'sdkwork-drive-open-api',
    'sdkwork-drive-admin-storage-api',
    'sdkwork-drive-install-worker',
    'sdkwork-api-drive-standalone-gateway',
  ].map((name) => ({
    name,
    image: `registry.sdkwork.com/apps/sdkwork-drive/${name.replace(/^sdkwork-drive-/, '')}@sha256:REPLACE_WITH_RELEASE_DIGEST`,
  }));
}

function realDigestImages() {
  return placeholderImages().map((entry, index) => ({
    ...entry,
    image: entry.image.replace(
      'REPLACE_WITH_RELEASE_DIGEST',
      String(index + 1).repeat(64),
    ),
  }));
}

async function createTempWorkspace(imageRefs, options = {}) {
  const tempRoot = await mkdtemp(path.join(os.tmpdir(), 'sdkwork-drive-deploy-'));
  const appId = path.basename(tempRoot);
  await mkdir(path.join(tempRoot, 'deployments/kubernetes'), { recursive: true });
  await mkdir(path.join(tempRoot, 'deployments/nginx'), { recursive: true });
  await mkdir(path.join(tempRoot, 'deployments/docker'), { recursive: true });
  await mkdir(path.join(tempRoot, 'deployments/container'), { recursive: true });
  await mkdir(path.join(tempRoot, 'deployments/systemd'), { recursive: true });
  await mkdir(path.join(tempRoot, 'docs/runbooks'), { recursive: true });
  await mkdir(path.join(tempRoot, 'docs/guides/operator'), { recursive: true });
  await mkdir(path.join(tempRoot, `apps/${appId}-pc`), { recursive: true });
  await mkdir(path.join(tempRoot, 'etc/topology'), { recursive: true });
  await mkdir(path.join(tempRoot, 'specs'), { recursive: true });
  await mkdir(path.join(tempRoot, 'node_modules/yaml'), { recursive: true });

  await writeFile(
    path.join(tempRoot, 'deployments/deploy.yaml'),
    options.deployYaml ??
      `${JSON.stringify({
        version: 1,
        defaultProfile: 'cloud.production',
        profiles: {
          'cloud.production': {
            install: {
              layout: 'binary-package',
            },
            expose: [
              {
                domain: 'drive.example.com',
                tls: 'managed',
                mode: 'web+api',
                web: 'pc',
                apiPathStyle: 'full-prefix',
              },
            ],
            packages: [],
            overrides: {
              topology: {
                spec: 'specs/topology.spec.json',
                profile: 'cloud.production',
                env: 'etc/topology/cloud.production.env',
              },
              proxy: {
                upstreams: {
                  'application.public-ingress': 'http://127.0.0.1:3900',
                },
              },
            },
          },
        },
      }, null, 2)}\n`,
    'utf8',
  );
  await writeFile(
    path.join(tempRoot, 'etc/topology/cloud.production.env'),
    'SDKWORK_DRIVE_PROFILE_ID=cloud.production\n',
    'utf8',
  );
  await writeFile(
    path.join(tempRoot, 'node_modules/yaml/package.json'),
    `${JSON.stringify({ name: 'yaml', main: 'index.js' }, null, 2)}\n`,
    'utf8',
  );
  await writeFile(
    path.join(tempRoot, 'node_modules/yaml/index.js'),
    'exports.parse = JSON.parse;\nexports.stringify = (value) => JSON.stringify(value, null, 2);\n',
    'utf8',
  );
  await writeFile(
    path.join(tempRoot, 'specs/topology.spec.json'),
    `${JSON.stringify({
      appId,
      database: { appPrefix: 'SDKWORK_DRIVE' },
      profileFiles: {
        'cloud.production': 'etc/topology/cloud.production.env',
      },
      surfaces: {
        'application.public-ingress': {
          bindEnv: 'SDKWORK_DRIVE_APPLICATION_PUBLIC_INGRESS_BIND',
        },
      },
      defaults: {
        gatewayBind: '127.0.0.1:3900',
      },
    }, null, 2)}\n`,
    'utf8',
  );
  await writeFile(
    path.join(tempRoot, 'sdkwork.app.config.json'),
    `${JSON.stringify({ schemaVersion: 3, kind: 'sdkwork.app' }, null, 2)}\n`,
    'utf8',
  );
  await writeFile(
    path.join(tempRoot, 'deployments/kubernetes/drive-services.yaml'),
    options.legacyWithoutRedis === true
      ? legacyDeploymentManifestWithoutRedis(imageRefs)
      : deploymentManifest(imageRefs, options),
    'utf8',
  );

  await writeFile(path.join(tempRoot, 'deployments/nginx/drive-edge-rate-limit.conf.example'), '# rate limit\n', 'utf8');
  await writeFile(path.join(tempRoot, 'deployments/docker-compose.minio-test.yml'), 'services: {}\n', 'utf8');
  await writeFile(path.join(tempRoot, 'deployments/docker/Dockerfile.app-api'), 'FROM scratch\n', 'utf8');
  await writeFile(path.join(tempRoot, 'deployments/container/README.md'), '# Container\n', 'utf8');
  await writeFile(path.join(tempRoot, 'docs/runbooks/drive-production-operations.md'), '# Operations\n', 'utf8');
  await writeFile(path.join(tempRoot, 'docs/runbooks/drive-backup-disaster-recovery.md'), '# DR\n', 'utf8');
  await writeFile(path.join(tempRoot, 'docs/guides/operator/pre-launch-checklist.md'), '# Checklist\n', 'utf8');

  for (const service of [
    'sdkwork-drive-app-api.service',
    'sdkwork-drive-backend-api.service',
    'sdkwork-drive-open-api.service',
    'sdkwork-drive-admin-storage-api.service',
    'sdkwork-drive-install-worker.service',
    'sdkwork-api-drive-standalone-gateway.service',
  ]) {
    await writeFile(
      path.join(tempRoot, 'deployments/systemd', service),
      '[Service]\nEnvironment=SDKWORK_DRIVE_DEPLOYMENT_PROFILE=cloud\n',
      'utf8',
    );
  }

  return tempRoot;
}

function runDeployValidate(tempRoot, env = {}) {
  return spawnSync(process.execPath, [toolPath, '--root', tempRoot], {
    cwd: tempRoot,
    env: {
      ...process.env,
      ...env,
    },
    encoding: 'utf8',
  });
}

{
  const tempRoot = await createTempWorkspace(placeholderImages());
  const result = runDeployValidate(tempRoot);
  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.match(result.stderr, /REPLACE_WITH_RELEASE_DIGEST/);
  assert.match(result.stdout, /deployment descriptors ok/);
}

{
  const tempRoot = await createTempWorkspace(placeholderImages());
  const result = runDeployValidate(tempRoot, { SDKWORK_DEPLOY_VALIDATION: 'strict' });
  assert.notEqual(result.status, 0, result.stdout);
  assert.match(result.stderr, /REPLACE_WITH_RELEASE_DIGEST/);
  assert.match(result.stderr, /strict deployment validation/);
}

{
  const tempRoot = await createTempWorkspace(placeholderImages());
  const result = runDeployValidate(tempRoot, { SDKWORK_RELEASE_VALIDATION: 'strict' });
  assert.notEqual(result.status, 0, result.stdout);
  assert.match(result.stderr, /REPLACE_WITH_RELEASE_DIGEST/);
  assert.match(result.stderr, /strict deployment validation/);
}

{
  const tempRoot = await createTempWorkspace(realDigestImages());
  const result = runDeployValidate(tempRoot, { SDKWORK_DEPLOY_VALIDATION: 'strict' });
  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.match(result.stdout, /deployment descriptors ok/);
}

{
  const tempRoot = await createTempWorkspace(realDigestImages(), { legacyWithoutRedis: true });
  const result = runDeployValidate(tempRoot);
  assert.notEqual(result.status, 0, result.stdout);
  assert.match(result.stderr, /SDKWORK_DRIVE_RATE_LIMIT_BACKEND=redis/);
  assert.match(result.stderr, /SDKWORK_DRIVE_RATE_LIMIT_FAIL_CLOSED=true/);
  assert.match(result.stderr, /sdkwork-drive-rate-limit/);
}

{
  const tempRoot = await createTempWorkspace(realDigestImages(), {
    deployYaml: `${JSON.stringify({
      version: 1,
      defaultProfile: 'cloud.split-services.production',
      profiles: {
        'cloud.split-services.production': {
          install: { layout: 'binary-package' },
          expose: [
            {
              domain: 'drive.example.com',
              tls: 'managed',
              mode: 'web+api',
              web: 'pc',
              apiPathStyle: 'full-prefix',
            },
          ],
          packages: [],
          overrides: {
            topology: {
              spec: 'specs/topology.spec.json',
              profile: 'cloud.split-services.production',
              env: 'etc/topology/cloud.split-services.production.env',
            },
            proxy: {
              upstreams: {
                'application.public-ingress': 'http://127.0.0.1:3900',
              },
            },
          },
        },
      },
    }, null, 2)}\n`,
  });
  const result = runDeployValidate(tempRoot);
  assert.notEqual(result.status, 0, result.stdout);
  assert.match(result.stderr, /profile "cloud\.split-services\.production" is not listed in topology\.profileFiles/);
  assert.match(result.stderr, /etc\/topology\/cloud\.split-services\.production\.env does not exist/);
}

console.log('check_drive_deployments.test.mjs passed');
