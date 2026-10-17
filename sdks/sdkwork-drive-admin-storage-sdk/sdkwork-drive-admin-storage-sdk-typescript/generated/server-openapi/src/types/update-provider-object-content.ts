export interface UpdateProviderObjectContent {
  /** Object content. Empty content with a trailing-slash object key creates a directory placeholder. */
  content: string;
  encoding?: 'utf8' | 'base64';
  /** Optional object content type. */
  contentType?: string;
}
