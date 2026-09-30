import type {
  StorageProviderAccountScope,
  StorageProviderAccountView,
} from '../types/storageProviderAdminTypes';
import { BADGE_BASE_CLASS } from '../utils/uiPrimitives';
import { useTranslation } from '../hooks/useTranslation';

const SCOPE_LABEL_KEY: Record<StorageProviderAccountScope, string> = {
  user: 'accountScopeUser',
  tenant: 'accountScopeTenant',
  platform: 'accountScopePlatform',
};

const SCOPE_BADGE_CLASS: Record<StorageProviderAccountScope, string> = {
  platform: 'bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300',
  tenant: 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-300',
  user: 'bg-neutral-100 text-neutral-600 dark:bg-neutral-800 dark:text-neutral-300',
};

/**
 * The identity badges one account row carries: scope, the scope default star,
 * a non-production environment tag, and the disabled tag.
 *
 * Shared by the account picker dialog and the credential section's
 * selected-account summary row so an account reads the same way everywhere.
 */
export function StorageProviderAccountBadges({ account }: { account: StorageProviderAccountView }) {
  const { t } = useTranslation();
  return (
    <span className="inline-flex flex-wrap items-center gap-1">
      <span className={`${BADGE_BASE_CLASS} ${SCOPE_BADGE_CLASS[account.scopeType] ?? SCOPE_BADGE_CLASS.user} !px-1.5 !py-0 !text-[10px]`}>
        {t(SCOPE_LABEL_KEY[account.scopeType] ?? 'accountScopeTenant')}
      </span>
      {account.isDefault && (
        <span className={`${BADGE_BASE_CLASS} bg-amber-100 text-amber-700 !px-1.5 !py-0 !text-[10px] dark:bg-amber-900/40 dark:text-amber-300`}>
          ★ {t('accountDefaultBadge')}
        </span>
      )}
      {account.environment !== 'production' && (
        <span className={`${BADGE_BASE_CLASS} bg-neutral-100 text-neutral-500 !px-1.5 !py-0 !text-[10px] dark:bg-neutral-800 dark:text-neutral-400`}>
          {account.environment}
        </span>
      )}
      {account.status !== 'active' && (
        <span className={`${BADGE_BASE_CLASS} bg-red-100 text-red-600 !px-1.5 !py-0 !text-[10px] dark:bg-red-900/40 dark:text-red-300`}>
          {account.status === 'disabled' ? t('accountStatusDisabled') : account.status}
        </span>
      )}
    </span>
  );
}
