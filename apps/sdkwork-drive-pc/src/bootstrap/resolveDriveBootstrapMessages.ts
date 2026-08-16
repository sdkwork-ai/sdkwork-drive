import enUS from '../../packages/sdkwork-drive-pc-commons/src/i18n/en-US/drive/commons/settings';
import zhCN from '../../packages/sdkwork-drive-pc-commons/src/i18n/zh-CN/drive/commons/settings';

export type DriveBootstrapMessages = {
  bootstrapFailedTitle: string;
  bootstrapFailedDesc: string;
  bootstrapReload: string;
};

type BootstrapLanguage = 'en-US' | 'zh-CN';

function normalizeBootstrapLanguage(value: string | null): BootstrapLanguage | undefined {
  if (!value) {
    return undefined;
  }
  const lower = value.toLowerCase();
  if (lower === 'en' || lower === 'en-us' || lower.startsWith('en-')) {
    return 'en-US';
  }
  if (lower === 'zh' || lower === 'zh-cn' || lower.startsWith('zh-')) {
    return 'zh-CN';
  }
  return undefined;
}

function resolveBootstrapLanguage(): BootstrapLanguage {
  if (typeof window === 'undefined') {
    return 'en-US';
  }
  const saved = window.localStorage.getItem('sdkwork-ui-language');
  const normalizedSaved = normalizeBootstrapLanguage(saved);
  if (normalizedSaved) {
    if (saved !== normalizedSaved) {
      window.localStorage.setItem('sdkwork-ui-language', normalizedSaved);
    }
    return normalizedSaved;
  }
  const browserLang = window.navigator.language.toLowerCase();
  return browserLang.startsWith('zh') ? 'zh-CN' : 'en-US';
}

export function resolveDriveBootstrapMessages(): DriveBootstrapMessages {
  const locale = resolveBootstrapLanguage() === 'zh-CN' ? zhCN : enUS;
  return {
    bootstrapFailedTitle: locale.bootstrapFailedTitle,
    bootstrapFailedDesc: locale.bootstrapFailedDesc,
    bootstrapReload: locale.bootstrapReload,
  };
}
