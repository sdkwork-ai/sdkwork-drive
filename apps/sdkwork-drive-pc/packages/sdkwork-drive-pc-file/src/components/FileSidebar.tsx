import React, { useState } from 'react';
import { Clock, Star, Share2, Trash2, Cloud, HardDrive, ChevronRight, ChevronDown, Book, Plus, Folder, Activity } from 'lucide-react';
import { formatDriveBytes, useTranslation } from 'sdkwork-drive-pc-commons';
import type { DriveSection } from '../pages/DrivePage';
import type { DriveStorageSummary, KnowledgeBaseSpace, SharedSpace } from 'sdkwork-drive-pc-core';
import { useDriveRuntime } from 'sdkwork-drive-pc-core';
import { SPACE_ICONS } from './CreateSharedSpaceModal';

interface FileSidebarProps {
  activeSection: DriveSection;
  onSectionChange: (section: DriveSection) => void;
  sharedSpaces?: SharedSpace[];
  knowledgeBaseSpaces?: KnowledgeBaseSpace[];
  storageSummary?: DriveStorageSummary;
  storageSummaryUnavailable?: boolean;
  onAddSpaceClick?: () => void;
  onDeleteSpace?: (id: string) => void;
  onOpenStorageSettings?: () => void;
}

export function FileSidebar({ 
  activeSection, 
  onSectionChange,
  sharedSpaces = [],
  knowledgeBaseSpaces = [],
  storageSummary,
  storageSummaryUnavailable = false,
  onAddSpaceClick,
  onDeleteSpace,
  onOpenStorageSettings
}: FileSidebarProps) {
  const { t } = useTranslation();
  const runtime = useDriveRuntime();
  const showComputersSection =
    runtime.config.runtimeTarget === 'desktop' || runtime.host.isNativeHost;
  const [isKbExpanded, setIsKbExpanded] = useState(false);
  
  const storageUsedLabel = storageSummary ? formatDriveBytes(storageSummary.usedBytes) : '--';
  const storageTotalLabel = storageSummary?.totalBytes
    ? formatDriveBytes(storageSummary.totalBytes)
    : '--';
  const storageUsagePercent = Math.min(100, Math.max(0, storageSummary?.usagePercent ?? 0));

  return (
    <div className="w-[220px] xl:w-[260px] h-full min-h-0 bg-[#f2f2f2] dark:bg-[#1b1b1b] border-r border-[#d6d6d6] dark:border-[#2a2a2a] flex flex-col shrink-0 transition-colors select-none">
      <div className="p-6 h-20 flex items-center">
        <span className="text-lg font-semibold tracking-tight text-[#1a1a1a] dark:text-[#eee]">Sdkwork Drive</span>
      </div>
      
      <div className="flex-1 overflow-y-auto px-3 space-y-1 flex flex-col">
        <SidebarItem icon={<Cloud size={18} />} label={t('sidebar.myStorage')} active={activeSection === 'my-storage'} onClick={() => onSectionChange('my-storage')} />
        
        <div className="mt-1 space-y-0.5">
          <button 
            onClick={() => setIsKbExpanded(!isKbExpanded)}
            className="w-full px-3 py-2 rounded-md flex items-center justify-between text-sm cursor-pointer transition-colors text-left hover:bg-[#e8e8e8] dark:hover:bg-[#2a2a2a] text-[#555] dark:text-[#aaa]"
          >
            <div className="flex items-center gap-3">
              <span className="opacity-60 text-[#555] dark:text-[#999]"><Book size={18} /></span>
              <span>{t('sidebar.knowledgeBase')}</span>
            </div>
            <span className="opacity-50">
              {isKbExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
            </span>
          </button>
          
          {isKbExpanded && (
            <div className="pl-9 pr-2 py-1 space-y-0.5 border-l-2 border-gray-200 dark:border-neutral-800 ml-4 mb-2">
              {knowledgeBaseSpaces.length === 0 ? (
                <div className="text-[12px] px-2 py-1.5 text-gray-400 dark:text-neutral-500">
                  {t('sidebar.noKnowledgeBaseSpaces')}
                </div>
              ) : (
                knowledgeBaseSpaces.map((space) => (
                  <button
                    key={space.id}
                    onClick={() => onSectionChange(space.id)}
                    className={`block w-full text-left text-[12.5px] px-2 py-1.5 rounded transition-colors truncate ${
                      activeSection === space.id
                        ? 'bg-[#e2e2e2] dark:bg-[#2c3140] text-blue-600 dark:text-blue-400 font-semibold'
                        : 'text-gray-600 dark:text-neutral-400 hover:bg-gray-200/50 dark:hover:bg-neutral-800'
                    }`}
                    title={space.description || space.name}
                  >
                    {space.name}
                  </button>
                ))
              )}
            </div>
          )}
        </div>

        <SidebarItem icon={<Clock size={18} />} label={t('sidebar.recent')} active={activeSection === 'recent'} onClick={() => onSectionChange('recent')} />
        <SidebarItem icon={<Star size={18} />} label={t('sidebar.starred')} active={activeSection === 'starred'} onClick={() => onSectionChange('starred')} />
        
        <div className="border-t border-[#dbdbdb] dark:border-[#333] my-4 mx-2" />
        
        <div 
          onClick={onAddSpaceClick}
          className="flex items-center justify-between px-3 pb-2 text-[11px] font-semibold text-[#888] dark:text-[#666] uppercase tracking-wider group cursor-pointer hover:text-blue-600 dark:hover:text-blue-400 transition-colors" 
          title="Create new Shared Space"
        >
          <span>{t('sidebar.sharedSpaces')}</span>
          <Plus size={13} className="opacity-100 sm:opacity-0 group-hover:opacity-100 text-gray-500 hover:text-blue-600 dark:hover:text-blue-400 transition-opacity" />
        </div>

        {sharedSpaces.map(space => {
          const IconComponent = SPACE_ICONS[space.icon] || Folder;
          return (
            <SidebarItem 
              key={space.id} 
              icon={<IconComponent size={18} />} 
              label={space.name} 
              active={activeSection === space.id} 
              onClick={() => onSectionChange(space.id)}
              onDelete={space.isCustom ? () => onDeleteSpace?.(space.id) : undefined}
            />
          );
        })}
        
        <SidebarItem icon={<Share2 size={18} />} label={t('sidebar.sharedWithMe')} active={activeSection === 'shared'} onClick={() => onSectionChange('shared')} />
        
        <div className="border-t border-[#dbdbdb] dark:border-[#333] my-4 mx-2" />
        
        {showComputersSection && (
          <SidebarItem icon={<HardDrive size={18} />} label={t('sidebar.computers')} active={activeSection === 'computers'} onClick={() => onSectionChange('computers')} />
        )}
        <SidebarItem icon={<Activity size={18} />} label={t('sidebar.transferCenter')} active={activeSection === 'transfer'} onClick={() => onSectionChange('transfer')} />
        <SidebarItem icon={<Trash2 size={18} />} label={t('sidebar.trash')} active={activeSection === 'trash'} onClick={() => onSectionChange('trash')} />
      </div>

      <div className="p-6 bg-[#ebebeb] dark:bg-[#1a1a1a]">
        <div className="text-[11px] uppercase tracking-wider text-[#888] dark:text-[#666] font-bold mb-2">
          {t('sidebar.storageTitle')}
        </div>
        <div className="w-full bg-[#d6d6d6] dark:bg-[#333] h-1.5 rounded-full overflow-hidden mb-2 relative">
          <div
            className="bg-blue-600 dark:bg-blue-500 h-full absolute left-0 top-0 transition-all duration-500 ease-out"
            style={{ width: `${storageUsagePercent}%` }}
          />
        </div>
        <div className="text-xs text-[#777] dark:text-[#999]">
          {storageSummaryUnavailable
            ? t('sidebar.storageUnavailable')
            : t('sidebar.storageUsed', { used: storageUsedLabel, total: storageTotalLabel })}
        </div>
        <button
          onClick={onOpenStorageSettings}
          className="mt-4 w-full py-2 px-4 rounded border border-[#d6d6d6] dark:border-[#444] bg-white dark:bg-[#222] text-[#444] dark:text-[#ddd] text-xs font-medium hover:bg-gray-50 dark:hover:bg-[#333] transition-colors cursor-pointer text-center"
        >
          {t('sidebar.getMoreStorage')}
        </button>
      </div>
    </div>
  );
}

function UsersShare({ size }: { size: number }) {
  return (
    <svg fill="currentColor" viewBox="0 0 24 24" style={{ width: size, height: size }}><path d="M16 11c1.66 0 2.99-1.34 2.99-3S17.66 5 16 5c-1.66 0-3 1.34-3 3s1.34 3 3 3zm-8 0c1.66 0 2.99-1.34 2.99-3S9.66 5 8 5C6.34 5 5 6.34 5 8s1.34 3 3 3zm0 2c-2.33 0-7 1.17-7 3.5V19h14v-2.5c0-2.33-4.67-3.5-7-3.5zm8 0c-.29 0-.62.02-.97.05 1.16.84 1.97 1.97 1.97 3.45V19h6v-2.5c0-2.33-4.67-3.5-7-3.5z"/></svg>
  );
}

function SidebarItem({ 
  icon, 
  label, 
  active, 
  onClick,
  onDelete 
}: { 
  icon: React.ReactNode;
  label: string;
  active?: boolean;
  onClick?: () => void;
  onDelete?: () => void;
}) {
  return (
    <div className="group/item relative w-full animate-in slide-in-from-left-2 duration-200">
      <button 
        onClick={onClick}
        className={`w-full px-3 py-2 rounded-md flex items-center gap-3 text-sm cursor-pointer transition-all text-left ${
          active 
            ? 'bg-[#e2e2e2] dark:bg-[#2c3140] text-blue-600 dark:text-blue-400 font-semibold shadow-sm' 
            : 'hover:bg-[#e8e8e8] dark:hover:bg-[#2a2a2a] text-[#555] dark:text-[#aaa]'
        }`}
      >
        <span className={active ? 'opacity-100 text-blue-600 dark:text-blue-400' : 'opacity-60 text-[#555] dark:text-[#999]'}>
          {icon}
        </span>
        <span className="truncate pr-5 text-[13.5px]" title={label}>{label}</span>
      </button>
      
      {onDelete && (
        <button
          onClick={(e) => {
            e.stopPropagation();
            onDelete();
          }}
          className="absolute right-2 top-[8px] p-1 text-gray-400 hover:text-red-500 hover:bg-neutral-200 dark:hover:bg-neutral-800 rounded opacity-0 group-hover/item:opacity-100 transition-opacity cursor-pointer flex items-center justify-center"
          title="Delete shared space"
        >
          <Trash2 size={12} className="stroke-[2.5]" />
        </button>
      )}
    </div>
  );
}
