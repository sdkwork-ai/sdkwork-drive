import React, { useState } from 'react';
import { 
  X, Megaphone, Palette, LineChart, Code, GraduationCap, 
  Building, ShoppingCart, Target, Shield, Sparkles, Brain, 
  Award, FolderKanban, Cpu, Globe, Rocket
} from 'lucide-react';
import { useTranslation } from 'sdkwork-drive-pc-commons';

interface CreateSharedSpaceModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (name: string, icon: string, color: string, description: string) => void;
}

export const SPACE_ICONS: Record<string, React.ComponentType<{ size?: number; className?: string }>> = {
  Megaphone,
  Palette,
  LineChart,
  Code,
  GraduationCap,
  Building,
  ShoppingCart,
  Target,
  Shield,
  Sparkles,
  Brain,
  Award,
  FolderKanban,
  Cpu,
  Globe,
  Rocket
};

export const SPACE_COLORS = [
  { name: 'rose', bg: 'bg-rose-500', text: 'text-rose-500', ring: 'ring-rose-500/30' },
  { name: 'blue', bg: 'bg-blue-500', text: 'text-blue-500', ring: 'ring-blue-500/30' },
  { name: 'emerald', bg: 'bg-emerald-500', text: 'text-emerald-500', ring: 'ring-emerald-500/30' },
  { name: 'violet', bg: 'bg-violet-500', text: 'text-violet-500', ring: 'ring-violet-500/30' },
  { name: 'amber', bg: 'bg-amber-500', text: 'text-amber-500', ring: 'ring-amber-500/30' },
  { name: 'pink', bg: 'bg-pink-500', text: 'text-pink-500', ring: 'ring-pink-500/30' },
  { name: 'cyan', bg: 'bg-cyan-500', text: 'text-cyan-500', ring: 'ring-cyan-500/30' },
  { name: 'indigo', bg: 'bg-indigo-500', text: 'text-indigo-500', ring: 'ring-indigo-500/30' }
];

export function CreateSharedSpaceModal({ isOpen, onClose, onSubmit }: CreateSharedSpaceModalProps) {
  const { t } = useTranslation();
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [selectedIcon, setSelectedIcon] = useState('Rocket');
  const [selectedColor, setSelectedColor] = useState('blue');

  if (!isOpen) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;
    onSubmit(name.trim(), selectedIcon, selectedColor, description.trim());
    
    setName('');
    setDescription('');
    setSelectedIcon('Rocket');
    setSelectedColor('blue');
  };

  const handleClose = () => {
    setName('');
    setDescription('');
    setSelectedIcon('Rocket');
    setSelectedColor('blue');
    onClose();
  };

  return (
    <div className="fixed inset-0 bg-black/65 backdrop-blur-sm z-[100] flex items-center justify-center p-4 animate-in fade-in duration-250 select-none" id="shared-space-modal-backdrop">
      <div className="bg-white dark:bg-[#1a1a1a] border border-gray-100 dark:border-neutral-800 rounded-2xl w-[480px] max-w-full p-6 shadow-2xl animate-in zoom-in-95 duration-200 overflow-y-auto max-h-[90vh] scrollbar-none" id="shared-space-modal-content">
        <div className="flex items-center justify-between mb-5">
          <div>
            <h3 className="text-md font-bold text-gray-900 dark:text-white">{t('sharedSpace.createTitle')}</h3>
            <p className="text-[11px] text-gray-400 dark:text-neutral-500 mt-0.5">{t('sharedSpace.createSubtitle')}</p>
          </div>
          <button onClick={handleClose} className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 cursor-pointer p-1 rounded-full hover:bg-gray-100 dark:hover:bg-neutral-800 transition-colors">
            <X size={18} />
          </button>
        </div>
        
        <form onSubmit={handleSubmit} className="space-y-5">
          <div>
            <label className="block text-xs font-bold text-gray-450 dark:text-neutral-400 uppercase tracking-wider mb-2">{t('sharedSpace.nameLabel')}</label>
            <input 
              type="text" 
              autoFocus
              required
              maxLength={40}
              placeholder={t('sharedSpace.namePlaceholder')}
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full bg-gray-50 dark:bg-neutral-900 border border-gray-200 dark:border-neutral-850 rounded-xl px-3.5 py-2.5 text-sm text-gray-800 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-neutral-600 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 focus:bg-white dark:focus:bg-[#111] transition-all"
            />
          </div>

          <div>
            <label className="block text-xs font-bold text-gray-450 dark:text-neutral-400 uppercase tracking-wider mb-2">{t('sharedSpace.descriptionLabel')}</label>
            <textarea 
              rows={2}
              maxLength={150}
              placeholder={t('sharedSpace.descriptionPlaceholder')}
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              className="w-full bg-gray-50 dark:bg-neutral-900 border border-gray-200 dark:border-neutral-850 rounded-xl px-3.5 py-2.5 text-sm text-gray-800 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-neutral-600 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 focus:bg-white dark:focus:bg-[#111] transition-all resize-none"
            />
          </div>

          <div>
            <label className="block text-xs font-bold text-gray-450 dark:text-neutral-400 uppercase tracking-wider mb-2">{t('sharedSpace.themeColorLabel')}</label>
            <div className="flex flex-wrap gap-2.5">
              {SPACE_COLORS.map((color) => {
                const isSelected = selectedColor === color.name;
                return (
                  <button
                    key={color.name}
                    type="button"
                    onClick={() => setSelectedColor(color.name)}
                    className={`w-7 h-7 rounded-full ${color.bg} cursor-pointer transition-all hover:scale-110 flex items-center justify-center border-2 border-transparent relative ${
                      isSelected ? 'scale-110 ring-4 ' + color.ring : ''
                    }`}
                    title={color.name}
                  >
                    {isSelected && (
                      <span className="w-1.5 h-1.5 rounded-full bg-white"></span>
                    )}
                  </button>
                );
              })}
            </div>
          </div>

          <div>
            <label className="block text-xs font-bold text-gray-450 dark:text-neutral-400 uppercase tracking-wider mb-2">{t('sharedSpace.iconLabel')}</label>
            <div className="grid grid-cols-8 gap-2 bg-gray-50 dark:bg-neutral-900 border border-gray-200/60 dark:border-neutral-850/60 p-3 rounded-xl max-h-[140px] overflow-y-auto">
              {Object.keys(SPACE_ICONS).map((iconName) => {
                const IconComponent = SPACE_ICONS[iconName];
                const isSelected = selectedIcon === iconName;
                const activeColor = SPACE_COLORS.find(c => c.name === selectedColor) || SPACE_COLORS[0];
                return (
                  <button
                    key={iconName}
                    type="button"
                    onClick={() => setSelectedIcon(iconName)}
                    className={`h-[38px] rounded-lg flex items-center justify-center border transition-all cursor-pointer hover:bg-white dark:hover:bg-neutral-850 hover:scale-105 ${
                      isSelected 
                        ? 'border-blue-500 bg-blue-50/20 dark:bg-blue-950/20 text-blue-600 dark:text-blue-400 ring-2 ring-blue-500/10 font-bold' 
                        : 'border-transparent text-gray-500 dark:text-neutral-400'
                    }`}
                    title={iconName}
                  >
                    <IconComponent size={20} className={isSelected ? activeColor.text : ''} />
                  </button>
                );
              })}
            </div>
          </div>

          <div className="flex items-center gap-3 pt-3">
            <button 
              type="button" 
              onClick={handleClose}
              className="flex-1 py-3 text-xs font-semibold text-gray-650 dark:text-gray-300 bg-gray-50 dark:bg-[#252525] border border-gray-150 dark:border-neutral-800 hover:bg-gray-100 dark:hover:bg-[#303030] rounded-xl transition-colors cursor-pointer"
            >
              {t('sharedSpace.cancel')}
            </button>
            <button 
              type="submit" 
              className="flex-1 py-3 text-xs font-bold text-white bg-blue-600 hover:bg-blue-750 rounded-xl transition-colors cursor-pointer shadow-md shadow-blue-500/10 hover:shadow-blue-500/20"
            >
              {t('sharedSpace.confirmCreate')}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
