import type { DriveUploadImageDeclaration } from "./types";
import { DriveUploadImageError } from "./types";

/**
 * Validates the host application's upload intent constant against
 * `DRIVE_SPEC.md` §18 field rules. Unknown fields are rejected so a typo
 * cannot silently drop a declaration dimension, and the reserved Drive `im`
 * scene is refused. The input stays `unknown` because declaration constants
 * reach this boundary as JSON or hand-built objects; after the assert, the
 * value narrows to {@linkcode DriveUploadImageDeclaration}.
 */

const APP_RESOURCE_TYPE_PATTERN = /^[a-z][a-z0-9-]*(\.[a-z][a-z0-9-]*)+$/;
const KEKBEL_LABEL_PATTERN = /^[a-z0-9]+(-[a-z0-9]+)*$/;
const PROFILES: ReadonlySet<string> = new Set(["image", "avatar", "thumbnail"]);
const DECLARATION_FIELDS: ReadonlySet<string> = new Set([
  "appResourceType",
  "appResourceIdKind",
  "scene",
  "source",
  "uploadProfileCode",
  "retention",
  "retentionTtlSeconds",
  "purpose",
]);
const RESOURCE_ID_KINDS: ReadonlySet<string> = new Set([
  "application",
  "entity",
  "draft",
]);

export function assertDriveUploadImageDeclaration(
  declaration: unknown,
): asserts declaration is DriveUploadImageDeclaration {
  if (declaration === null || typeof declaration !== "object") {
    throw new DriveUploadImageError(
      "invalid-declaration",
      "Upload declaration must be an object.",
    );
  }
  const candidate: Record<string, unknown> = declaration as Record<string, unknown>;
  for (const key of Object.keys(candidate)) {
    if (!DECLARATION_FIELDS.has(key)) {
      throw new DriveUploadImageError(
        "invalid-declaration",
        `Unknown upload declaration field: ${key}`,
      );
    }
  }
  if (
    typeof candidate.appResourceType !== "string" ||
    !APP_RESOURCE_TYPE_PATTERN.test(candidate.appResourceType)
  ) {
    throw new DriveUploadImageError(
      "invalid-declaration",
      "appResourceType must be a lowercase dotted business type with at least two segments.",
    );
  }
  if (
    typeof candidate.appResourceIdKind !== "string" ||
    !RESOURCE_ID_KINDS.has(candidate.appResourceIdKind)
  ) {
    throw new DriveUploadImageError(
      "invalid-declaration",
      "appResourceIdKind must be one of: application, entity, draft.",
    );
  }
  if (
    typeof candidate.scene !== "string" ||
    !KEKBEL_LABEL_PATTERN.test(candidate.scene) ||
    candidate.scene === "im"
  ) {
    throw new DriveUploadImageError(
      "invalid-declaration",
      "scene must be a lowercase kebab-case workflow label and must not use the reserved Drive scene `im`.",
    );
  }
  if (
    typeof candidate.source !== "string" ||
    !KEKBEL_LABEL_PATTERN.test(candidate.source)
  ) {
    throw new DriveUploadImageError(
      "invalid-declaration",
      "source must be a stable lowercase kebab-case label, not a package name or module path.",
    );
  }
  if (
    typeof candidate.uploadProfileCode !== "string" ||
    !PROFILES.has(candidate.uploadProfileCode)
  ) {
    throw new DriveUploadImageError(
      "invalid-declaration",
      "uploadProfileCode must be one of the standard image-shaped profiles: image, avatar, thumbnail.",
    );
  }
  if (candidate.retention !== "long_term" && candidate.retention !== "temporary") {
    throw new DriveUploadImageError(
      "invalid-declaration",
      "retention must be long_term or temporary.",
    );
  }
  if (
    candidate.retention === "temporary" &&
    (typeof candidate.retentionTtlSeconds !== "number" ||
      !Number.isInteger(candidate.retentionTtlSeconds) ||
      candidate.retentionTtlSeconds <= 0)
  ) {
    throw new DriveUploadImageError(
      "invalid-declaration",
      "A temporary declaration must declare a positive integer retentionTtlSeconds.",
    );
  }
  if (typeof candidate.purpose !== "string" || candidate.purpose.trim() === "") {
    throw new DriveUploadImageError(
      "invalid-declaration",
      "purpose must be a non-empty sentence.",
    );
  }
}
