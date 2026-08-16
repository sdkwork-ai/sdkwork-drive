> Owner: SDKWork maintainers

# SDKWork Drive IAM Integration Standard

## Scope

Drive can run as a standalone SDKWork application or as an embeddable module inside
another SDKWork product. IAM integration must be owned by exactly one application
shell. Drive business packages must not force a second IAM runtime, second auth
route tree, or second copy of appbase packages into a host that already provides
IAM.

## Package Boundary

Drive packages are split into two integration layers:

```text
host application
  -> owns IAM appbase runtime, /auth/* routes, session lifecycle, logout, refresh
  -> passes session provider and Drive services into Drive module

Drive embeddable packages
  -> sdkwork-drive-pc-core
  -> @sdkwork/drive-file
  -> @sdkwork/drive-transfer
  -> @sdkwork/drive-commons
  -> @sdkwork/drive-types

Drive standalone shell
  -> apps/sdkwork-drive-pc/src/App.tsx
  -> apps/sdkwork-drive-pc/src/bootstrap/driveIamRuntime.ts
  -> owns DriveAuthGate in standalone mode
  -> mounts appbase IAM route host for /auth/*
```

Rules:

- `sdkwork-drive-pc-core` owns Drive runtime contracts, session snapshot shape,
  generated Drive App SDK access, auth gate helpers, host adapter, and service
  facade creation.
- `sdkwork-drive-pc-core` must not depend on `@sdkwork/auth-pc-react` or
  `@sdkwork/appbase-pc-react`.
- Drive feature packages must not depend on appbase IAM packages, create auth
  routes, call raw auth HTTP APIs, or manually build IAM tokens.
- React and React DOM are peer dependencies for embeddable Drive packages that
  render React components.
- The standalone PC app may declare IAM auth packages (`@sdkwork/auth-pc-react`,
  `@sdkwork/auth-runtime-pc-react`) and platform shell packages (`@sdkwork/appbase-pc-react`)
  as optional peer dependencies because it is an application shell, not an embeddable business
  package.
- The standalone PC app may alias local `sdkwork-iam` and `sdkwork-appbase` packages during development.
  The default local IAM root is
  `../../sdkwork-iam` from the app root. Override it with `SDKWORK_IAM_ROOT` only for an
  explicit nonstandard local layout.

## Integration Modes

## Runtime API Auth Boundary

App, backend, and admin-storage Drive routes must validate the same dual-token contract:
`Authorization: Bearer <auth_token>` and `Access-Token: <access_token>`. Services derive
tenant, user, actor, and scope context from verified token claims. Clients must not send AppContext projection headers such as `x-sdkwork-tenant-id`, `x-sdkwork-user-id`,
`x-sdkwork-actor-id`, or `x-sdkwork-actor-kind`.

Admin-storage `/healthz` is public; `/backend/v3/api/drive/storage/*` is protected. Dedicated admin SDKs must declare the same security contract in OpenAPI and must not bypass the shared Drive security crate with handwritten token parsing.

### Backend Web Framework IAM Resolver

Production assembly paths for Drive HTTP routers must finalize with
`wrap_router_with_web_framework_from_env`, which resolves sessions through
`sdkwork_iam_web_adapter::iam_web_request_context_resolver_from_env()`.

Rules:

- `build_router_with_database_config` and standalone-gateway embedded routers
  must call `build_protected_router_with_pool*` helpers; they must not hardcode
  `DefaultWebRequestContextResolver` in production paths.
- Sync test helpers may wrap routers with `DefaultWebRequestContextResolver`
  only for unit/integration tests in non-production profiles.
- Cloud Kubernetes Deployments must mount `sdkwork-drive-iam` secrets alongside
  Drive database secrets for app-api, backend-api, open-api, and admin-storage-api.
- Development fallback to unsigned JWT parsing is allowed only when
  `allows_dev_authentication_fallback()` is true and no IAM database pool is
  configured.

### Standalone Mode

Use standalone mode when Drive is the top-level product application.

Required behavior:

- The Drive shell mounts `DriveAuthGate` with `integrationMode="standalone"` or
  the default mode.
- Anonymous product-route visits redirect to `/auth/login?redirect=<target>`.
- `/auth/*` renders the SDKWork appbase IAM route host.
- The Drive shell does not implement `/app/v3/api/auth/*`,
  `/backend/v3/api/auth/*`, login, refresh, or current-session endpoints.
- The Drive App SDK wrapper injects `Authorization` and `Access-Token` from the centralized
  session store through the global TokenManager. It must not attach AppContext projection
  headers; the server resolves tenant/user context from dual-token claims.
- `createDriveIamRuntime` consumes
  `@sdkwork/auth-runtime-pc-react` `createSdkworkAppbasePcAuthRuntime(...)`,
  passes the generated appbase app SDK client, the Drive downstream SDK clients,
  the global Drive `TokenManager`, and a Drive session bridge into that factory.
  It must not import `@sdkwork/iam-sdk-adapter`, call `createIamSdkAdapters`, or
  call `createIamRuntime(...)` directly in product code.
- The IAM bridge writes `authToken`, `accessToken`, `refreshToken`, `sessionId`,
  user identity, and `IamAppContext` into the Drive session store.
- If login, registration, OAuth, or refresh returns only tokens, the IAM bridge
  must immediately call `sessions.current.retrieve()` through the generated app
  SDK/appbase runtime to hydrate `IamAppContext`. `DriveAuthGate` must not treat
  a token-only session as a complete Drive login.
- Profile-menu logout must call `runtime.service.auth.sessions.current.delete()`
  first and clear the Drive session in a finally block.
- Local demo sessions are not supported. Real backend and standalone PC modes
  require appbase IAM login and must not send `tenant-local-demo` or
  `user-local-demo` context to the generated Drive App SDK.

### Host-Managed Mode

Use host-managed mode when another application embeds Drive and already owns IAM.

Required behavior:

- The host application owns route guards, `/auth/*`, refresh, logout, current
  session, account switch, tenant switch, and appbase runtime.
- The host must pass a `SessionStore`-compatible session provider to Drive core or
  construct Drive services with a `getSession` function backed by the host
  session.
- Drive modules must render product UI without mounting a second auth route host.
- `DriveAuthGate` must be omitted or set to `integrationMode="host-managed"`.
- The host must avoid nesting Drive standalone `App.tsx`; it should consume Drive
  packages or service facades directly.

## Dependency Conflict Rules

- A host that already depends on `@sdkwork/auth-pc-react` or
  `@sdkwork/appbase-pc-react` must provide those packages at the application
  level. Drive embeddable packages must not pull their own copies.
- Drive package manifests must keep IAM appbase packages out of normal
  dependencies and out of feature packages entirely.
- Drive package manifests must keep React as a peer dependency where the package
  is intended for reuse by another React application.
- Generated Drive SDK output must remain generated-only. Missing auth or session
  capability must be fixed in `sdkwork-iam` app SDK/OpenAPI inputs, not
  inside Drive feature packages.
- Appbase IAM packages belong only to application shells. `sdkwork-drive-pc-core`
  may expose an opaque `auth.iamRuntime` slot on `DriveRuntime`, but it must not
  import appbase IAM React/runtime packages.

## App SDK IAM Contract

Drive app SDK declares `sdkwork-iam-app-sdk` as the dependency for appbase
IAM app operations needed by the PC auth route host:

- `auth.sessions.create`
- `auth.sessions.current.retrieve`
- `auth.sessions.current.delete`
- `auth.sessions.current.update`
- `auth.sessions.refresh`
- `auth.sessions.organizationSelection.create`
- `auth.sessions.tenantSelection.create` (required by IAM SDK ports; Drive PC maps
  this from `organizationSelection` until appbase app SDK emits the renamed surface)
- `auth.registrations.create`
- `auth.passwordResetRequests.create`
- `auth.passwordResets.create`
- `oauth.authorizationUrls.create`
- `oauth.sessions.create`
- `oauth.deviceAuthorizations.create`
- `oauth.deviceAuthorizations.retrieve`
- `oauth.deviceAuthorizations.scans.create`
- `oauth.deviceAuthorizations.passwordCompletions.create`
- `system.iam.runtime.retrieve`
- `system.iam.verificationPolicy.retrieve`
- `iam.users.current.retrieve`

Verification-code delivery and verification are not appbase app operations in
the current architecture. They must be consumed through the messaging app SDK
surface (`messaging.verificationCodes.create` and
`messaging.verificationCodes.verify`) or an approved appbase auth wrapper that
delegates to that injected messaging client.

These operations are composed from the appbase app OpenAPI into
`apis/app-api/drive/drive-app-api.openapi.json` by `tools/drive_openapi_export.mjs`
for runtime contract visibility. The Drive app SDK generator receives the
owner-only input after appbase dependency operations are removed. Do not add
these methods by hand to generated SDK output or by handwritten PC HTTP clients.

## Verification

Required checks:

- IAM bridge tests prove token/context/user mapping, logout clearing, generated
  app SDK adapter shape, and token-only login context hydration.
- PC architecture tests prove `DriveRuntime` has no `authRoutes` field.
- PC architecture tests prove core and feature packages do not depend on
  `@sdkwork/auth-pc-react` or `@sdkwork/appbase-pc-react`.
- Auth gate tests prove `host-managed` mode does not redirect to `/auth/login`
  and does not render a duplicate auth route tree.
- SDK wrapper tests prove Drive API calls still inject dual-token and AppContext
  headers from the provided session.
- `node tools\drive_sdk_generate.mjs --check --language typescript` proves the
  composed app OpenAPI remains valid and idempotent.
- `pnpm --dir apps\sdkwork-drive-pc build` proves the appbase IAM route host and
  aliased appbase packages are resolvable by the standalone PC shell.
- Workspace tests prove Rust services do not expose product-local auth routes.
- `pnpm check:architecture-alignment` proves production router builders call
  `wrap_router_with_web_framework_from_env` and IAM web bootstrap does not gate
  resolver wiring on Drive database env presence.
- OpenAPI contract tests prove app/backend/admin-storage Drive APIs require
  `AuthToken` and `AccessToken`, while open APIs remain public.

