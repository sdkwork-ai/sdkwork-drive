import React from 'react';
import { IconButton } from '@sdkwork/ui-mobile-react';
import { Folder, File as FileIcon, FileText, Image as ImageIcon, Video, MoreHorizontal, PieChart } from 'lucide-react';
import { motion } from 'motion/react';
import { CloudFile } from '../services/CloudDriveService';

interface CloudDriveFileItemProps {
  file: CloudFile;
  setActiveFile: (id: string) => void;
}

export const CloudDriveFileItem: React.FC<CloudDriveFileItemProps> = ({ file, setActiveFile }) => {
  const getFileIcon = (type: string) => {
    switch (type) {
      case "folder":
        return (
          <Folder className="w-6 h-6 text-yellow-500 fill-yellow-500/20" />
        );
      case "pdf":
        return <FileText className="w-6 h-6 text-rose-500" />;
      case "video":
        return <Video className="w-6 h-6 text-indigo-500" />;
      case "excel":
        return <PieChart className="w-6 h-6 text-emerald-500" />;
      case "image":
        return <ImageIcon className="w-6 h-6 text-blue-500" />;
      default:
        return <FileIcon className="w-6 h-6 text-slate-500" />;
    }
  };

  return (
    <motion.div
      whileTap={{ scale: 0.98 }}
      className="flex items-center gap-3 p-4 bg-white dark:bg-[#2c2d2e] rounded-xl cursor-pointer shadow-sm border border-border-color/30"
    >
      <div className="w-12 h-12 rounded-xl bg-gray-50 dark:bg-[#3a3b3c] flex items-center justify-center shrink-0">
        {getFileIcon(file.type)}
      </div>
      <div className="flex-1 min-w-0 pr-2">
        <div className="text-[15px] font-medium text-text-main truncate mb-1">
          {file.name}
        </div>
        <div className="flex items-center gap-2 text-[12px] text-text-sub font-mono">
          <span>{file.date}</span>
          <span className="w-1 h-1 rounded-full bg-border-color" />
          <span>{file.size}</span>
        </div>
      </div>
      <IconButton
        icon={<MoreHorizontal className="w-5 h-5 text-text-sub" />}
        className="w-8 h-8 -mr-2"
        onClick={async (e) => {
          e.stopPropagation();
          setActiveFile(file.id);
        }}
      />
    </motion.div>
  );
};
