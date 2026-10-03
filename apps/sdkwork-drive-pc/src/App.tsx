import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import {
  SettingsModal,
  SystemSidebar,
  type SettingsTab,
} from 'sdkwork-drive-pc-commons';
import {
  drivePathToSection,
  driveSectionToPath,
  isShareLinkClaimPath,
  parseShareLinkClaimToken,
  runDriveFileBrowserCommand,
  type DriveSection,
} from 'sdkwork-drive-pc-file';
import {
  DriveAuthGate,
  DriveRuntimeProvider,
  isDriveAbortError,
  isDriveAuthRoute,
  useDriveHostEffects,
  type DriveStorageSummary,
} from 'sdkwork-drive-pc-core';
import {
  useDrivePcPreferences,
  useTranslation,
} from 'sdkwork-drive-pc-commons';
import {
  canAccessAdminSection,
  resolveDriveAdminSectionAccess,
  type DriveAdminSectionAccess,
  type DriveRuntime,
} from 'sdkwork-drive-pc-admin-core';
import {
  createDriveAccountViewModel,
  signOutDriveAccount,
} from './bootstrap/driveAccountViewModel';
import type { DriveIamRuntime } from './bootstrap/driveIamRuntime';
import { DriveAuthShell } from './components/DriveAuthShell';
import {
  resolveDriveAuthAppearance,
  resolveDriveAuthLocale,
  resolveDriveAuthRuntimeConfig,
  type SdkworkAuthAppearanceConfig,
  type SdkworkAuthRuntimeConfig,
} from './bootstrap/driveAuthConfig';
import type { SdkworkIamAuthRoutesProps } from './bootstrap/sdkworkAuthPcReactShim';

const ADMIN_SECTION_ACCESS_KEYS: Record<string, keyof DriveAdminSectionAccess> = {
  'admin-storage-overview': 'storageOverview',
  'admin-storage-providers': 'storageProviders',
  'admin-storage-bindings': 'storageBindings',
  'admin-storage-kinds': 'storageKinds',
  'admin-storage-buckets': 'storageBuckets',
  'admin-audit': 'audit',
  'admin-maintenance': 'maintenance',
  'admin-quotas': 'quotas',
  'admin-labels': 'labels',
  'admin-spaces': 'spaces',
  'admin-download-packages': 'downloadPackages',
};

const DrivePage = React.lazy(() =>
  import('sdkwork-drive-pc-file').then((module) => ({ default: module.DrivePage })),
);
const StorageOverviewAdminPage = React.lazy(() =>
  import('sdkwork-drive-pc-admin-storage-providers').then((module) => ({
    default: module.StorageOverviewAdminPage,
  })),
);
const StorageProvidersAdminPage = React.lazy(() =>
  import('sdkwork-drive-pc-admin-storage-providers').then((module) => ({
    default: module.StorageProvidersAdminPage,
  })),
);
const StorageBindingsAdminPage = React.lazy(() =>
  import('sdkwork-drive-pc-admin-storage-providers').then((module) => ({
    default: module.StorageBindingsAdminPage,
  })),
);
const StorageProviderKindsAdminPage = React.lazy(() =>
  import('sdkwork-drive-pc-admin-storage-providers').then((module) => ({
    default: module.StorageProviderKindsAdminPage,
  })),
);
const StorageBucketsAdminPage = React.lazy(() =>
  import('sdkwork-drive-pc-admin-storage-buckets').then((module) => ({
    default: module.StorageBucketsAdminPage,
  })),
);
const AuditAdminPage = React.lazy(() =>
  import('sdkwork-drive-pc-admin-operations').then((module) => ({
    default: module.AuditAdminPage,
  })),
);
const MaintenanceAdminPage = React.lazy(() =>
  import('sdkwork-drive-pc-admin-operations').then((module) => ({
    default: module.MaintenanceAdminPage,
  })),
);
const QuotaAdminPage = React.lazy(() =>
  import('sdkwork-drive-pc-admin-operations').then((module) => ({
    default: module.QuotaAdminPage,
  })),
);
const LabelsAdminPage = React.lazy(() =>
  import('sdkwork-drive-pc-admin-operations').then((module) => ({
    default: module.LabelsAdminPage,
  })),
);
const SpacesAdminPage = React.lazy(() =>
  import('sdkwork-drive-pc-admin-operations').then((module) => ({
    default: module.SpacesAdminPage,
  })),
);
const DownloadPackagesAdminPage = React.lazy(() =>
  import('sdkwork-drive-pc-admin-operations').then((module) => ({
    default: module.DownloadPackagesAdminPage,
  })),
);
const SdkworkIamAuthRoutes = React.lazy(() =>
  import('@sdkwork/auth-pc-react').then((module) => ({ default: module.SdkworkIamAuthRoutes })),
);

export default function App({ runtime }: { runtime: DriveRuntime }) {
  const location = useLocation();
  const navigate = useNavigate();
  const activeSection = useMemo(
    () => drivePathToSection(location.pathname),
    [location.pathname],
  );
  const shareClaimToken = useMemo(
    () => parseShareLinkClaimToken(location.pathname),
    [location.pathname],
  );
  const setActiveSection = useCallback((section: DriveSection) => {
    navigate(driveSectionToPath(section));
  }, [navigate]);
  const [sessionSnapshot, setSessionSnapshot] = useState(() => runtime.session.getSnapshot());
  const [storageSummary, setStorageSummary] = useState<DriveStorageSummary | undefined>();
  const [storageSummaryUnavailable, setStorageSummaryUnavailable] = useState(false);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [settingsInitialTab, setSettingsInitialTab] = useState<SettingsTab>('account');

  useEffect(() => runtime.session.subscribe(setSessionSnapshot), [runtime.session]);

  useEffect(() => {
    if (isDriveAuthRoute(location.pathname)) {
      return;
    }
    if (isShareLinkClaimPath(location.pathname)) {
      return;
    }

    const canonicalPath = driveSectionToPath(activeSection);
    if (location.pathname !== canonicalPath) {
      navigate(canonicalPath, { replace: true });
    }
  }, [activeSection, location.pathname, navigate]);

  useEffect(() => {
    if (runtime.config.runtimeTarget !== 'desktop' && !runtime.host.isNativeHost && activeSection === 'computers') {
      setActiveSection('my-storage');
    }
  }, [runtime.config.runtimeTarget, runtime.host.isNativeHost, activeSection]);

  useEffect(() => {
    if (!sessionSnapshot.context?.tenantId || !sessionSnapshot.context?.userId) {
      setStorageSummary(undefined);
      setStorageSummaryUnavailable(false);
      return;
    }

    let active = true;
    const storageAbortController = new AbortController();
    setStorageSummaryUnavailable(false);
    runtime.services.fileService
      .getStorageSummary({
        signal: storageAbortController.signal,
      })
      .then((summary) => {
        if (active) {
          setStorageSummary(summary);
          setStorageSummaryUnavailable(false);
        }
      })
      .catch((err) => {
        if (isDriveAbortError(err)) {
          return;
        }
        if (active) {
          setStorageSummary(undefined);
          setStorageSummaryUnavailable(true);
        }
      });

    return () => {
      active = false;
      storageAbortController.abort();
    };
  }, [
    runtime.services.fileService,
    sessionSnapshot.context?.tenantId,
    sessionSnapshot.context?.userId,
  ]);

  const account = useMemo(
    () => createDriveAccountViewModel(sessionSnapshot, storageSummary),
    [sessionSnapshot, storageSummary],
  );
  const adminSectionAccess = useMemo(
    () => resolveDriveAdminSectionAccess(sessionSnapshot),
    [sessionSnapshot],
  );

  useEffect(() => {
    const accessKey = ADMIN_SECTION_ACCESS_KEYS[activeSection];
    if (accessKey && !canAccessAdminSection(sessionSnapshot, accessKey)) {
      setActiveSection('my-storage');
    }
  }, [activeSection, sessionSnapshot]);

  const handleSignOut = () => {
    void (async () => {
      try {
        await getDriveIamRuntime(runtime).service.auth.sessions.current.delete();
      } finally {
        await signOutDriveAccount(runtime.session, {
          tokenStorage: runtime.config.auth.tokenStorage,
        });
        if (typeof window !== 'undefined') {
          window.location.assign('/auth/login?redirect=%2F');
        }
      }
    })();
  };

  const openSettings = (tab: SettingsTab = 'account') => {
    setSettingsInitialTab(tab);
    setIsSettingsOpen(true);
  };

  /**
   * Desktop host effects: tray menu, global shortcut bindings, in-app key
   * fallback. Commands resolve through the shared command table so a tray click
   * and a shortcut press produce identical behavior.
   */
  const { preferences } = useDrivePcPreferences();
  const { t } = useTranslation();
  const trayLabels = useMemo(
    () => ({
      openSettings: t('settings.shortcut_openSettings_label'),
      showWindow: t('settings.shortcut_showWindow_label'),
      cutSelection: t('settings.shortcut_cutSelection_label'),
      pasteSelection: t('settings.shortcut_pasteSelection_label'),
      refresh: t('settings.shortcut_refresh_label'),
      toggleTray: t('settings.shortcut_toggleTray_label'),
    }),
    [t],
  );
  useDriveHostEffects({
    host: runtime.host,
    preferences,
    trayLabels,
    handlers: {
      openSettings: () => openSettings('shortcuts'),
      showWindow: () => {
        void runtime.host.tray.restoreWindow();
      },
      cutSelection: () => {
        runDriveFileBrowserCommand('cut');
      },
      pasteSelection: () => {
        runDriveFileBrowserCommand('paste');
      },
      refresh: () => {
        runDriveFileBrowserCommand('refresh');
      },
      toggleTray: () => {
        void runtime.host.tray.setVisible(!preferences.trayEnabled);
      },
    },
  });

  const getIamRuntime = useMemo(() => {
    return () => getDriveIamRuntime(runtime);
  }, [runtime]);

  const authRoutes = useMemo(() => (
    <DriveAuthShell>
      <DriveAppbaseAuthRouteHost getRuntime={getIamRuntime} />
    </DriveAuthShell>
  ), [getIamRuntime]);

  return (
    <DriveRuntimeProvider runtime={runtime}>
      <DriveAuthGate
            authRoutes={authRoutes}
            session={runtime.session}
          >
            <div className="flex h-dvh min-h-0 w-full min-w-0 overflow-hidden text-[#333] dark:text-[#eee] select-none font-sans bg-[#f5f5f5] dark:bg-[#111]">
              <SystemSidebar
                activeSection={activeSection}
                onSectionChange={setActiveSection}
                account={account}
                onSignOut={handleSignOut}
                isSettingsOpen={isSettingsOpen}
                onOpenSettings={openSettings}
                adminSectionAccess={adminSectionAccess}
              />
              <React.Suspense fallback={<DriveWorkspaceFallback />}>
                {adminSectionAccess.storageOverview && activeSection === 'admin-storage-overview' ? (
                  <StorageOverviewAdminPage
                    adminStorageSdkClient={runtime.admin.adminStorage}
                    getSession={runtime.session.getSnapshot}
                  />
                ) : adminSectionAccess.storageProviders && activeSection === 'admin-storage-providers' ? (
                  <StorageProvidersAdminPage
                    adminStorageSdkClient={runtime.admin.adminStorage}
                    getSession={runtime.session.getSnapshot}
                  />
                ) : adminSectionAccess.storageBindings && activeSection === 'admin-storage-bindings' ? (
                  <StorageBindingsAdminPage
                    adminStorageSdkClient={runtime.admin.adminStorage}
                    getSession={runtime.session.getSnapshot}
                    onManageProviders={() => setActiveSection('admin-storage-providers')}
                  />
                ) : adminSectionAccess.storageKinds && activeSection === 'admin-storage-kinds' ? (
                  <StorageProviderKindsAdminPage
                    adminStorageSdkClient={runtime.admin.adminStorage}
                    getSession={runtime.session.getSnapshot}
                  />
                ) : adminSectionAccess.storageBuckets && activeSection === 'admin-storage-buckets' ? (
                  <StorageBucketsAdminPage
                    adminStorageSdkClient={runtime.admin.adminStorage}
                    getSession={runtime.session.getSnapshot}
                    onCreateProvider={() => setActiveSection('admin-storage-providers')}
                    onConfigureProviders={() => setActiveSection('admin-storage-providers')}
                  />
                ) : adminSectionAccess.audit && activeSection === 'admin-audit' ? (
                  <AuditAdminPage
                    backendSdkClient={runtime.admin.backend}
                    getSession={runtime.session.getSnapshot}
                  />
                ) : adminSectionAccess.maintenance && activeSection === 'admin-maintenance' ? (
                  <MaintenanceAdminPage
                    backendSdkClient={runtime.admin.backend}
                    getSession={runtime.session.getSnapshot}
                  />
                ) : adminSectionAccess.quotas && activeSection === 'admin-quotas' ? (
                  <QuotaAdminPage
                    backendSdkClient={runtime.admin.backend}
                    getSession={runtime.session.getSnapshot}
                  />
                ) : adminSectionAccess.labels && activeSection === 'admin-labels' ? (
                  <LabelsAdminPage
                    backendSdkClient={runtime.admin.backend}
                    getSession={runtime.session.getSnapshot}
                  />
                ) : adminSectionAccess.spaces && activeSection === 'admin-spaces' ? (
                  <SpacesAdminPage
                    backendSdkClient={runtime.admin.backend}
                    getSession={runtime.session.getSnapshot}
                  />
                ) : adminSectionAccess.downloadPackages && activeSection === 'admin-download-packages' ? (
                  <DownloadPackagesAdminPage
                    backendSdkClient={runtime.admin.backend}
                    getSession={runtime.session.getSnapshot}
                  />
                ) : (
                  <DrivePage
                    activeSection={activeSection}
                    fileService={runtime.services.fileService}
                    storageSummary={storageSummary}
                    storageSummaryUnavailable={storageSummaryUnavailable}
                    onOpenExternal={runtime.host.openExternal}
                    onOpenStorageSettings={() => openSettings('storage')}
                    onSectionChange={setActiveSection}
                    shareClaimToken={shareClaimToken ?? undefined}
                    onShareClaimDismiss={() => navigate('/shared', { replace: true })}
                  />
                )}
              </React.Suspense>
              <SettingsModal
                isOpen={isSettingsOpen}
                initialTab={settingsInitialTab}
                onClose={() => setIsSettingsOpen(false)}
                account={account}
                onSignOut={handleSignOut}
                runtimeMode={runtime.config.runtimeTarget}
                appApiBaseUrl={runtime.config.appApiBaseUrl}
              />
            </div>
          </DriveAuthGate>
    </DriveRuntimeProvider>
  );
}

function DriveAppbaseAuthRouteHost({
  getRuntime,
}: {
  getRuntime: () => DriveIamRuntime;
}) {
  const authProps: SdkworkIamAuthRoutesProps = {
    appearance: resolveDriveAuthAppearance(),
    basePath: '/auth',
    getRuntime: getRuntime as unknown as SdkworkIamAuthRoutesProps['getRuntime'],
    homePath: '/',
    locale: resolveDriveAuthLocale() ?? undefined,
    runtimeConfig: resolveDriveAuthRuntimeConfig(),
    viewportMode: 'fixed',
  };

  return (
    <React.Suspense fallback={<DriveAuthRoutesFallback />}>
      <SdkworkIamAuthRoutes {...authProps} />
    </React.Suspense>
  );
}

function getDriveIamRuntime(runtime: DriveRuntime): DriveIamRuntime {
  const iamRuntime = runtime.auth?.iamRuntime;
  if (!iamRuntime) {
    throw new Error('Drive IAM runtime is not configured.');
  }
  return iamRuntime as DriveIamRuntime;
}

function DriveWorkspaceFallback() {
  return (
    <div
      aria-label="Loading Drive workspace"
      className="flex h-full min-h-0 w-full min-w-0 flex-1 items-center justify-center bg-white dark:bg-[#111]"
    >
      <div className="h-7 w-7 rounded-full border-2 border-blue-500 border-t-transparent animate-spin" />
    </div>
  );
}

function DriveAuthRoutesFallback() {
  return (
    <div
      aria-label="Loading Drive auth routes"
      className="sdkwork-drive-auth-loading"
    >
      <div className="h-7 w-7 rounded-full border-2 border-blue-500 border-t-transparent animate-spin" />
    </div>
  );
}
