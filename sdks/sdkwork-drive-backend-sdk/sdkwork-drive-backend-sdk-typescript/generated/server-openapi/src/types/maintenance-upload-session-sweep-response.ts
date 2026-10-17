export interface MaintenanceUploadSessionSweepResponse {
  code: 0;
  /** Operation-specific payload typed per response schema. */
  data: unknown;
  /** Server-owned request correlation id. */
  traceId: string;
  scannedCount: string;
  affectedCount: string;
  dryRun: boolean;
}
