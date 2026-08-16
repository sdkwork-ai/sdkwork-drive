import type { SdkworkAuthAppearanceConfig, SdkworkAuthRuntimeConfig } from './driveAuthConfig';

// ── SdkworkIamAuthRoutes Props ────────────────────────────────────────────────
// Drive consumes the real auth-pc-react contracts; this shim only re-exports
// them so the app keeps a single import surface.

export type {
  SdkworkIamAuthRoutesProps,
} from '@sdkwork/auth-pc-react';
export {
  SdkworkIamAuthRoutes,
} from '@sdkwork/auth-pc-react';
export type {
  SdkworkSessionAuthBrowserRootProps,
} from '@sdkwork/auth-pc-react';
export {
  SdkworkSessionAuthBrowserRoot,
} from '@sdkwork/auth-pc-react';

// ── Extended IamAppContext with Drive-specific actor fields ───────────────────

declare module '@sdkwork/iam-contracts' {
  interface IamAppContext {
    /** Actor ID for impersonation audit trail (Drive-specific). */
    actorId?: string;
    /** Actor kind (user, admin, system) for audit trail (Drive-specific). */
    actorKind?: string;
  }
}
