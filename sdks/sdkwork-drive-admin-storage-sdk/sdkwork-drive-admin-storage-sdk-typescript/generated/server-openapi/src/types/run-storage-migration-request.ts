export interface RunStorageMigrationRequest {
  /** Objects to copy in this call. Clamped into the supported range rather than rejected, because it is a throughput knob and not a contract. */
  batchSize?: string | null;
}
