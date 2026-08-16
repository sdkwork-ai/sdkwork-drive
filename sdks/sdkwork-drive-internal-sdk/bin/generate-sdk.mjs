#!/usr/bin/env node
import {
  resolveFamilySdkRoot,
  runDriveSdkGenerator,
} from "../../../tools/drive_sdk_generator_runner.mjs";

runDriveSdkGenerator(
  {
    sdkName: "sdkwork-drive-internal-sdk",
    sdkOwner: "sdkwork-drive",
    apiAuthority: "sdkwork-drive-internal-api",
    sdkSurface: "internal",
    sdkDependencies: [],
    sdkRoot: resolveFamilySdkRoot(import.meta.url),
    sdkType: "custom",
    apiPrefix: "/internal/v3/api",
    defaultBaseUrl: "http://127.0.0.1:18080",
    defaultOpenapiPath: "apis/internal-api/drive/sdkwork-drive-internal-api.openapi.yaml",
    authorityMirrorFile: "sdkwork-drive-internal-api.openapi.yaml",
    generationInputFile: "sdkwork-drive-internal-api.sdkgen.yaml",
    standardProfileArgs: ["--standard-profile", "sdkwork-v3"],
    manifestStandardProfile: "sdkwork-v3",
  },
  process.argv.slice(2),
);
