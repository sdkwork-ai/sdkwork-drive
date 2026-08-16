import React, { useEffect, useState } from 'react';
import {
  Bell,
  HardDrive,
  Info,
  Languages,
  LogOut,
  Monitor,
  Moon,
  Shield,
  Sun,
  UserRound,
  X,
} from 'lucide-react';
import { useTheme } from './ThemeProvider';
import { useTranslation, type Language } from './LanguageProvider';
import { useDrivePcPreferences, type DrivePcPreferences } from './drivePcPreferences';
import type { DriveSidebarAccount } from './UserProfileModal';

export type SettingsTab = 'account' | 'general' | 'notifications' | 'security' | 'storage' | 'about';

interface SettingsModalProps {
  isOpen: boolean;
  initialTab?: SettingsTab;
  onClose: () => void;
  account?: DriveSidebarAccount;
  onSignOut?: () => void | Promise<void>;
  runtimeMode?: string;
  appApiBaseUrl?: string;
}

export function SettingsModal({
  isOpen,
  initialTab,
  onClose,
  account,
  onSignOut,
  runtimeMode,
  appApiBaseUrl,
}: SettingsModalProps) {
  const { theme, setTheme } = useTheme();
  const { language, setLanguage, t } = useTranslation();
  const { preferences, updatePreferences } = useDrivePcPreferences();
  const [activeTab, setActiveTab] = useState<SettingsTab>(initialTab ?? 'account');

  useEffect(() => {
    if (isOpen && initialTab) {
      setActiveTab(initialTab);
    }
  }, [initialTab, isOpen]);

  if (!isOpen) return null;

  const activeTitle: Record<SettingsTab, string> = {
    account: t('settings.account'),
    general: t('commons.general'),
    notifications: t('commons.notifications'),
    security: t('settings.security'),
    storage: t('commons.storage'),
    about: t('settings.about'),
  };

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/60 p-4 backdrop-blur-sm">
      <div
        role="dialog"
        aria-label={t('commons.settings')}
        className="flex h-[75vh] min-h-[520px] w-[900px] max-w-[calc(100vw-32px)] overflow-hidden rounded-2xl border border-white/10 bg-[#181818] text-gray-200 shadow-2xl"
      >
        <div className="flex w-[210px] shrink-0 flex-col border-r border-white/5 bg-[#1e1e1e]">
          <div className="flex h-[72px] items-center px-6">
            <span className="text-lg font-semibold tracking-wide">{t('commons.settings')}</span>
          </div>
          <div className="flex-1 space-y-1 overflow-y-auto px-3 py-2">
            <SettingsNavItem
              icon={<UserRound size={16} />}
              label={t('settings.account')}
              active={activeTab === 'account'}
              onClick={() => setActiveTab('account')}
            />
            <SettingsNavItem
              icon={<Languages size={16} />}
              label={t('commons.general')}
              active={activeTab === 'general'}
              onClick={() => setActiveTab('general')}
            />
            <SettingsNavItem
              icon={<Bell size={16} />}
              label={t('commons.notifications')}
              active={activeTab === 'notifications'}
              onClick={() => setActiveTab('notifications')}
            />
            <SettingsNavItem
              icon={<Shield size={16} />}
              label={t('settings.security')}
              active={activeTab === 'security'}
              onClick={() => setActiveTab('security')}
            />
            <SettingsNavItem
              icon={<HardDrive size={16} />}
              label={t('commons.storage')}
              active={activeTab === 'storage'}
              onClick={() => setActiveTab('storage')}
            />
            <SettingsNavItem
              icon={<Info size={16} />}
              label={t('settings.about')}
              active={activeTab === 'about'}
              onClick={() => setActiveTab('about')}
            />
          </div>
          <div className="border-t border-white/5 p-3">
            <button
              onClick={() => {
                onClose();
                void onSignOut?.();
              }}
              className="flex w-full items-center justify-center gap-2 rounded-xl px-3 py-2.5 text-sm font-medium text-red-400 transition-colors hover:bg-red-500/10 hover:text-red-300"
            >
              <LogOut size={16} />
              {t('settings.signOut')}
            </button>
          </div>
        </div>

        <div className="flex min-w-0 flex-1 flex-col bg-[#181818]">
          <div className="flex h-[72px] shrink-0 items-center justify-between border-b border-white/5 px-8">
            <h2 className="text-lg font-medium tracking-wide text-gray-100">
              {activeTitle[activeTab]}
            </h2>
            <button
              onClick={onClose}
              className="flex h-8 w-8 items-center justify-center rounded-full text-gray-400 transition-all hover:bg-white/10 hover:text-gray-100"
              title={t('commons.close')}
            >
              <X size={18} />
            </button>
          </div>

          <div className="flex-1 overflow-y-auto">
            <div className="w-full p-8 pb-16">
              {activeTab === 'account' && <AccountSettings account={account} t={t} />}
              {activeTab === 'general' && (
                <GeneralSettings
                  language={language}
                  setLanguage={setLanguage}
                  theme={theme}
                  setTheme={setTheme}
                  compactMode={preferences.compactMode}
                  onCompactModeChange={(checked) => updatePreferences({ compactMode: checked })}
                  t={t}
                />
              )}
              {activeTab === 'notifications' && (
                <NotificationSettings
                  preferences={preferences}
                  onPreferenceChange={updatePreferences}
                  t={t}
                />
              )}
              {activeTab === 'security' && (
                <SecuritySettings
                  account={account}
                  preferences={preferences}
                  onPreferenceChange={updatePreferences}
                  t={t}
                />
              )}
              {activeTab === 'storage' && <StorageSettings account={account} t={t} />}
              {activeTab === 'about' && (
                <AboutSettings runtimeMode={runtimeMode} appApiBaseUrl={appApiBaseUrl} t={t} />
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function AccountSettings({
  account,
  t,
}: {
  account?: DriveSidebarAccount;
  t: (key: string, params?: Record<string, string | number>) => string;
}) {
  return (
    <div className="space-y-6">
      <SettingsSection title={t('settings.accountProfileTitle')} description={t('settings.accountProfileDesc')}>
        <KeyValueRow label={t('settings.displayName')} value={account?.displayName ?? t('settings.defaultDisplayName')} />
        <KeyValueRow label={t('settings.email')} value={account?.email ?? '--'} />
        <KeyValueRow label={t('settings.accountId')} value={account?.id ?? '--'} mono />
        <KeyValueRow label={t('settings.tenantId')} value={account?.tenantId ?? '--'} mono />
        <KeyValueRow label={t('settings.organizationId')} value={account?.organizationId ?? '--'} mono />
      </SettingsSection>
      <SettingsSection title={t('settings.currentSessionTitle')} description={t('settings.currentSessionDesc')}>
        <KeyValueRow label={t('settings.sessionId')} value={account?.sessionId ?? '--'} mono />
        <KeyValueRow label={t('settings.authLevel')} value={account?.authLevel ?? 'standard'} />
        <KeyValueRow label={t('settings.runtimeEnvironment')} value={account?.environmentLabel ?? 'standard'} />
      </SettingsSection>
    </div>
  );
}

function GeneralSettings({
  language,
  setLanguage,
  theme,
  setTheme,
  compactMode,
  onCompactModeChange,
  t,
}: {
  language: Language;
  setLanguage: (language: Language) => void;
  theme: 'dark' | 'light' | 'system';
  setTheme: (theme: 'dark' | 'light' | 'system') => void;
  compactMode: boolean;
  onCompactModeChange: (checked: boolean) => void;
  t: (key: string, params?: Record<string, string | number>) => string;
}) {
  return (
    <div className="space-y-6">
      <SettingsSection title={t('commons.selectLanguage')} description={t('commons.languageDesc')}>
        <div className="grid grid-cols-2 gap-4">
          <LanguageCard
            active={language === 'en-US'}
            onClick={() => setLanguage('en-US')}
            title={t('commons.english')}
            localeCode="EN"
          />
          <LanguageCard
            active={language === 'zh-CN'}
            onClick={() => setLanguage('zh-CN')}
            title={t('commons.chinese')}
            localeCode="中文"
          />
        </div>
      </SettingsSection>
      <SettingsSection title={t('commons.themePreferences')} description={t('settings.themeSectionDesc')}>
        <div className="grid grid-cols-3 gap-3">
          <ThemeCard
            active={theme === 'light'}
            onClick={() => setTheme('light')}
            icon={<Sun size={20} />}
            title={t('commons.light')}
          />
          <ThemeCard
            active={theme === 'dark'}
            onClick={() => setTheme('dark')}
            icon={<Moon size={20} />}
            title={t('commons.dark')}
          />
          <ThemeCard
            active={theme === 'system'}
            onClick={() => setTheme('system')}
            icon={<Monitor size={20} />}
            title={t('commons.system')}
          />
        </div>
      </SettingsSection>
      <SettingsSection title={t('commons.compactMode')} description={t('commons.compactDescription')}>
        <NotificationToggle
          label={t('commons.enableCompact')}
          desc={t('settings.compactModeDescExtra')}
          checked={compactMode}
          onCheckedChange={onCompactModeChange}
        />
      </SettingsSection>
    </div>
  );
}

function NotificationSettings({
  preferences,
  onPreferenceChange,
  t,
}: {
  preferences: DrivePcPreferences;
  onPreferenceChange: (patch: Partial<DrivePcPreferences>) => void;
  t: (key: string, params?: Record<string, string | number>) => string;
}) {
  return (
    <SettingsSection title={t('commons.alertNotifications')} description={t('settings.notificationsSectionDesc')}>
      <div className="space-y-4">
        <NotificationToggle
          label={t('commons.transferStartAlert')}
          desc={t('commons.transferStartAlertDesc')}
          checked={preferences.transferStartAlert}
          onCheckedChange={(checked) => onPreferenceChange({ transferStartAlert: checked })}
        />
        <NotificationToggle
          label={t('commons.systemDialogVerification')}
          desc={t('commons.systemDialogVerificationDesc')}
          checked={preferences.systemDialogVerification}
          onCheckedChange={(checked) => onPreferenceChange({ systemDialogVerification: checked })}
        />
        <NotificationToggle
          label={t('commons.malwareCheckBanners')}
          desc={t('commons.malwareCheckBannersDesc')}
          checked={preferences.malwareCheckBanners}
          onCheckedChange={(checked) => onPreferenceChange({ malwareCheckBanners: checked })}
        />
      </div>
    </SettingsSection>
  );
}

function SecuritySettings({
  account,
  preferences,
  onPreferenceChange,
  t,
}: {
  account?: DriveSidebarAccount;
  preferences: DrivePcPreferences;
  onPreferenceChange: (patch: Partial<DrivePcPreferences>) => void;
  t: (key: string, params?: Record<string, string | number>) => string;
}) {
  return (
    <div className="space-y-6">
      <SettingsSection title={t('settings.loginDevicesTitle')} description={t('settings.loginDevicesDesc')}>
        <KeyValueRow label={t('settings.authMode')} value={account?.authLevel ?? 'standard'} />
        <KeyValueRow label={t('settings.currentDevice')} value={t('settings.driveDeviceLabel')} />
        <KeyValueRow label={t('settings.dataScope')} value={t('settings.dataScopeValue')} />
      </SettingsSection>
      <SettingsSection title={t('settings.securityPreferencesTitle')} description={t('settings.securityPreferencesDesc')}>
        <div className="space-y-4">
          <NotificationToggle
            label={t('settings.deleteShareConfirm')}
            desc={t('settings.deleteShareConfirmDesc')}
            checked={preferences.deleteShareConfirm}
            onCheckedChange={(checked) => onPreferenceChange({ deleteShareConfirm: checked })}
          />
          <NotificationToggle
            label={t('settings.previewCacheAutoClear')}
            desc={t('settings.previewCacheAutoClearDesc')}
            checked={preferences.previewCacheAutoClear}
            onCheckedChange={(checked) => onPreferenceChange({ previewCacheAutoClear: checked })}
          />
        </div>
      </SettingsSection>
    </div>
  );
}

function StorageSettings({
  account,
  t,
}: {
  account?: DriveSidebarAccount;
  t: (key: string, params?: Record<string, string | number>) => string;
}) {
  const storageUsedLabel = account?.storageUsedLabel ?? '--';
  const storageTotalLabel = account?.storageTotalLabel ?? '--';
  const storageUsagePercent = Math.min(100, Math.max(0, account?.storageUsagePercent ?? 0));

  return (
    <SettingsSection title={t('commons.drivePartitionDetails')} description={t('settings.storageSectionDesc')}>
      <div className="rounded-xl border border-white/5 bg-[#202020] p-5">
        <div className="mb-3 flex items-end justify-between gap-4">
          <div className="min-w-0">
            <div className="truncate text-sm font-semibold text-gray-100">
              {account?.planLabel ?? 'Drive'}
            </div>
            <div className="mt-1 text-xs text-gray-500">{t('settings.unifiedQuotaDesc')}</div>
          </div>
          <div className="shrink-0 text-sm font-semibold text-blue-400">
            {storageUsedLabel} / {storageTotalLabel}
          </div>
        </div>
        <div className="h-2 overflow-hidden rounded-full bg-white/10">
          <div
            className="h-full rounded-full bg-blue-500"
            style={{ width: `${storageUsagePercent}%` }}
          />
        </div>
      </div>
      <KeyValueRow label={t('commons.cachePackets')} value={t('settings.cacheUnavailable')} />
      <KeyValueRow label={t('commons.activeCompressedDir')} value={t('settings.cacheUnavailable')} />
      <KeyValueRow label={t('commons.quotaLimits')} value={storageTotalLabel} />
    </SettingsSection>
  );
}

function AboutSettings({
  runtimeMode,
  appApiBaseUrl,
  t,
}: {
  runtimeMode?: string;
  appApiBaseUrl?: string;
  t: (key: string, params?: Record<string, string | number>) => string;
}) {
  return (
    <div className="space-y-6">
      <SettingsSection title={t('settings.aboutAppTitle')} description={t('settings.aboutAppDesc')}>
        <KeyValueRow label={t('settings.application')} value="SDKWork Drive" />
        <KeyValueRow label={t('settings.runtimeMode')} value={runtimeMode ?? '--'} />
        <KeyValueRow label={t('settings.appApi')} value={appApiBaseUrl ?? '--'} mono />
        <KeyValueRow label={t('settings.identityIntegration')} value={t('settings.identityIntegrationValue')} />
      </SettingsSection>
      <SettingsSection title={t('settings.capabilitiesTitle')} description={t('settings.capabilitiesDesc')}>
        <KeyValueRow label={t('settings.fileManagement')} value={t('settings.fileManagementValue')} />
        <KeyValueRow label={t('settings.storageBackend')} value={t('settings.storageBackendValue')} />
        <KeyValueRow label={t('settings.syncStatus')} value={t('settings.syncStatusValue')} />
      </SettingsSection>
    </div>
  );
}

function SettingsNavItem({
  icon,
  label,
  active,
  onClick,
}: {
  icon: React.ReactNode;
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={`flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-left text-sm font-medium transition-all ${
        active
          ? 'bg-blue-600 text-white shadow-md shadow-blue-600/20'
          : 'text-gray-400 hover:bg-white/5 hover:text-gray-200'
      }`}
    >
      {icon}
      <span className="truncate">{label}</span>
    </button>
  );
}

function SettingsSection({
  title,
  description,
  children,
}: {
  title: string;
  description?: string;
  children: React.ReactNode;
}) {
  return (
    <section className="space-y-4">
      <div>
        <h3 className="text-sm font-semibold text-gray-100">{title}</h3>
        {description && <p className="mt-1 text-xs leading-5 text-gray-500">{description}</p>}
      </div>
      <div className="space-y-3">{children}</div>
    </section>
  );
}

function KeyValueRow({
  label,
  value,
  mono,
}: {
  label: string;
  value: string;
  mono?: boolean;
}) {
  return (
    <div className="flex items-center gap-4 rounded-xl border border-white/5 bg-[#202020] px-4 py-3">
      <span className="w-32 shrink-0 text-xs text-gray-500">{label}</span>
      <span className={`min-w-0 flex-1 truncate text-sm text-gray-200 ${mono ? 'font-mono' : ''}`}>
        {value}
      </span>
    </div>
  );
}

function ThemeCard({
  active,
  onClick,
  icon,
  title,
}: {
  active: boolean;
  onClick: () => void;
  icon: React.ReactNode;
  title: string;
}) {
  return (
    <button
      onClick={onClick}
      className={`flex h-20 w-full cursor-pointer flex-col items-center justify-center gap-1.5 rounded-xl border-2 transition-all ${
        active
          ? 'border-blue-500 bg-blue-500/10 font-semibold text-blue-400'
          : 'border-white/10 text-gray-400 hover:border-white/20 hover:text-gray-200'
      }`}
    >
      {icon}
      <span className="text-xs">{title}</span>
    </button>
  );
}

function LanguageCard({
  active,
  onClick,
  title,
  localeCode,
}: {
  active: boolean;
  onClick: () => void;
  title: string;
  localeCode: string;
}) {
  const { t } = useTranslation();
  return (
    <button
      onClick={onClick}
      className={`flex w-full cursor-pointer items-center gap-4 rounded-xl border-2 p-4 text-left transition-all ${
        active
          ? 'border-blue-500 bg-blue-500/10 font-bold text-blue-400'
          : 'border-white/10 text-gray-200 hover:border-white/20'
      }`}
    >
      <div
        className={`flex h-9 w-9 shrink-0 select-none items-center justify-center rounded-lg text-xs font-bold ${
          active ? 'bg-blue-500 text-white' : 'bg-neutral-800 text-gray-400'
        }`}
      >
        {localeCode}
      </div>
      <div className="min-w-0">
        <div className="truncate text-xs font-medium">{title}</div>
        <div className="mt-0.5 truncate text-[10px] text-gray-500">
          {localeCode === 'EN' ? t('commons.bilingualDescEn') : t('commons.bilingualDescZh')}
        </div>
      </div>
    </button>
  );
}

function NotificationToggle({
  label,
  desc,
  checked,
  onCheckedChange,
}: {
  label: string;
  desc: string;
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
}) {
  return (
    <label className="flex cursor-pointer select-none items-start justify-between gap-4 rounded-xl border border-white/5 bg-[#202020] px-4 py-3">
      <div className="space-y-0.5">
        <span className="block text-xs font-semibold text-gray-200">{label}</span>
        <span className="block text-[10px] text-gray-500">{desc}</span>
      </div>
      <div className="relative shrink-0 pt-0.5">
        <input
          type="checkbox"
          className="peer sr-only"
          checked={checked}
          onChange={(event) => onCheckedChange(event.target.checked)}
        />
        <div className="h-4 w-8 rounded-full bg-gray-700 peer-checked:bg-blue-500 after:absolute after:left-[2px] after:top-[4px] after:h-3 after:w-3 after:rounded-full after:border after:border-gray-300 after:bg-white after:transition-all after:content-[''] peer-checked:after:translate-x-full peer-checked:after:border-white" />
      </div>
    </label>
  );
}
