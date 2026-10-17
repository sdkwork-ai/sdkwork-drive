/** One monthly ingestion bucket. */
export interface StorageOverviewTrendPoint {
  /** Closed month bucket label in YYYY-MM. */
  periodLabel: string;
  /** Objects created in that month. */
  objectCount: string;
  /** Sum of content_length created in that month. */
  bytes: string;
}
