export interface AuditEventView {
  id: number;
  tenantId: string;
  action: string;
  resourceType: string;
  resourceId: string;
  operatorId: string;
  correlationId?: string;
  traceId?: string;
  createdAt: string;
}

export interface AuditEventPageView {
  items: AuditEventView[];
  page?: number;
  pageSize?: number;
  total?: number;
  pageInfo?: {
    mode?: 'offset' | 'cursor';
    page?: number;
    pageSize?: number;
    totalItems?: string;
    totalPages?: number;
    hasMore?: boolean;
  };
}

export type MaintenanceJobType =
  | 'object_sweep'
  | 'upload_session_sweep'
  | 'expired_upload_content_sweep'
  | 'abandoned_upload_task_sweep';

export type MaintenanceJobStatus = 'completed' | 'failed';

export interface MaintenanceJobView {
  id: number;
  jobType: MaintenanceJobType;
  status: MaintenanceJobStatus;
  dryRun: boolean;
  scannedCount: number;
  affectedCount: number;
  operatorId: string;
  startedAt: string;
  finishedAt: string;
  createdAt: string;
}

export interface MaintenanceJobPageView {
  items: MaintenanceJobView[];
  page?: number;
  pageSize?: number;
  total?: number;
  pageInfo?: {
    mode?: 'offset' | 'cursor';
    page?: number;
    pageSize?: number;
    totalItems?: string;
    totalPages?: number;
    hasMore?: boolean;
  };
}

export interface MaintenanceSweepResultView {
  scannedCount: number;
  affectedCount: number;
  dryRun: boolean;
}

export interface QuotaSummaryView {
  tenantId: string;
  totalBytes: number;
  objectCount: number;
  quotaBytes?: number | null;
}

export interface LabelView {
  id: string;
  tenantId: string;
  labelKey: string;
  displayName: string;
  color?: string | null;
  description?: string | null;
  lifecycleStatus: string;
  version: number;
}

export interface LabelListView {
  items: LabelView[];
  nextPageToken?: string | null;
  pageInfo?: {
    mode?: 'offset' | 'cursor';
    pageSize?: number;
    hasMore?: boolean;
    nextCursor?: string | null;
  };
}

export interface DriveSpaceAdminView {
  id: string;
  tenantId: string;
  ownerSubjectType: string;
  ownerSubjectId: string;
  displayName: string;
  spaceType: string;
  lifecycleStatus: string;
  version: number;
}

export interface DriveSpaceListView {
  items: DriveSpaceAdminView[];
}

export interface DownloadPackageView {
  id: string;
  tenantId: string;
  packageName: string;
  state: 'creating' | 'ready' | 'failed' | 'expired';
  storageProviderId: string;
  bucket: string;
  archiveObjectKey: string;
  contentType: string;
  fileCount: number;
  totalBytes: number;
  archiveSizeBytes: number;
  expiresAtEpochMs: number;
  errorMessage?: string | null;
  createdBy: string;
  updatedBy: string;
  createdAt: string;
  updatedAt: string;
}

export interface DownloadPackagePageView {
  items: DownloadPackageView[];
  page?: number;
  pageSize?: number;
  total?: number;
  pageInfo?: {
    mode?: 'offset' | 'cursor';
    page?: number;
    pageSize?: number;
    totalItems?: string;
    totalPages?: number;
    hasMore?: boolean;
  };
}

export interface ListAuditEventsQuery {
  action?: string;
  resourceType?: string;
  resourceId?: string;
  correlationId?: string;
  traceId?: string;
  page?: number;
  pageSize?: number;
  signal?: AbortSignal;
}

export interface ListMaintenanceJobsQuery {
  jobType?: MaintenanceJobType;
  status?: MaintenanceJobStatus;
  operatorId?: string;
  page?: number;
  pageSize?: number;
  signal?: AbortSignal;
}

export interface ListLabelsQuery {
  lifecycleStatus?: string;
  pageSize?: number;
  pageToken?: string;
  signal?: AbortSignal;
}

export interface ListSpacesAdminQuery {
  ownerSubjectType?: string;
  ownerSubjectId?: string;
  pageSize?: number;
  pageToken?: string;
  signal?: AbortSignal;
}

export interface ListDownloadPackagesQuery {
  state?: DownloadPackageView['state'];
  page?: number;
  pageSize?: number;
  signal?: AbortSignal;
}

export interface StartMaintenanceSweepInput {
  jobType: MaintenanceJobType;
  dryRun: boolean;
  limit?: number;
}

export interface UpdateQuotaPolicyInput {
  quotaBytes?: number;
  clearTenantPolicy?: boolean;
}

export interface CreateLabelInput {
  id: string;
  labelKey: string;
  displayName: string;
  color?: string;
  description?: string;
}

export interface UpdateLabelInput {
  displayName?: string;
  color?: string;
  description?: string;
}
