import { useEffect, useRef, useState, useSyncExternalStore } from "react";
import {
  DriveUploadImageController,
  type DriveUploadImageControllerOptions,
  type DriveUploadImageValue,
} from "@sdkwork/drive-upload-image-core";

/**
 * Binds the shared upload state machine to React. The controller is created
 * once per mount; mutable option callbacks always delegate to the latest
 * render's props so callers can pass inline closures safely. An
 * `externalController` (tests, shared state) is adopted as-is and its
 * lifecycle stays with the caller.
 */
export function useDriveUploadImageController(
  options: DriveUploadImageControllerOptions,
  externalController?: DriveUploadImageController,
): DriveUploadImageController {
  const latest = useRef(options);
  latest.current = options;
  // Deferred destruction. React StrictMode mounts, destroys, and remounts a
  // component while keeping its state, so destroying synchronously in the
  // effect cleanup would leave the remounted tree holding a dead controller
  // ("This controller has been destroyed"). A zero-delay timer is cancelled by
  // the remount's effect when it runs first; a real unmount lets it fire.
  const destroyTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const [controller] = useState(() => {
    if (externalController !== undefined) {
      return externalController;
    }
    return new DriveUploadImageController({
      ...options,
      resolveAppResourceId: () => latest.current.resolveAppResourceId?.(),
      onUploaded: (values) => latest.current.onUploaded?.(values),
      onRejected: (rejection) => latest.current.onRejected?.(rejection),
      onFailed: (failure) => latest.current.onFailed?.(failure),
    });
  });

  useEffect(() => {
    if (externalController !== undefined) {
      return undefined;
    }
    if (destroyTimer.current !== null) {
      clearTimeout(destroyTimer.current);
      destroyTimer.current = null;
    }
    return () => {
      destroyTimer.current = setTimeout(() => {
        destroyTimer.current = null;
        controller.destroy();
      }, 0);
    };
  }, [controller, externalController]);

  return controller;
}

/** Subscribes a React tree to the controller snapshot. */
export function useDriveUploadImageSnapshot(
  controller: DriveUploadImageController,
): ReturnType<DriveUploadImageController["getSnapshot"]> {
  return useSyncExternalStore(
    controller.subscribe,
    controller.getSnapshot,
    controller.getSnapshot,
  );
}

/**
 * Keeps the controller's uploaded set in sync with a controlled `value`
 * prop. Undefined leaves the controller uncontrolled. Reconciliation is
 * idempotent: the controller ignores identical uri sequences.
 */
export function useDriveUploadImageControlledValue(
  controller: DriveUploadImageController,
  value: readonly DriveUploadImageValue[] | null | undefined,
): void {
  useEffect(() => {
    if (value !== undefined) {
      controller.setValues(value);
    }
  }, [controller, value]);
}
