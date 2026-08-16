import type { DriveLabelSummary } from './drive-label-summary';

export interface NodeLabel {
  id: string;
  tenantId?: string;
  nodeId: string;
  labelId: string;
  lifecycleStatus: 'active';
  version: number;
  label: DriveLabelSummary;
}
