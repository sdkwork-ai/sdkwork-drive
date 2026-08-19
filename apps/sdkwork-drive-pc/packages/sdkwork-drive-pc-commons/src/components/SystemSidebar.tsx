import React, { useState } from 'react';
import { HardDrive, Settings, Activity, ServerCog, Link2, ScrollText, Wrench, PieChart, Tags, LayoutGrid, Package, Database, FolderCog } from 'lucide-react';
import type { SettingsTab } from './SettingsModal';
import {
  AccountAvatar,
  type DriveSidebarAccount,
  UserProfileModal,
} from './UserProfileModal';
import { useTranslation } from './LanguageProvider';

export interface DriveAdminNavigationAccess {
  storageProviders: boolean;
  storageBindings: boolean;
  storageKinds: boolean;
  storageBuckets: boolean;
  audit: boolean;
  maintenance: boolean;
  quotas: boolean;
  labels: boolean;
  spaces: boolean;
  downloadPackages: boolean;
}

interface SystemSidebarProps {
  activeSection?: string;
  onSectionChange?: (section: any) => void;
  account: DriveSidebarAccount;
  onSignOut?: () => void | Promise<void>;
  isSettingsOpen?: boolean;
  onOpenSettings?: (tab?: SettingsTab) => void;
  adminSectionAccess?: DriveAdminNavigationAccess;
  runtimeMode?: string;
  appApiBaseUrl?: string;
}

export function SystemSidebar({
  activeSection = 'my-storage',
  onSectionChange,
  account,
  onSignOut,
  isSettingsOpen,
  onOpenSettings,
  adminSectionAccess,
}: SystemSidebarProps) {
  const [isProfileOpen, setIsProfileOpen] = useState(false);
  const { t } = useTranslation();
  const showAdminNavigation = adminSectionAccess
    ? Object.values(adminSectionAccess).some(Boolean)
    : false;

  const handleOpenSettings = () => {
    onOpenSettings?.();
  };

  const adminSections = new Set([
    'admin-storage-providers',
    'admin-storage-bindings',
    'admin-storage-kinds',
    'admin-storage-buckets',
    'admin-audit',
    'admin-maintenance',
    'admin-quotas',
    'admin-labels',
    'admin-spaces',
    'admin-download-packages',
  ]);
  const isStorageActive = activeSection !== 'transfer' && !adminSections.has(activeSection);
  const isTransferActive = activeSection === 'transfer';
  const isAdminStorageProvidersActive = activeSection === 'admin-storage-providers';
  const isAdminStorageBindingsActive = activeSection === 'admin-storage-bindings';
  const isAdminStorageKindsActive = activeSection === 'admin-storage-kinds';
  const isAdminStorageBucketsActive = activeSection === 'admin-storage-buckets';
  const isAdminAuditActive = activeSection === 'admin-audit';
  const isAdminMaintenanceActive = activeSection === 'admin-maintenance';
  const isAdminQuotasActive = activeSection === 'admin-quotas';
  const isAdminLabelsActive = activeSection === 'admin-labels';
  const isAdminSpacesActive = activeSection === 'admin-spaces';
  const isAdminDownloadPackagesActive = activeSection === 'admin-download-packages';

  return (
    <>
    <div className="w-[60px] h-full bg-zinc-100 dark:bg-[#2e2e2e] flex flex-col items-center py-6 gap-8 select-none shrink-0 z-50 border-r border-black/5 dark:border-transparent">
      <div className="flex flex-col gap-5 w-full items-center">
        {/* User / App Avatar */}
        <button
          type="button"
          className="mb-2 rounded-lg transition-transform hover:scale-105 focus:outline-none focus:ring-2 focus:ring-blue-500/60"
          onClick={() => setIsProfileOpen(true)}
          title={t('settings.profileMenuTitle')}
        >
          <AccountAvatar account={account} sizeClassName="h-10 w-10 rounded-lg text-sm" />
        </button>
        
        {/* Navigation Routes */}
        <SidebarIcon 
          icon={<HardDrive size={22} />} 
          title={t('sidebar.myStorage')} 
          active={isStorageActive} 
          onClick={() => onSectionChange?.('my-storage')}
        />
        <SidebarIcon
          icon={<Activity size={22} />}
          title={t('sidebar.transferCenter')}
          active={isTransferActive}
          onClick={() => onSectionChange?.('transfer')}
        />
        {showAdminNavigation ? (
          <>
            {adminSectionAccess?.storageProviders ? (
            <SidebarIcon
              icon={<ServerCog size={22} />}
              title={t('sidebar.adminStorageProviders')}
              active={isAdminStorageProvidersActive}
              onClick={() => onSectionChange?.('admin-storage-providers')}
            />
            ) : null}
            {adminSectionAccess?.storageBindings ? (
            <SidebarIcon
              icon={<Link2 size={22} />}
              title={t('sidebar.adminStorageBindings')}
              active={isAdminStorageBindingsActive}
              onClick={() => onSectionChange?.('admin-storage-bindings')}
            />
            ) : null}
            {adminSectionAccess?.storageKinds ? (
            <SidebarIcon
              icon={<Database size={22} />}
              title={t('sidebar.adminStorageKinds')}
              active={isAdminStorageKindsActive}
              onClick={() => onSectionChange?.('admin-storage-kinds')}
            />
            ) : null}
            {adminSectionAccess?.storageBuckets ? (
            <SidebarIcon
              icon={<FolderCog size={22} />}
              title={t('sidebar.adminStorageBuckets')}
              active={isAdminStorageBucketsActive}
              onClick={() => onSectionChange?.('admin-storage-buckets')}
            />
            ) : null}
            {adminSectionAccess?.audit ? (
            <SidebarIcon
              icon={<ScrollText size={22} />}
              title={t('sidebar.adminAudit')}
              active={isAdminAuditActive}
              onClick={() => onSectionChange?.('admin-audit')}
            />
            ) : null}
            {adminSectionAccess?.maintenance ? (
            <SidebarIcon
              icon={<Wrench size={22} />}
              title={t('sidebar.adminMaintenance')}
              active={isAdminMaintenanceActive}
              onClick={() => onSectionChange?.('admin-maintenance')}
            />
            ) : null}
            {adminSectionAccess?.quotas ? (
            <SidebarIcon
              icon={<PieChart size={22} />}
              title={t('sidebar.adminQuotas')}
              active={isAdminQuotasActive}
              onClick={() => onSectionChange?.('admin-quotas')}
            />
            ) : null}
            {adminSectionAccess?.labels ? (
            <SidebarIcon
              icon={<Tags size={22} />}
              title={t('sidebar.adminLabels')}
              active={isAdminLabelsActive}
              onClick={() => onSectionChange?.('admin-labels')}
            />
            ) : null}
            {adminSectionAccess?.spaces ? (
            <SidebarIcon
              icon={<LayoutGrid size={22} />}
              title={t('sidebar.adminSpaces')}
              active={isAdminSpacesActive}
              onClick={() => onSectionChange?.('admin-spaces')}
            />
            ) : null}
            {adminSectionAccess?.downloadPackages ? (
            <SidebarIcon
              icon={<Package size={22} />}
              title={t('sidebar.adminDownloadPackages')}
              active={isAdminDownloadPackagesActive}
              onClick={() => onSectionChange?.('admin-download-packages')}
            />
            ) : null}
          </>
        ) : null}
      </div>
      <div className="flex flex-col gap-5 w-full items-center mt-auto mb-4">
         <SidebarIcon 
           sidebarIconId="sidebar-settings-button"
           icon={<Settings size={22} />} 
           title={t('commons.settings')} 
           active={isSettingsOpen}
           onClick={handleOpenSettings}
         />
      </div>
    </div>
    
    <UserProfileModal
      isOpen={isProfileOpen}
      onClose={() => setIsProfileOpen(false)}
      account={account}
      onOpenSettings={handleOpenSettings}
      onOpenStarred={() => onSectionChange?.('starred')}
      onSignOut={onSignOut}
    />
    </>
  );
}

interface SidebarIconProps {
  icon: React.ReactNode;
  title: string;
  active?: boolean;
  onClick?: () => void;
  sidebarIconId?: string;
}

function SidebarIcon({ icon, title, active, onClick, sidebarIconId }: SidebarIconProps) {
  return (
    <button 
      id={sidebarIconId}
      title={title} 
      onClick={onClick}
      className={`w-10 h-10 rounded-lg flex items-center justify-center cursor-pointer transition-all ${
        active 
          ? 'bg-white/10 text-white opacity-100 shadow-sm' 
          : 'text-white/60 hover:text-white hover:bg-white/5 active:bg-white/10'
      }`}
    >
      {icon}
    </button>
  );
}
