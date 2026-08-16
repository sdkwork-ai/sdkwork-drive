export interface EmptyTrashResponse {
  deletedCount: string;
  skippedCount: string;
  /** True when more trashed items remain after this batch (500-item cap per request). */
  hasMore: boolean;
}
