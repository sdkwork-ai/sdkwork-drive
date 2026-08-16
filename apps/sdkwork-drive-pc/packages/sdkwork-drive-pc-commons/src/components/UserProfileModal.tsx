import React from 'react';
import {
  Cloud,
  Copy,
  Database,
  HardDrive,
  LogOut,
  Settings,
  ShieldCheck,
  Star,
} from 'lucide-react';
import { useTranslation } from './LanguageProvider';

export interface DriveSidebarAccount {
  id: string;
  displayName: string;
  email?: string;
  avatarUrl?: string;
  initials: string;
  tenantId?: string;
  organizationId?: string;
  sessionId?: string;
  environmentLabel?: string;
  authLevel?: string;
  planLabel?: string;
  storageUsedLabel?: string;
  storageTotalLabel?: string;
  storageUsagePercent?: number;
  storageObjectCount?: number;
}

interface UserProfileModalProps {
  isOpen: boolean;
  onClose: () => void;
  account: DriveSidebarAccount;
  onOpenSettings?: () => void;
  onOpenStarred?: () => void;
  onSignOut?: () => void | Promise<void>;
}

export function UserProfileModal({
  isOpen,
  onClose,
  account,
  onOpenSettings,
  onOpenStarred,
  onSignOut,
}: UserProfileModalProps) {
  const { t } = useTranslation();
  if (!isOpen) return null;

  const copyAccountId = async () => {
    if (typeof navigator === 'undefined' || !navigator.clipboard) {
      return;
    }
    await navigator.clipboard.writeText(account.id);
  };

  return (
    <>
      <button
        type="button"
        aria-label={t('settings.profileMenuClose')}
        className="fixed inset-0 z-[90] cursor-default bg-transparent"
        onClick={onClose}
      />
      <div
        role="dialog"
        aria-label={t('settings.profileMenuTitle')}
        className="absolute top-12 left-16 z-[110] w-80 overflow-hidden rounded-2xl border border-white/10 bg-[#1e1e1e] text-gray-100 shadow-2xl"
      >
        <div className="flex items-center gap-4 border-b border-white/5 p-5">
          <AccountAvatar account={account} sizeClassName="h-16 w-16 rounded-xl text-lg" />
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <h3 className="truncate text-lg font-semibold">{account.displayName}</h3>
              <ShieldCheck size={16} className="shrink-0 text-blue-400" />
            </div>
            {account.email && (
              <div className="mt-0.5 truncate text-xs text-gray-400">{account.email}</div>
            )}
            <button
              type="button"
              title={t('settings.copyAccountId')}
              onClick={() => void copyAccountId()}
              className="mt-2 flex max-w-full items-center gap-1.5 rounded text-left text-[11px] font-mono text-gray-500 transition-colors hover:text-gray-300"
            >
              <Copy size={12} />
              <span className="truncate">{account.id}</span>
            </button>
          </div>
        </div>

        <div className="grid grid-cols-2 gap-1 border-b border-white/5 p-2 text-center">
          <ProfileAction
            icon={<Star size={20} />}
            label={t('settings.profileStarred')}
            onClick={() => {
              onClose();
              onOpenStarred?.();
            }}
          />
          <ProfileAction
            icon={<Settings size={20} />}
            label={t('settings.profileSettings')}
            onClick={() => {
              onClose();
              onOpenSettings?.();
            }}
          />
        </div>

        <div className="space-y-1 p-2">
          <ProfileInfo icon={<Cloud size={15} />} label={t('settings.profilePlanVersion')} value={account.planLabel ?? 'Drive'} />
          <ProfileInfo
            icon={<HardDrive size={15} />}
            label={t('settings.profileStorage')}
            value={`${account.storageUsedLabel ?? '--'} / ${account.storageTotalLabel ?? '--'}`}
          />
          <ProfileInfo
            icon={<Database size={15} />}
            label={t('settings.profileRuntime')}
            value={account.environmentLabel ?? 'standard'}
          />
          <div className="mx-2 my-2 h-px bg-white/5" />
          <button
            type="button"
            onClick={() => {
              onClose();
              void onSignOut?.();
            }}
            className="flex w-full items-center gap-3 rounded-lg px-4 py-2.5 text-left text-sm text-red-400 transition-colors hover:bg-red-500/10 hover:text-red-300"
          >
            <LogOut size={16} />
            <span>{t('settings.signOut')}</span>
          </button>
        </div>
      </div>
    </>
  );
}

export function AccountAvatar({
  account,
  sizeClassName = 'h-8 w-8 rounded-full text-sm',
}: {
  account: Pick<DriveSidebarAccount, 'avatarUrl' | 'displayName' | 'initials'>;
  sizeClassName?: string;
}) {
  if (account.avatarUrl) {
    return (
      <img
        src={account.avatarUrl}
        alt={account.displayName}
        className={`${sizeClassName} object-cover`}
      />
    );
  }

  return (
    <div
      className={`${sizeClassName} flex items-center justify-center bg-gradient-to-br from-indigo-400 to-purple-500 font-bold text-white shadow-lg shadow-black/20`}
    >
      {account.initials}
    </div>
  );
}

function ProfileAction({
  icon,
  label,
  onClick,
}: {
  icon: React.ReactNode;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="flex flex-col items-center rounded-xl p-3 text-gray-400 transition-colors hover:bg-white/5 hover:text-gray-100"
    >
      {icon}
      <span className="mt-1 text-xs">{label}</span>
    </button>
  );
}

function ProfileInfo({
  icon,
  label,
  value,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
}) {
  return (
    <div className="flex items-center gap-3 rounded-lg px-4 py-2 text-xs text-gray-400">
      <span className="flex w-5 justify-center text-gray-500">{icon}</span>
      <span className="shrink-0">{label}</span>
      <span className="ml-auto min-w-0 truncate text-gray-300">{value}</span>
    </div>
  );
}
