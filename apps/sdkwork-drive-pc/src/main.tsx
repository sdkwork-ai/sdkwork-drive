import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { BrowserRouter, Route, Routes } from 'react-router-dom';
import { SdkworkSessionAuthBrowserRoot } from '@sdkwork/auth-pc-react';
import App from './App.tsx';
import { DriveErrorBoundaryShell } from './components/DriveErrorBoundaryShell.tsx';
import { createDrivePcRuntime } from './bootstrap/createDrivePcRuntime';
import { createBrowserPreferenceStorage } from './bootstrap/browserPreferenceStorage';
import { resolveDriveBootstrapMessages } from './bootstrap/resolveDriveBootstrapMessages';
import { renderBootstrapFailureMarkup } from './bootstrap/renderBootstrapFailure';
import {
  DrivePcPreferencesProvider,
  LanguageProvider,
  ThemeProvider,
} from 'sdkwork-drive-pc-commons';
import './index.css';

async function bootstrapDrivePcApp(): Promise<void> {
  const runtime = await createDrivePcRuntime();
  const rootElement = document.getElementById('root');
  if (!rootElement) {
    throw new Error('Drive PC root element was not found.');
  }
  const preferenceStorage = createBrowserPreferenceStorage();

  createRoot(rootElement).render(
    <StrictMode>
      <LanguageProvider preferenceStorage={preferenceStorage}>
        <ThemeProvider preferenceStorage={preferenceStorage}>
          <DrivePcPreferencesProvider preferenceStorage={preferenceStorage}>
            <DriveErrorBoundaryShell>
              <BrowserRouter>
                <SdkworkSessionAuthBrowserRoot>
                  <Routes>
                    <Route path="/*" element={<App runtime={runtime} />} />
                  </Routes>
                </SdkworkSessionAuthBrowserRoot>
              </BrowserRouter>
            </DriveErrorBoundaryShell>
          </DrivePcPreferencesProvider>
        </ThemeProvider>
      </LanguageProvider>
    </StrictMode>,
  );
}

void bootstrapDrivePcApp().catch((error: unknown) => {
  console.error('[sdkwork-drive-pc] bootstrap failed', error);
  const rootElement = document.getElementById('root');
  const detail = error instanceof Error ? error.message : String(error);
  const messages = resolveDriveBootstrapMessages();
  if (rootElement) {
    rootElement.innerHTML = renderBootstrapFailureMarkup(
      messages.bootstrapFailedTitle,
      messages.bootstrapFailedDesc,
      detail,
      messages.bootstrapReload,
    );
  }
});
