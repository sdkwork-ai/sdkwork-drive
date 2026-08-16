export interface DriveSpace {
  id: string;
  tenantId?: string;
  ownerSubjectType: string;
  ownerSubjectId: string;
  displayName: string;
  spaceType: 'personal' | 'team' | 'knowledge_base' | 'ai_generated' | 'git_repository' | 'deployment' | 'app_upload' | 'im' | 'rtc' | 'notary' | 'website';
  presentationIcon?: string;
  presentationColor?: string;
  description?: string;
  lifecycleStatus: string;
  version: string;
  createdBy?: string;
}
