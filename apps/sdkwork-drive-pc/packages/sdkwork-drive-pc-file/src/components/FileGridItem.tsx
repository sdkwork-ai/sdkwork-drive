import React, { useEffect, useRef } from 'react';
import { 
  Star, 
  MoreHorizontal, 
  Info, 
  Download, 
  Edit2, 
  Trash, 
  Trash2, 
  RefreshCcw,
  Link2,
  FolderInput,
  Copy,
} from 'lucide-react';
import { useTranslation } from 'sdkwork-drive-pc-commons';
import type { DriveFile } from 'sdkwork-drive-pc-types';
import type { DriveSection } from '../pages/DrivePage';
import { FileIcon } from './FileIcon';

interface FileGridItemProps {
  file: DriveFile;
  activeSection: DriveSection;
  activeMenuId: string | null;
  setActiveMenuId: (id: string | null) => void;
  onToggleStar: (e: React.MouseEvent, fileId: string, fileName: string) => void;
  onDownload: (e: React.MouseEvent, file: DriveFile) => void;
  onPreview: (file: DriveFile) => void;
  onRename: (e: React.MouseEvent, file: DriveFile) => void;
  onTrashAction: (e: React.MouseEvent, file: DriveFile) => void;
  onPermanentDelete: (e: React.MouseEvent, file: DriveFile) => void;
  onShare?: (file: DriveFile) => void;
  onMove?: (file: DriveFile) => void;
  onCopy?: (file: DriveFile) => void;
  onDrillDown: (folderId: string) => void;
  formatDate: (dateString: string) => string;
  formatSize: (bytes?: number) => string;
  isInlineEditing?: boolean;
  onInlineRenameSubmit?: (newName: string) => void;
  onInlineRenameCancel?: () => void;
  isSelected?: boolean;
  onToggleSelect?: (e: React.MouseEvent, fileId: string) => void;
  hasSelection?: boolean;
  isTrashSection?: boolean;
}

export function FileGridItem({
  file,
  activeSection,
  activeMenuId,
  setActiveMenuId,
  onToggleStar,
  onDownload,
  onPreview,
  onRename,
  onTrashAction,
  onPermanentDelete,
  onShare,
  onMove,
  onCopy,
  onDrillDown,
  formatDate,
  formatSize,
  isInlineEditing = false,
  onInlineRenameSubmit,
  onInlineRenameCancel,
  isSelected = false,
  onToggleSelect,
  hasSelection = false,
  isTrashSection: isTrashSectionProp,
}: FileGridItemProps) {
  const { t } = useTranslation();
  const menuRef = useRef<HTMLDivElement>(null);

  const isItemStarred = file.isStarred;
  const isMenuOpen = activeMenuId === file.id;
  const isTrashSection = isTrashSectionProp ?? activeSection === 'trash';
  const isComputerSection = activeSection === 'computers';
  const hideCloudFileActions = isTrashSection || isComputerSection;

  // Handle outside clicks to close the dropdown for this item
  useEffect(() => {
    if (!isMenuOpen) return;

    const handleOutsideClick = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setActiveMenuId(null);
      }
    };

    // Timeout prevents immediate trigger during button click propagation
    const timer = setTimeout(() => {
      window.addEventListener('click', handleOutsideClick);
    }, 0);

    return () => {
      clearTimeout(timer);
      window.removeEventListener('click', handleOutsideClick);
    };
  }, [isMenuOpen, setActiveMenuId]);

  const handleGridDoubleClick = () => {
    if (file.type === 'folder') {
      onDrillDown(file.id);
    } else {
      onPreview(file);
    }
  };

  const handleMenuToggle = (e: React.MouseEvent) => {
    e.stopPropagation();
    setActiveMenuId(isMenuOpen ? null : file.id);
  };

  return (
    <div 
      onDoubleClick={handleGridDoubleClick}
      className={`border rounded-2xl p-4 px-4.5 flex flex-col justify-between hover:shadow-lg transition-all group relative h-[145px] select-none cursor-pointer ${
        isSelected 
          ? 'border-blue-500 dark:border-blue-400 bg-blue-500/10 dark:bg-blue-500/10 shadow-[0_0_0_1px_#3b82f6]' 
          : 'border-neutral-100 dark:border-neutral-800/80 hover:border-neutral-200 dark:hover:border-neutral-700 bg-white/70 dark:bg-[#131315]/90 hover:bg-neutral-50 dark:hover:bg-[#18181c]'
      }`}
      style={{ contentVisibility: "auto", containIntrinsicSize: "0 145px" }}
    >
      {/* Top Action Indicators */}
      <div className="flex items-start justify-between w-full h-10">
        <div className="flex items-center gap-2.5 group/iconcontainer">
          {/* Custom Checkbox */}
          <button
            onClick={(e) => {
              e.stopPropagation();
              onToggleSelect?.(e, file.id);
            }}
            className={`w-4.5 h-4.5 rounded-md border transition-all flex items-center justify-center shrink-0 cursor-pointer ${
              isSelected 
                ? 'bg-blue-600 border-blue-500 text-white opacity-100' 
                : hasSelection 
                  ? 'border-gray-200 dark:border-neutral-700 opacity-60 bg-white dark:bg-neutral-900'
                  : 'border-gray-200 dark:border-neutral-800 hover:border-gray-300 bg-white dark:bg-neutral-900 opacity-0 group-hover:opacity-100'
            }`}
          >
            {isSelected && (
              <svg className="w-2.5 h-2.5 text-white fill-none stroke-current stroke-[3.5]" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
              </svg>
            )}
          </button>

          <div className="transition-transform duration-200 group-hover/iconcontainer:scale-110">
            <FileIcon type={file.type} mimeType={file.mimeType} color={file.color} />
          </div>
        </div>
        
        <div className="flex items-center gap-1.5" ref={menuRef}>
          {/* Hover Star Action shortcut */}
          {!hideCloudFileActions && (
            <button
              onClick={(e) => onToggleStar(e, file.id, file.name)}
              className={`text-gray-300 hover:text-amber-500 dark:text-neutral-700 transition-colors cursor-pointer ${isItemStarred ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'}`}
            >
              <Star size={15} className={isItemStarred ? 'fill-current text-[#ffb020] dark:text-[#ffb020]' : ''} />
            </button>
          )}

          <button 
            onClick={handleMenuToggle}
            className={`p-1 rounded-lg transition-all cursor-pointer ${isMenuOpen ? 'opacity-100 bg-[#ebebeb] dark:bg-[#444] text-gray-800 dark:text-white' : 'text-gray-400 dark:text-neutral-500 group-hover:opacity-100 opacity-0'}`}
          >
            <MoreHorizontal size={16} />
          </button>

          {/* Dropdown Options container */}
          {isMenuOpen && (
            <div className="absolute right-4 top-11 w-48 bg-white dark:bg-[#1e1e1e] border border-gray-100 dark:border-neutral-800 rounded-xl shadow-2xl z-50 py-1.5 text-left origin-top-right select-none animate-in fade-in zoom-in-95 duration-100">
              <button 
                onClick={(e) => {
                  e.stopPropagation();
                  onPreview(file);
                  setActiveMenuId(null);
                }}
                className="w-full flex items-center gap-2.5 px-4 py-2 text-[13px] text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-neutral-800 transition-colors cursor-pointer"
              >
                 <Info size={14} className="text-gray-400 shrink-0"/> {isComputerSection && file.type === 'file' ? (t('fileBrowser.openLocalFile') || 'Open') : t('fileBrowser.propertiesAndInfo')}
              </button>
              {!hideCloudFileActions && (
                <>
                  <button
                    onClick={(e) => {
                      onDownload(e, file);
                      setActiveMenuId(null);
                    }}
                    className="w-full flex items-center gap-2.5 px-4 py-2 text-[13px] text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-neutral-800 transition-colors cursor-pointer"
                  >
                     <Download size={14} className="text-gray-400 shrink-0"/> {t('fileBrowser.download')}
                  </button>
                  <button
                    onClick={(e) => onRename(e, file)}
                    className="w-full flex items-center gap-2.5 px-4 py-2 text-[13px] text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-neutral-800 transition-colors cursor-pointer"
                  >
                     <Edit2 size={14} className="text-gray-400 shrink-0"/> {t('fileBrowser.rename')}
                  </button>
                  <button
                    onClick={(e) => {
                      onToggleStar(e, file.id, file.name);
                      setActiveMenuId(null);
                    }}
                    className="w-full flex items-center gap-2.5 px-4 py-2 text-[13px] text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-neutral-800 transition-colors cursor-pointer"
                  >
                     <Star size={14} className={`shrink-0 ${isItemStarred ? 'text-amber-500 fill-amber-500' : 'text-gray-400'}`}/> {isItemStarred ? t('fileBrowser.unstarResource') : t('fileBrowser.starResource')}
                  </button>

                  {onShare ? (
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        onShare(file);
                      }}
                      className="w-full flex items-center gap-2.5 px-4 py-2 text-[13px] text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-neutral-800 transition-colors cursor-pointer"
                    >
                      <Link2 size={14} className="text-gray-400 shrink-0" /> {t('fileBrowser.shareLink')}
                    </button>
                  ) : null}

                  {onMove ? (
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        onMove(file);
                      }}
                      className="w-full flex items-center gap-2.5 px-4 py-2 text-[13px] text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-neutral-800 transition-colors cursor-pointer"
                    >
                      <FolderInput size={14} className="text-gray-400 shrink-0" /> {t('fileBrowser.move')}
                    </button>
                  ) : null}

                  {onCopy ? (
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        onCopy(file);
                      }}
                      className="w-full flex items-center gap-2.5 px-4 py-2 text-[13px] text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-neutral-800 transition-colors cursor-pointer"
                    >
                      <Copy size={14} className="text-gray-400 shrink-0" /> {t('fileBrowser.copy')}
                    </button>
                  ) : null}

                  <div className="h-px bg-gray-100 dark:bg-neutral-800 my-1 mx-2" />
                </>
              )}
              
              {isTrashSection ? (
                <>
                  <button 
                    onClick={(e) => onTrashAction(e, file)}
                    className="w-full flex items-center gap-2.5 px-4 py-2 text-[13px] text-emerald-600 dark:text-emerald-400 hover:bg-emerald-50 dark:hover:bg-emerald-950/20 transition-colors cursor-pointer"
                  >
                     <RefreshCcw size={14} className="shrink-0"/> {t('fileBrowser.restore')}
                  </button>
                  <button 
                    onClick={(e) => onPermanentDelete(e, file)}
                    className="w-full flex items-center gap-2.5 px-4 py-2 text-[13px] text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/20 transition-colors cursor-pointer"
                  >
                     <Trash2 size={14} className="shrink-0"/> {t('fileBrowser.permanentDelete')}
                  </button>
                </>
              ) : !isComputerSection ? (
                <button 
                  onClick={(e) => onTrashAction(e, file)}
                  className="w-full flex items-center gap-2.5 px-4 py-2 text-[13px] text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/20 transition-colors cursor-pointer"
                >
                   <Trash size={14} className="text-red-400 shrink-0"/> {t('fileBrowser.delete')}
                </button>
              ) : null}
            </div>
          )}
        </div>
      </div>
      
      {/* Bottom File Title Text */}
      <div className="mt-2 min-w-0 w-full animate-in fade-in duration-100 flex flex-col justify-end flex-1">
         {isInlineEditing ? (
           <input
             type="text"
             defaultValue={file.name}
             autoFocus
             onClick={(e) => e.stopPropagation()}
             onDoubleClick={(e) => e.stopPropagation()}
             onKeyDown={(e) => {
               e.stopPropagation();
               if (e.key === 'Enter') {
                 onInlineRenameSubmit?.(e.currentTarget.value);
               } else if (e.key === 'Escape') {
                 onInlineRenameCancel?.();
               }
             }}
             onBlur={(e) => {
               onInlineRenameSubmit?.(e.target.value);
             }}
             className="bg-white dark:bg-[#202020] border-2 border-blue-500 dark:border-blue-500 rounded px-2 py-0.5 text-xs text-gray-900 dark:text-gray-100 font-semibold outline-none w-full shadow-sm focus:ring-0 mb-1"
           />
         ) : (
           <div className="flex items-center justify-between gap-1 group/name">
             <h4 className="text-[13px] font-medium text-neutral-800 dark:text-neutral-200 truncate group-hover:text-blue-500 dark:group-hover:text-blue-400 transition-colors mb-0.5" title={file.name}>
               {file.name}
             </h4>
             {!hideCloudFileActions && (
               <button
                 onClick={(e) => {
                   e.stopPropagation();
                   onRename(e, file);
                 }}
                 className="p-1 opacity-0 group-hover/name:opacity-100 hover:bg-gray-100 dark:hover:bg-[#282828] text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded transition-all cursor-pointer inline-flex shrink-0"
                 title={t('fileBrowser.rename')}
               >
                 <Edit2 size={11} />
               </button>
             )}
           </div>
         )}
         <div className="flex items-center justify-between text-[11px] text-gray-400 dark:text-neutral-500 font-mono mt-0.5 select-none">
           <span>{formatSize(file.size)}</span>
           <span>{formatDate(file.updatedAt).split(',')[0]}</span>
         </div>
      </div>
    </div>
  );
}
