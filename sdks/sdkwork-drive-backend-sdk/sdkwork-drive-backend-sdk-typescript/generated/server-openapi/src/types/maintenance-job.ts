export interface MaintenanceJob {
  id: string;
  jobType: 'object_sweep' | 'upload_session_sweep' | 'expired_upload_content_sweep' | 'abandoned_upload_task_sweep';
  status: 'completed' | 'failed';
  dryRun: boolean;
  scannedCount: string;
  affectedCount: string;
  operatorId: string;
  correlationId?: string;
  traceId?: string;
  errorMessage?: string;
  startedAt: string;
  finishedAt: string;
  createdAt: string;
}
