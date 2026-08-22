import type { EffectivePermission } from './effective-permission';
import type { PageInfo } from './page-info';

export interface EffectivePermissionListHttpResponse {
  code: 0;
  data: unknown & { items: EffectivePermission[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
