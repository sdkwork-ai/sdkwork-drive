import React, { useEffect, useState } from 'react';
import { X } from 'lucide-react';
import { hasSiblingNameConflict, useTranslation } from 'sdkwork-drive-pc-commons';

interface FolderModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (folderName: string) => void;
  existingNames?: string[];
}

export function FolderModal({ isOpen, onClose, onSubmit, existingNames = [] }: FolderModalProps) {
  const { t } = useTranslation();
  const [newFolderName, setNewFolderName] = useState('');
  const [nameError, setNameError] = useState<string | null>(null);

  useEffect(() => {
    if (!isOpen) {
      setNewFolderName('');
      setNameError(null);
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = newFolderName.trim();
    if (!trimmed) {
      return;
    }
    if (hasSiblingNameConflict(trimmed, existingNames)) {
      setNameError(t('fileBrowser.nameConflict'));
      return;
    }
    onSubmit(trimmed);
    setNewFolderName('');
    setNameError(null);
  };

  const handleClose = () => {
    setNewFolderName('');
    setNameError(null);
    onClose();
  };

  return (
    <div className="fixed inset-0 bg-black/65 backdrop-blur-sm z-[100] flex items-center justify-center p-4 animate-in fade-in duration-200" id="create-folder-modal-backdrop">
      <div className="bg-white dark:bg-[#1a1a1a] border border-gray-100 dark:border-neutral-800 rounded-2xl w-[360px] p-6 shadow-2xl animate-in zoom-in-95 duration-200" id="create-folder-modal-content">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-md font-bold text-gray-900 dark:text-white">{t('fileBrowser.createFolder')}</h3>
          <button onClick={handleClose} className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 cursor-pointer">
            <X size={18} />
          </button>
        </div>
        
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-xs font-semibold text-gray-400 dark:text-neutral-500 uppercase tracking-wider mb-1.5">{t('fileBrowser.folderName')}</label>
            <input 
              type="text" 
              autoFocus
              required
              placeholder={t('fileBrowser.folderNamePlaceholder')}
              value={newFolderName}
              onChange={(e) => {
                setNewFolderName(e.target.value);
                if (nameError) {
                  setNameError(null);
                }
              }}
              aria-invalid={nameError ? true : undefined}
              className={`w-full bg-gray-50 dark:bg-neutral-900 border rounded-lg px-3 py-2 text-sm text-gray-800 dark:text-gray-100 focus:outline-none focus:border-blue-500 focus:bg-white dark:focus:bg-[#1e1e1e] transition-all ${
                nameError
                  ? 'border-rose-500 dark:border-rose-400'
                  : 'border-gray-200 dark:border-neutral-800'
              }`}
            />
            {nameError ? (
              <p className="mt-1.5 text-xs text-rose-500 dark:text-rose-400">{nameError}</p>
            ) : null}
          </div>
          <div className="flex items-center gap-2.5 pt-2">
            <button 
              type="button" 
              onClick={handleClose}
              className="flex-1 py-3 text-xs font-semibold text-gray-500 dark:text-gray-300 bg-gray-50 dark:bg-[#252525] hover:bg-gray-100 dark:hover:bg-[#303030] rounded-lg transition-colors cursor-pointer"
            >
              {t('fileBrowser.cancel')}
            </button>
            <button 
              type="submit" 
              className="flex-1 py-3 text-xs font-bold text-white bg-blue-600 hover:bg-blue-700/90 rounded-lg transition-colors cursor-pointer shadow-md hover:shadow-blue-500/10"
            >
              {t('fileBrowser.create')}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
