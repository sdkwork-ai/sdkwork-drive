import React from 'react';
import { useTranslation } from 'react-i18next';
import { cn } from '@sdkwork/ui-mobile-react';
import { Clock, FolderOpen, CloudUpload, Zap } from 'lucide-react';

interface CloudDriveActionGridProps {
  activeTab: string;
  setActiveTab: (tab: "recent" | "files") => void;
  onUnavailableAction: () => void;
}

export const CloudDriveActionGrid: React.FC<CloudDriveActionGridProps> = ({
  activeTab,
  setActiveTab,
  onUnavailableAction,
}) => {
  const { t } = useTranslation();
const actions = [
    {
      icon: Clock,
      label: t('drive.actions.recent'),
      id: "recent",
      color: "text-blue-500",
      bg: "bg-blue-50",
    },
    {
      icon: FolderOpen,
      label: t('drive.actions.files'),
      id: "files",
      color: "text-amber-500",
      bg: "bg-amber-50",
    },
    {
      icon: CloudUpload,
      label: t('drive.actions.transfer'),
      id: "transfer",
      color: "text-emerald-500",
      bg: "bg-emerald-50",
    },
    {
      icon: Zap,
      label: t('drive.actions.quick'),
      id: "quick",
      color: "text-purple-500",
      bg: "bg-purple-50",
    },
  ];

  return (
    <div className="px-4 -mt-6 mb-4">
      <div className="bg-white dark:bg-[#2c2d2e] rounded-xl shadow-sm grid grid-cols-4 py-4 px-2">
        {actions.map((item) => (
          <div
            key={item.id}
            className="flex flex-col items-center gap-2 cursor-pointer group"
            onClick={() => {
              if (item.id === "recent" || item.id === "files") {
                setActiveTab(item.id);
                return;
              }
              onUnavailableAction();
            }}
          >
            <div
              className={cn(
                "w-12 h-12 rounded-2xl flex items-center justify-center transition-all duration-200",
                activeTab === item.id
                  ? `${item.color} ${item.bg} dark:bg-[#3a3b3c] shadow-sm scale-110`
                  : "bg-gray-50 dark:bg-[#3a3b3c] text-text-sub group-hover:scale-105 group-active:scale-95",
              )}
            >
              <item.icon className="w-6 h-6" />
            </div>
            <span
              className={cn(
                "text-[12px]",
                activeTab === item.id
                  ? "text-text-main font-medium"
                  : "text-text-sub",
              )}
            >
              {item.label}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
};
