import { describe, expect, it } from 'vitest';
import { encodeLocalFilesystemId } from '../types';
import { buildLocalFilesystemFolderPath } from './localFilesystemService';

describe('buildLocalFilesystemFolderPath', () => {
  it('builds ordered Windows breadcrumbs with native drive paths', () => {
    const folderPath = 'C:\\Users\\Ada\\Documents';
    const path = buildLocalFilesystemFolderPath(folderPath, 'user-001');

    expect(path).toEqual([
      {
        id: encodeLocalFilesystemId('C:\\'),
        name: 'C:',
        type: 'folder',
        updatedAt: expect.any(String),
        ownerId: 'user-001',
      },
      {
        id: encodeLocalFilesystemId('C:\\Users'),
        name: 'Users',
        type: 'folder',
        updatedAt: expect.any(String),
        ownerId: 'user-001',
        parentId: encodeLocalFilesystemId('C:\\'),
      },
      {
        id: encodeLocalFilesystemId('C:\\Users\\Ada'),
        name: 'Ada',
        type: 'folder',
        updatedAt: expect.any(String),
        ownerId: 'user-001',
        parentId: encodeLocalFilesystemId('C:\\Users'),
      },
      {
        id: encodeLocalFilesystemId('C:\\Users\\Ada\\Documents'),
        name: 'Documents',
        type: 'folder',
        updatedAt: expect.any(String),
        ownerId: 'user-001',
        parentId: encodeLocalFilesystemId('C:\\Users\\Ada'),
      },
    ]);
  });

  it('builds a single Windows drive root breadcrumb', () => {
    const path = buildLocalFilesystemFolderPath('D:\\', 'user-001');

    expect(path).toEqual([
      {
        id: encodeLocalFilesystemId('D:\\'),
        name: 'D:',
        type: 'folder',
        updatedAt: expect.any(String),
        ownerId: 'user-001',
      },
    ]);
  });

  it('builds ordered Unix breadcrumbs', () => {
    const path = buildLocalFilesystemFolderPath('/var/log/app', 'user-001');

    expect(path.map((folder) => folder.name)).toEqual(['var', 'log', 'app']);
    expect(path.at(-1)?.id).toBe(encodeLocalFilesystemId('/var/log/app'));
  });
});
