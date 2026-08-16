import { existsSync, readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const DRIVE_PACKAGE = "@sdkwork/drive-app-sdk";
const DRIVE_PACKAGE_WORKSPACE_SEGMENTS = [
  "sdkwork-drive/sdks/sdkwork-drive-app-sdk/sdkwork-drive-app-sdk-typescript",
  "sdks/sdkwork-drive-app-sdk/sdkwork-drive-app-sdk-typescript",
];

const DEFAULT_WORKSPACE_ROOT = path.resolve(process.cwd(), "..");
const DEFAULT_DRIVE_ROOT = process.cwd();

const ignoredDirectoryNames = new Set([
  ".git",
  ".turbo",
  "backup",
  "bak",
  "dist",
  "external",
  "generated",
  "node_modules",
  "out",
  "sdks",
  "target",
]);

const forbiddenUploadPatterns = [
  {
    name: "legacy app-sdk presign flow",
    pattern: /\bclient\.upload\.getPresignedUrl\b|\bgetPresignedUrl\b|\bregisterPresigned\b|\bPresignedUrlForm\b|\bPresignedUploadRegisterForm\b/,
  },
  {
    name: "raw Drive uploader HTTP route",
    pattern: /\/app\/v3\/api\/drive\/(?:uploader|upload_sessions)\b|\/drive\/(?:uploader|upload_sessions)\b/,
  },
  {
    name: "product local presign/upload-session facade",
    pattern: /\bpresignUploadPart\b|\bcreateUploadSession\b|\bcompleteUploadSession\b/,
  },
];

const driveConsumerPatterns = [
  {
    name: "Drive app SDK package import",
    pattern: /@sdkwork\/drive-app-sdk\b|sdkwork-drive-app-sdk\b/,
  },
  {
    name: "Drive uploader client surface",
    pattern: /\bcreateDriveUploaderClient\b|\bclient\.uploader\.|\buploader\.upload(?:ByProfile|Video|Image|Audio|Document|Archive|Text|Dataset|Attachment|Avatar|Thumbnail)?\b/,
  },
];

function readJson(filePath) {
  return JSON.parse(readFileSync(filePath, "utf8").replace(/^\uFEFF/u, ""));
}

function readTextIfExists(filePath) {
  return existsSync(filePath) ? readFileSync(filePath, "utf8") : "";
}

function toPosix(relativePath) {
  return relativePath.split(path.sep).join("/");
}

function relativeFromWorkspace(workspaceRoot, absolutePath) {
  return toPosix(path.relative(workspaceRoot, absolutePath));
}

function walk(directory, visitor) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    if (ignoredDirectoryNames.has(entry.name)) {
      continue;
    }

    const absolutePath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      walk(absolutePath, visitor);
      continue;
    }
    visitor(absolutePath);
  }
}

function findAppManifestPaths(workspaceRoot) {
  const manifestPaths = [];
  walk(workspaceRoot, (absolutePath) => {
    if (path.basename(absolutePath) === "sdkwork.app.config.json") {
      manifestPaths.push(absolutePath);
    }
  });
  return manifestPaths.sort((left, right) =>
    relativeFromWorkspace(workspaceRoot, left).localeCompare(relativeFromWorkspace(workspaceRoot, right)),
  );
}

function packageJsonPaths(appRoot) {
  const paths = [];
  walk(appRoot, (absolutePath) => {
    if (path.basename(absolutePath) === "package.json") {
      paths.push(absolutePath);
    }
  });
  return paths;
}

function declaredDependency(packageJson) {
  return [
    packageJson.dependencies,
    packageJson.devDependencies,
    packageJson.peerDependencies,
    packageJson.optionalDependencies,
  ].some((dependencies) => dependencies && Object.hasOwn(dependencies, DRIVE_PACKAGE));
}

function hasDriveDependency(appRoot) {
  return packageJsonPaths(appRoot).some((packagePath) => declaredDependency(readJson(packagePath)));
}

function hasDriveWorkspaceLink(appRoot) {
  const workspaceFile = path.join(appRoot, "pnpm-workspace.yaml");
  const workspaceText = readTextIfExists(workspaceFile).replaceAll("\\", "/");
  return DRIVE_PACKAGE_WORKSPACE_SEGMENTS.some((segment) => workspaceText.includes(segment));
}

function sourcePaths(appRoot) {
  const paths = [];
  walk(appRoot, (absolutePath) => {
    if (/\.(?:ts|tsx|js|jsx|mts|mjs)$/.test(absolutePath)) {
      paths.push(absolutePath);
    }
  });
  return paths;
}

function sourceFindings(appRoot, patterns, workspaceRoot) {
  const findings = [];
  for (const sourcePath of sourcePaths(appRoot)) {
    const text = readFileSync(sourcePath, "utf8");
    for (const { name, pattern } of patterns) {
      if (pattern.test(text)) {
        findings.push({
          relativePath: relativeFromWorkspace(workspaceRoot, sourcePath),
          name,
        });
      }
    }
  }
  return findings;
}

function forbiddenUploadUsages(appRoot, workspaceRoot) {
  return sourceFindings(appRoot, forbiddenUploadPatterns, workspaceRoot);
}

function driveConsumerUsages(appRoot, workspaceRoot) {
  return sourceFindings(appRoot, driveConsumerPatterns, workspaceRoot);
}

function isExcludedApp(appRoot, workspaceRoot) {
  const relativeRoot = relativeFromWorkspace(workspaceRoot, appRoot);
  return (
    relativeRoot === "" ||
    relativeRoot === "sdkwork-drive" ||
    relativeRoot.startsWith("sdkwork-drive/") ||
    relativeRoot === "sdkwork-appbase" ||
    relativeRoot.startsWith("sdkwork-appbase/")
  );
}

function classifyApp(manifestPath, workspaceRoot) {
  const appRoot = path.dirname(manifestPath);
  const manifest = readJson(manifestPath);
  const app = manifest.app ?? manifest;
  return {
    appRoot,
    manifest,
    key: app.key ?? app.id ?? app.name ?? relativeFromWorkspace(workspaceRoot, appRoot),
    framework: app.framework ?? manifest.framework ?? "unknown",
    family: app.family ?? app.surface ?? manifest.family ?? "unknown",
    relativeRoot: relativeFromWorkspace(workspaceRoot, appRoot),
  };
}

function resolveSpecsRoot(workspaceRoot, driveRoot) {
  const candidates = [
    path.join(workspaceRoot, "sdkwork-specs"),
    path.join(path.dirname(driveRoot), "sdkwork-specs"),
  ];
  for (const candidate of candidates) {
    if (existsSync(path.join(candidate, "DRIVE_SPEC.md"))) {
      return candidate;
    }
  }
  return null;
}

export function analyzeDriveAppSdkConsumerIntegration({
  workspaceRoot = DEFAULT_WORKSPACE_ROOT,
  driveRoot = DEFAULT_DRIVE_ROOT,
} = {}) {
  const specsRoot = resolveSpecsRoot(workspaceRoot, driveRoot);
  if (!specsRoot) {
    throw new Error(`Cannot resolve SDKWork specs from ${workspaceRoot} or ${path.dirname(driveRoot)}.`);
  }

  if (!existsSync(path.join(driveRoot, "sdks", "sdkwork-drive-app-sdk"))) {
    throw new Error(`Run this check from sdkwork-drive root, got ${driveRoot}.`);
  }

  const apps = findAppManifestPaths(workspaceRoot)
    .map((manifestPath) => classifyApp(manifestPath, workspaceRoot))
    .filter((app) => !isExcludedApp(app.appRoot, workspaceRoot));

  const failures = [];
  for (const app of apps) {
    const consumerEvidence = driveConsumerUsages(app.appRoot, workspaceRoot);
    const forbidden = forbiddenUploadUsages(app.appRoot, workspaceRoot);
    if (consumerEvidence.length === 0 && forbidden.length === 0) {
      continue;
    }

    const missingDependency = !hasDriveDependency(app.appRoot);
    const missingWorkspaceLink = existsSync(path.join(app.appRoot, "pnpm-workspace.yaml")) &&
      !hasDriveWorkspaceLink(app.appRoot);

    if (missingDependency || missingWorkspaceLink || forbidden.length > 0) {
      failures.push({
        app,
        consumerEvidence,
        missingDependency,
        missingWorkspaceLink,
        forbidden,
      });
    }
  }

  return { apps, failures };
}

function main() {
  const { apps, failures } = analyzeDriveAppSdkConsumerIntegration({
    workspaceRoot: DEFAULT_DRIVE_ROOT,
    driveRoot: DEFAULT_DRIVE_ROOT,
  });

  if (failures.length > 0) {
    console.error("Drive app SDK consumer integration check failed.");
    for (const failure of failures) {
      console.error(`- ${failure.app.relativeRoot} (${failure.app.key})`);
      if (failure.missingDependency) {
        console.error(`  missing dependency declaration for ${DRIVE_PACKAGE}`);
      }
      if (failure.missingWorkspaceLink) {
        console.error(
          `  pnpm-workspace.yaml does not include one of ${DRIVE_PACKAGE_WORKSPACE_SEGMENTS.join(", ")}`,
        );
      }
      for (const finding of failure.forbidden) {
        console.error(`  forbidden ${finding.name}: ${finding.relativePath}`);
      }
    }
    process.exitCode = 1;
    return;
  }

  console.log(
    `Drive app SDK consumer integration check passed for ${apps.length} frontend application roots.`,
  );
}

if (path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main();
}
