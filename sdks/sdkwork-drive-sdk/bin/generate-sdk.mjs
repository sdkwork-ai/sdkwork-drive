#!/usr/bin/env node
import {
  resolveFamilySdkRoot,
  runDriveSdkGenerator,
} from "../../../tools/drive_sdk_generator_runner.mjs";

runDriveSdkGenerator(
  {
    sdkName: "sdkwork-drive-sdk",
    sdkOwner: "sdkwork-drive",
    apiAuthority: "sdkwork-drive.open",
    sdkRoot: resolveFamilySdkRoot(import.meta.url),
    sdkType: "custom",
    apiPrefix: "/open/v3/api",
    defaultBaseUrl: "http://127.0.0.1:18082",
    defaultOpenapiPath: "apis/open-api/drive/drive-open-api.openapi.json",
    standardProfileArgs: [],
    manifestStandardProfile: "sdkwork-drive-open-v3",
  },
  process.argv.slice(2),
);
