export interface ProviderObjectMutation {
  providerId: string;
  bucket: string;
  /** Drive object key. UTF-8 1-1024 bytes, trimmed relative key; no leading/trailing slash, double slash, NUL, or period-only path segments. */
  objectKey: string;
  changed: boolean;
}
