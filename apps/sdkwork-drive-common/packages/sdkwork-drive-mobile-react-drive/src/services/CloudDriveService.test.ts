import assert from "node:assert/strict";
import test from "node:test";

import type { SdkworkDriveAppClient } from "@sdkwork/drive-app-sdk";

import {
  CloudDriveCapabilityUnavailableError,
  CloudDriveService,
  configureCloudDriveRuntime,
  resetCloudDriveRuntime,
} from "./CloudDriveService";

const node = {
  id: "node-1",
  spaceId: "space-1",
  nodeType: "file" as const,
  nodeName: "report.pdf",
  lifecycleStatus: "active" as const,
  version: "1",
  spaceType: "personal" as const,
  contentState: "ready" as const,
  fileExtension: "pdf",
  contentType: "application/pdf",
  contentTypeGroup: "document" as const,
  contentLength: "2097152",
  createdAt: "2026-07-31T00:00:00.000Z",
  updatedAt: "2026-07-31T01:00:00.000Z",
};

test("cloud drive operations fail closed until the Drive owner SDK is composed", async () => {
  resetCloudDriveRuntime();
  await assert.rejects(CloudDriveService.getFiles(), CloudDriveCapabilityUnavailableError);
});

test("cloud drive delegates file operations to the injected Drive app SDK", async () => {
  const calls: string[] = [];
  const client = {
    drive: {
      spaces: {
        list: async () => {
          calls.push("spaces.list");
          return { items: [{ id: "space-1" }], pageInfo: {} };
        },
      },
      nodes: {
        list: async () => {
          calls.push("nodes.list");
          return { items: [node], pageInfo: {} };
        },
        folders: {
          create: async () => {
            calls.push("nodes.folders.create");
            return { ...node, id: "folder-1", nodeName: "Folder", nodeType: "folder" as const };
          },
        },
        update: async () => {
          calls.push("nodes.update");
          return node;
        },
        delete: async () => {
          calls.push("nodes.delete");
        },
      },
    },
  } as unknown as SdkworkDriveAppClient;
  configureCloudDriveRuntime({ client });

  assert.deepEqual(await CloudDriveService.getFiles(), [
    {
      id: "node-1",
      name: "report.pdf",
      type: "pdf",
      size: "2.0 MB",
      date: new Date(node.updatedAt).toLocaleString(),
      owner: "",
    },
  ]);
  assert.equal((await CloudDriveService.createFolder("Folder")).id, "folder-1");
  await CloudDriveService.renameFile("node-1", "renamed.pdf");
  await CloudDriveService.deleteFile("node-1");

  assert.deepEqual(calls, [
    "spaces.list",
    "nodes.list",
    "spaces.list",
    "nodes.folders.create",
    "nodes.update",
    "nodes.delete",
  ]);
  resetCloudDriveRuntime();
});
