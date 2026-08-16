export interface DriveLabelSummary {
  id: string;
  tenantId?: string;
  labelKey: string;
  displayName: string;
  color?: string | null;
  description?: string | null;
  lifecycleStatus: 'active';
  version: number;
}
