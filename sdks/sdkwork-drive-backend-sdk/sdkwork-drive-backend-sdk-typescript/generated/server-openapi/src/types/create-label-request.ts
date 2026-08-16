export interface CreateLabelRequest {
  id: string;
  labelKey: string;
  displayName: string;
  color?: string;
  description?: string;
}
