export interface DriveLabel {
  id: string;
  tenantId: string;
  labelKey: string;
  displayName: string;
  color?: string | null;
  description?: string | null;
  lifecycleStatus: 'active' | 'deleted';
  version: number;
}
