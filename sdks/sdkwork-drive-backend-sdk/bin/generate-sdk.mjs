#!/usr/bin/env node
import {
  resolveFamilySdkRoot,
  runDriveSdkGenerator,
} from "../../../tools/drive_sdk_generator_runner.mjs";

runDriveSdkGenerator(
  {
    sdkName: "sdkwork-drive-backend-sdk",
    sdkOwner: "sdkwork-drive",
    apiAuthority: "sdkwork-drive.backend",
    sdkDependencies: [
      {
        workspace: "sdkwork-iam-backend-sdk",
        role: "appbase-backend-management-capability",
        required: true,
        dependencyMode: "consumer-sdk",
        apiPrefix: "/backend/v3/api",
        apiAuthority: "sdkwork-iam-backend-api",
        generatedTransportImportPolicy: "forbidden",
        packageByLanguage: {
          typescript: "@sdkwork/iam-backend-sdk",
          rust: "sdkwork-iam-backend-sdk",
          java: "com.sdkwork:sdkwork-iam-backend-sdk",
          python: "sdkwork-iam-backend-sdk",
          go: "github.com/sdkwork/sdkwork-iam-backend-sdk",
        },
      },
    ],
    sdkRoot: resolveFamilySdkRoot(import.meta.url),
    sdkType: "backend",
    apiPrefix: "/backend/v3/api",
    defaultBaseUrl: "http://127.0.0.1:18080",
    defaultOpenapiPath: "apis/backend-api/drive/drive-backend-api.openapi.json",
    standardProfileArgs: ["--standard-profile", "sdkwork-v3"],
    manifestStandardProfile: "sdkwork-v3",
  },
  process.argv.slice(2),
);
