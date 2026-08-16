#!/usr/bin/env node
import {
  resolveFamilySdkRoot,
  runDriveSdkGenerator,
} from "../../../tools/drive_sdk_generator_runner.mjs";

runDriveSdkGenerator(
  {
    sdkName: "sdkwork-drive-app-sdk",
    sdkOwner: "sdkwork-drive",
    apiAuthority: "sdkwork-drive-app-api",
    sdkDependencies: [
      {
        workspace: "sdkwork-iam-app-sdk",
        role: "appbase-app-capability",
        required: true,
        dependencyMode: "consumer-sdk",
        apiPrefix: "/app/v3/api",
        apiAuthority: "sdkwork-iam-app-api",
        generatedTransportImportPolicy: "forbidden",
        packageByLanguage: {
          typescript: "@sdkwork/iam-app-sdk",
          rust: "sdkwork-iam-app-sdk",
          java: "com.sdkwork:sdkwork-iam-app-sdk",
          python: "sdkwork-iam-app-sdk",
          go: "github.com/sdkwork/sdkwork-iam-app-sdk",
        },
      },
    ],
    sdkRoot: resolveFamilySdkRoot(import.meta.url),
    sdkType: "app",
    apiPrefix: "/app/v3/api",
    defaultBaseUrl: "http://127.0.0.1:18080",
    defaultOpenapiPath: "apis/app-api/drive/drive-app-api.openapi.json",
    standardProfileArgs: ["--standard-profile", "sdkwork-v3"],
    manifestStandardProfile: "sdkwork-v3",
  },
  process.argv.slice(2),
);
