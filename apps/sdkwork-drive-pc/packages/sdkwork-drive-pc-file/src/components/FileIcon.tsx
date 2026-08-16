import React from 'react';
import { Folder as FolderIcon, File as DefaultFileIcon, FileText, Music } from 'lucide-react';

interface FileIconProps {
  type: string;
  mimeType?: string;
  color?: string;
}

export function FileIcon({ type, mimeType, color }: FileIconProps) {
  if (type === 'folder') {
    const colorConfigs: Record<string, { bg: string; text: string; border: string }> = {
      rose: { bg: 'bg-rose-50/70 dark:bg-rose-950/20', text: 'text-rose-500', border: 'border-rose-100 dark:border-rose-950/30' },
      emerald: { bg: 'bg-emerald-50/70 dark:bg-emerald-950/20', text: 'text-emerald-500', border: 'border-emerald-100 dark:border-emerald-950/30' },
      blue: { bg: 'bg-blue-50/75 dark:bg-blue-950/20', text: 'text-blue-500', border: 'border-blue-100 dark:border-blue-950/30' },
      violet: { bg: 'bg-violet-50/75 dark:bg-violet-950/20', text: 'text-violet-500', border: 'border-violet-100 dark:border-violet-950/30' },
      amber: { bg: 'bg-amber-50/75 dark:bg-amber-950/20', text: 'text-amber-500', border: 'border-amber-100 dark:border-amber-950/30' },
      red: { bg: 'bg-red-50/75 dark:bg-red-950/20', text: 'text-red-500', border: 'border-red-100 dark:border-red-950/30' },
      gray: { bg: 'bg-gray-50/75 dark:bg-gray-950/20', text: 'text-gray-500', border: 'border-gray-100 dark:border-gray-950/30' }
    };

    const matched = color ? colorConfigs[color] : null;
    if (matched) {
      return (
        <div className={`w-10 h-10 ${matched.bg} ${matched.text} flex items-center justify-center rounded-xl shrink-0 border ${matched.border} transition-all duration-300`}>
          <FolderIcon size={21} className="fill-current" />
        </div>
      );
    }

    return (
      <div className="w-10 h-10 bg-[#fef7e6] dark:bg-[#3d331d] flex items-center justify-center rounded-xl text-[#f39c12] dark:text-[#f3b040] shrink-0 border border-[#fbe9c8] dark:border-[#4d4022]">
        <FolderIcon size={21} className="fill-[#f39c12]" />
      </div>
    );
  }
  
  // Determine extension color coding
  if (mimeType?.includes('pdf')) return (
    <div className="w-10 h-10 bg-red-50 dark:bg-[#341b1b] flex items-center justify-center rounded-xl text-red-500 dark:text-red-400 shrink-0 border border-red-100 dark:border-[#4c2424]">
      <FileText size={20} />
    </div>
  );
  if (mimeType?.includes('audio')) return (
    <div className="w-10 h-10 bg-amber-50 dark:bg-[#322718] flex items-center justify-center rounded-xl text-amber-600 dark:text-amber-400 shrink-0 border border-amber-100 dark:border-[#4c3922]">
      <Music size={20} />
    </div>
  );
  if (mimeType?.includes('image')) return (
    <div className="w-10 h-10 bg-sky-50 dark:bg-[#1a2d3d] flex items-center justify-center rounded-xl text-sky-600 dark:text-sky-400 shrink-0 border border-sky-100 dark:border-[#223d53]">
      <DefaultFileIcon size={20} />
    </div>
  );
  if (mimeType?.includes('presentationml') || mimeType?.includes('powerpoint')) return (
    <div className="w-10 h-10 bg-orange-50 dark:bg-[#3d2919] flex items-center justify-center rounded-xl text-orange-600 dark:text-orange-400 shrink-0 border border-orange-100 dark:border-[#533821]">
      <FileText size={20} />
    </div>
  );
  if (mimeType?.includes('spreadsheetml') || mimeType?.includes('csv')) return (
    <div className="w-10 h-10 bg-emerald-50 dark:bg-[#1c3525] flex items-center justify-center rounded-xl text-emerald-600 dark:text-emerald-400 shrink-0 border border-emerald-100 dark:border-[#244b32]">
      <FileText size={20} />
    </div>
  );
  if (mimeType?.includes('zip')) return (
    <div className="w-10 h-10 bg-indigo-50 dark:bg-[#25223c] flex items-center justify-center rounded-xl text-indigo-600 dark:text-indigo-400 shrink-0 border border-indigo-100 dark:border-[#332e54]">
      <FileText size={20} />
    </div>
  );
  if (mimeType?.includes('video')) return (
    <div className="w-10 h-10 bg-purple-50 dark:bg-[#2d1b33] flex items-center justify-center rounded-xl text-purple-600 dark:text-purple-400 shrink-0 border border-purple-100 dark:border-[#3e2347]">
      <FileText size={20} />
    </div>
  );
  return (
    <div className="w-10 h-10 bg-gray-50 dark:bg-[#272727] flex items-center justify-center rounded-xl text-gray-500 dark:text-gray-400 shrink-0 border border-gray-100 dark:border-[#3a3a3a]">
      <FileText size={20} />
    </div>
  );
}
