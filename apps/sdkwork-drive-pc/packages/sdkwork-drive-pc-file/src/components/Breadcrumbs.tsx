import React from 'react';
import { ChevronRight, Folder, HardDrive } from 'lucide-react';
import type { DriveFile } from 'sdkwork-drive-pc-types';

interface BreadcrumbsProps {
  currentFolderId: string | null;
  allFiles: DriveFile[];
  sectionTitle: string;
  onNavigate: (folderId: string | null) => void;
  variant?: 'pill' | 'inline';
}

export function Breadcrumbs({ 
  currentFolderId, 
  allFiles, 
  sectionTitle, 
  onNavigate,
  variant = 'pill',
}: BreadcrumbsProps) {
  
  const getPathTrail = () => {
    if (!currentFolderId) {
      return [];
    }

    const folderNodes = allFiles.filter((file) => file.type === 'folder');
    const lastFolder = folderNodes.at(-1);
    if (lastFolder?.id === currentFolderId) {
      return folderNodes.map((folder) => ({ id: folder.id, name: folder.name }));
    }

    const trail: { id: string; name: string }[] = [];
    let searchId: string | undefined = currentFolderId;
    let safeguard = 0;

    while (searchId && safeguard < 20) {
      const folder = allFiles.find((file) => file.id === searchId && file.type === 'folder');
      if (folder) {
        trail.unshift({ id: folder.id, name: folder.name });
        searchId = folder.parentId;
      } else {
        break;
      }
      safeguard += 1;
    }

    return trail;
  };

  const trail = getPathTrail();
  const isInline = variant === 'inline';

  return (
    <div
      className={
        isInline
          ? 'flex min-w-0 max-w-full items-center gap-1 overflow-x-auto whitespace-nowrap text-[13px] font-medium text-gray-500 scrollbar-none dark:text-gray-400 select-none'
          : 'flex min-w-0 max-w-full items-center gap-1.5 overflow-x-auto whitespace-nowrap rounded-xl border border-gray-100 bg-gray-50/50 px-3 py-1 text-xs font-medium text-gray-500 scrollbar-none transition-all dark:border-neutral-800/40 dark:bg-[#1a1a1a]/30 dark:text-gray-400 select-none'
      }
    >
      
      {/* Root Section node link */}
      <button
        onClick={() => onNavigate(null)}
        className={`flex shrink-0 items-center gap-1.5 cursor-pointer transition-colors hover:text-blue-600 dark:hover:text-blue-400 ${
          isInline
            ? 'sdkwork-drive-breadcrumb-link font-medium text-gray-700 dark:text-gray-200'
            : 'uppercase tracking-wider font-bold text-[10.5px]'
        }`}
      >
        <HardDrive size={isInline ? 14 : 13} className="shrink-0 text-gray-400" />
        <span>{sectionTitle}</span>
      </button>

      {/* Trailing folder node links */}
      {trail.map((folder, index) => {
        const isLastItem = index === trail.length - 1;
        
        return (
          <React.Fragment key={folder.id}>
            <ChevronRight size={isInline ? 14 : 12} className="shrink-0 text-gray-300 dark:text-neutral-600" />
            <button
              onClick={() => onNavigate(folder.id)}
              disabled={isLastItem}
      className={`flex shrink-0 items-center gap-1 transition-colors ${
                isLastItem
                  ? isInline
                    ? 'sdkwork-drive-breadcrumb-current cursor-default font-semibold text-gray-900 dark:text-white'
                    : 'cursor-default font-semibold text-gray-900 dark:text-white'
                  : isInline
                    ? 'sdkwork-drive-breadcrumb-link cursor-pointer font-medium text-gray-600 hover:text-blue-600 dark:text-gray-400 dark:hover:text-blue-400'
                    : 'cursor-pointer font-medium hover:text-blue-600 dark:hover:text-blue-400'
              }`}
            >
              {!isLastItem && (
                <Folder size={isInline ? 13 : 12} className="shrink-0 fill-amber-500 text-amber-500" />
              )}
              <span className={isLastItem && trail.length > 0 ? 'max-w-[12rem] truncate sm:max-w-xs' : ''}>
                {folder.name}
              </span>
            </button>
          </React.Fragment>
        );
      })}
    </div>
  );
}
