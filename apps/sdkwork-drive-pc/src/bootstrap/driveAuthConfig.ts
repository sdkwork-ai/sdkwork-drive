import type {
  SdkworkAuthAppearanceConfig,
  SdkworkAuthRuntimeConfig,
} from '@sdkwork/auth-pc-react';
export type {
  SdkworkAuthAppearanceConfig,
  SdkworkAuthRuntimeConfig,
};

const DRIVE_VERIFICATION_POLICY = {
  emailCodeLoginEnabled: false,
  emailRegistrationVerificationRequired: false,
  phoneCodeLoginEnabled: false,
  phoneRegistrationVerificationRequired: false,
};

export function resolveDriveAuthRuntimeConfig(): SdkworkAuthRuntimeConfig {
  return {
    leftRailMode: 'qr-only',
    loginMethods: ['password'],
    oauthLoginEnabled: false,
    oauthProviders: [],
    qrLoginEnabled: true,
    recoveryMethods: [],
    registerMethods: ['email', 'phone'],
    verificationPolicy: DRIVE_VERIFICATION_POLICY,
  };
}

export function resolveDriveAuthAppearance(): SdkworkAuthAppearanceConfig {
  return {
    asidePanelClassName: 'sdkwork-drive-auth-aside-panel',
    bodyClassName: 'sdkwork-drive-auth-body',
    contentContainerClassName: 'sdkwork-drive-auth-content',
    pageClassName: 'sdkwork-drive-auth-page',
    qrFrameClassName: 'sdkwork-drive-auth-qr-frame',
    shellClassName: 'sdkwork-drive-auth-card-shell',
    slotProps: {
      background: {
        className: 'sdkwork-drive-auth-background',
      },
      page: {
        className: 'sdkwork-drive-auth-page',
      },
      shell: {
        className: 'sdkwork-drive-auth-card-shell',
      },
    },
    theme: {
      asideCardBackgroundColor: 'var(--sdkwork-drive-auth-aside-card-bg)',
      asideCardBorderColor: 'var(--sdkwork-drive-auth-aside-card-border)',
      asidePanelBackgroundColor: 'var(--sdkwork-drive-auth-aside-bg)',
      asidePanelBorderColor: 'var(--sdkwork-drive-auth-aside-border)',
      asidePanelColor: 'var(--sdkwork-drive-auth-aside-text)',
      badgeBackgroundColor: 'var(--sdkwork-drive-auth-aside-badge-bg)',
      badgeTextColor: 'var(--sdkwork-drive-auth-aside-badge-text)',
      contentBackgroundColor: 'var(--sdkwork-drive-auth-content-bg)',
      contentBorderColor: 'transparent',
      contentTextColor: 'var(--sdkwork-drive-auth-content-text)',
      descriptionColor: 'var(--sdkwork-drive-auth-muted-text)',
      dividerColor: 'var(--sdkwork-drive-auth-divider)',
      fieldBackgroundColor: 'var(--sdkwork-drive-auth-field-bg)',
      fieldBorderColor: 'transparent',
      fieldPlaceholderColor: '#9ca3af',
      fieldTextColor: 'var(--sdkwork-drive-auth-content-text)',
      formMutedTextColor: 'var(--sdkwork-drive-auth-muted-text)',
      iconMutedColor: 'var(--sdkwork-drive-auth-muted-text)',
      labelColor: 'var(--sdkwork-drive-auth-content-text)',
      pageBackgroundColor: 'var(--sdkwork-drive-auth-bg)',
      qrFrameBackgroundColor: 'var(--sdkwork-drive-auth-qr-bg)',
      qrFrameBorderColor: 'transparent',
      shellBackgroundColor: 'var(--sdkwork-drive-auth-content-bg)',
      shellBorderColor: 'transparent',
      tabActiveBackgroundColor: 'transparent',
      tabActiveTextColor: 'var(--sdkwork-drive-auth-content-text)',
      tabBackgroundColor: 'transparent',
      tabInactiveTextColor: 'var(--sdkwork-drive-auth-muted-text)',
      titleColor: 'var(--sdkwork-drive-auth-content-text)',
    },
  };
}

export function resolveDriveAuthLocale(): string | null {
  if (typeof navigator === 'undefined') {
    return null;
  }
  const language = navigator.language.trim();
  return language || null;
}
