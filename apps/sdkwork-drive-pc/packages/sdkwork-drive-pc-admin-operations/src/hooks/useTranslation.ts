import { useCallback, useContext, useMemo } from 'react';
import { LanguageProviderContext } from 'sdkwork-drive-pc-commons';

const fallbackSetLanguage = () => {};

export function useTranslation() {
  const context = useContext(LanguageProviderContext);
  const baseT = context?.t;
  const language = context?.language ?? 'en';
  const setLanguage = context?.setLanguage ?? fallbackSetLanguage;
  const t = useCallback(
    (key: string, params?: Record<string, string | number>) =>
      baseT ? baseT(`adminOperations.${key}`, params) : key,
    [baseT],
  );

  return useMemo(
    () => ({ t, language, setLanguage }),
    [language, setLanguage, t],
  );
}
