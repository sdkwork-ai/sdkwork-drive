export interface CreatePermissionRequest {
  id: string;
  subjectType: 'user' | 'group' | 'domain' | 'app';
  subjectId: string;
  role: 'reader' | 'commenter' | 'writer' | 'owner';
}
