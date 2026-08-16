import { describe, expect, it } from "vitest";
import type { DriveFile } from "sdkwork-drive-pc-types";
import { sortDriveFiles } from "./fileBrowserSort";

function file(overrides: Partial<DriveFile> & Pick<DriveFile, "id" | "name">): DriveFile {
  return {
    type: "file",
    ownerId: "owner",
    updatedAt: "2026-01-01T00:00:00.000Z",
    size: 1,
    ...overrides,
  } as DriveFile;
}

describe("sortDriveFiles", () => {
  it("keeps folders before files", () => {
    const sorted = sortDriveFiles(
      [
        file({ id: "file-1", name: "alpha.txt" }),
        file({ id: "folder-1", name: "beta", type: "folder" }),
      ],
      "name",
      "asc",
    );

    expect(sorted.map((item) => item.id)).toEqual(["folder-1", "file-1"]);
  });

  it("sorts by name using the requested order", () => {
    const sorted = sortDriveFiles(
      [
        file({ id: "b", name: "Bravo" }),
        file({ id: "a", name: "alpha" }),
      ],
      "name",
      "asc",
    );

    expect(sorted.map((item) => item.id)).toEqual(["a", "b"]);
  });
});
