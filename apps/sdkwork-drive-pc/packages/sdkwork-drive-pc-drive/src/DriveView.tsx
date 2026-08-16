import React, { useEffect, useMemo, useState } from 'react';
import './driveSurface.css';
import {
  DrivePcPreferencesProvider,
  LanguageProvider,
  type PreferenceStorage,
} from 'sdkwork-drive-pc-commons';
import {
  DriveRuntimeProvider,
  isDriveAbortError,
  type DriveStorageSummary,
} from 'sdkwork-drive-pc-core';
import {
  DrivePage,
  type DriveOpenRequest,
  type DriveSection,
} from 'sdkwork-drive-pc-file';

import { bindHostSessionToDriveStore } from './sessionBridge';
import { createHostManagedDriveRuntime } from './createHostManagedDriveRuntime';
import { tryGetDrivePcSdkPorts } from './sdkPorts';

function DriveWorkspaceFallback() {
  return (
    <div className="flex flex-1 items-center justify-center bg-[#f5f5f5] text-gray-500 dark:bg-[#111] dark:text-gray-400">
      Loading Drive...
    </div>
  );
}

function createBrowserPreferenceStorage(): PreferenceStorage | undefined {
  if (typeof window === 'undefined') {
    return undefined;
  }

  return {
    getItem(key) {
      return window.localStorage.getItem(key) ?? undefined;
    },
    setItem(key, value) {
      window.localStorage.setItem(key, value);
    },
  };
}

export interface DriveViewProps {
  openRequest?: DriveOpenRequest;
  onOpenRequestHandled?: (requestId: string) => void;
}

export const DriveView: React.FC<DriveViewProps> = ({
  openRequest,
  onOpenRequestHandled,
}) => {
  const runtime = useMemo(() => createHostManagedDriveRuntime(), []);
  const preferenceStorage = useMemo(() => createBrowserPreferenceStorage(), []);
  const hostLanguagePorts = useMemo(() => {
    const ports = tryGetDrivePcSdkPorts();
    if (!ports?.resolveHostLanguage || !ports.subscribeHostLanguage) {
      return null;
    }
    return {
      resolveHostLanguage: ports.resolveHostLanguage,
      subscribeHostLanguage: ports.subscribeHostLanguage,
    };
  }, []);
  const [activeSection, setActiveSection] = useState<DriveSection>('my-storage');
  const [storageSummary, setStorageSummary] = useState<DriveStorageSummary | undefined>();
  const [sessionSnapshot, setSessionSnapshot] = useState(() => runtime.session.getSnapshot());

  useEffect(() => {
    const unbindHostSession = bindHostSessionToDriveStore(runtime.session);
    const unsubscribeSession = runtime.session.subscribe(setSessionSnapshot);
    return () => {
      unbindHostSession();
      unsubscribeSession();
    };
  }, [runtime.session]);

  useEffect(() => {
    if (!sessionSnapshot.context?.tenantId || !sessionSnapshot.context?.userId) {
      setStorageSummary(undefined);
      return;
    }

    let active = true;
    const storageAbortController = new AbortController();
    runtime.services.fileService
      .getStorageSummary({
        signal: storageAbortController.signal,
      })
      .then((summary) => {
        if (active) {
          setStorageSummary(summary);
        }
      })
      .catch((error) => {
        if (isDriveAbortError(error)) {
          return;
        }
        if (active) {
          setStorageSummary(undefined);
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

  return (
    <DriveRuntimeProvider runtime={runtime}>
      <DrivePcPreferencesProvider preferenceStorage={preferenceStorage}>
        <LanguageProvider
          preferenceStorage={preferenceStorage}
          resolveHostLanguage={hostLanguagePorts?.resolveHostLanguage}
          subscribeHostLanguage={hostLanguagePorts?.subscribeHostLanguage}
        >
          <div className="flex h-full w-full flex-1 min-h-0 min-w-0 overflow-hidden bg-[#f5f5f5] text-[#333] dark:bg-[#111] dark:text-[#eee]">
            <React.Suspense fallback={<DriveWorkspaceFallback />}>
              <DrivePage
                activeSection={activeSection}
                fileService={runtime.services.fileService}
                openRequest={openRequest}
                storageSummary={storageSummary}
                onOpenExternal={runtime.host.openExternal}
                onOpenRequestHandled={onOpenRequestHandled}
                onSectionChange={setActiveSection}
              />
            </React.Suspense>
          </div>
        </LanguageProvider>
      </DrivePcPreferencesProvider>
    </DriveRuntimeProvider>
  );
};
