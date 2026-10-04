/* @vitest-environment jsdom */

import { act } from "react";
import { cleanup, fireEvent, render, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type {
  DriveUploadImageService,
  DriveUploadImageValue,
} from "@sdkwork/drive-upload-image-core";
import { DriveUploadImage, type DriveUploadImageHandle } from "../src";

/**
 * Persist-first regression for the PC shell (`DRIVE_SPEC.md` §18.3).
 *
 * Create dialogs pick a file while the entity id does not exist yet: the
 * field must park the pick in its controller, report the parked state through
 * `onPendingChange`, and flush it through the ref handle once the host has
 * the id — the same first-class flow the mini-program shell exposes as
 * `chooseAndUpload`/`uploadPending`. Edit dialogs with a static anchor keep
 * uploading immediately.
 */

afterEach(() => cleanup());

// The persist-first flush is driven through manual `act` batches; React only
// allows them when the test environment declares itself as one.
(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

const uploadedValue: DriveUploadImageValue = {
  uri: "drive://spaces/space-1/nodes/node-9",
  source: "drive",
  metadata: { drive: { nodeId: "node-9", spaceId: "space-1" } },
};

function createDeferredRecordingService() {
  const anchors: string[] = [];
  let resolveUpload: ((value: DriveUploadImageValue) => void) | undefined;
  const service: DriveUploadImageService = {
    upload: (input) => {
      anchors.push(input.appResourceId);
      return new Promise<DriveUploadImageValue>((resolve) => {
        resolveUpload = resolve;
      });
    },
    resolvePreview: () => Promise.resolve(null),
  };
  return {
    anchors,
    service,
    settle: () => resolveUpload?.(uploadedValue),
  };
}

async function pickAvatarFile(): Promise<void> {
  const picker = document.querySelector('input[type="file"]') as HTMLInputElement;
  expect(picker).not.toBeNull();
  const file = new File(["avatar-bytes"], "avatar.png", { type: "image/png" });
  Object.defineProperty(picker, "files", { value: [file] });
  await act(async () => {
    fireEvent.change(picker);
    await Promise.resolve();
    await Promise.resolve();
  });
}

describe("DriveUploadImage persist-first handle (PC shell)", () => {
  it("parks a create-mode pick without an anchor and flushes it through the handle", async () => {
    const recording = createDeferredRecordingService();
    const onPendingChange = vi.fn();
    const handle: { current: DriveUploadImageHandle | null } = { current: null };

    render(
      <DriveUploadImage
        appResourceId={() => null}
        onPendingChange={onPendingChange}
        ref={handle}
        service={recording.service}
      />,
    );

    await pickAvatarFile();

    expect(recording.anchors).toEqual([]);
    expect(handle.current?.hasPending()).toBe(true);
    expect(onPendingChange).toHaveBeenLastCalledWith(true);

    let values: readonly DriveUploadImageValue[] = [];
    await act(async () => {
      const pending = handle.current?.uploadPending({ appResourceId: "user-9" });
      recording.settle();
      values = (await pending) ?? [];
    });

    expect(recording.anchors).toEqual(["user-9"]);
    expect(values.map((value) => value.uri)).toEqual([uploadedValue.uri]);
    expect(handle.current?.hasPending()).toBe(false);
    expect(onPendingChange).toHaveBeenLastCalledWith(false);
  });

  it("uploads immediately when the entity anchor already exists", async () => {
    const recording = createDeferredRecordingService();
    const onChange = vi.fn();

    render(
      <DriveUploadImage
        appResourceId="user-1"
        onChange={onChange}
        service={recording.service}
      />,
    );

    await pickAvatarFile();

    await waitFor(() => expect(recording.anchors).toEqual(["user-1"]));
    await act(async () => {
      recording.settle();
    });
    await waitFor(() =>
      expect(onChange).toHaveBeenCalledWith(expect.objectContaining({ uri: uploadedValue.uri })),
    );
  });
});
