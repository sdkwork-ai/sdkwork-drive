# SDKWork Drive 标准对齐审查报告

> 更新日期: 2026-07-08
> 审查标准: sdkwork-specs 全栈规范  
> 状态: **P0/P1 代码债务已修复 — 可进入受控试点；商业化 GA 依赖运营证据链**

---

## 一、对齐结论

| 维度 | 状态 | 验证命令 |
|------|:----:|----------|
| sdkwork-specs 仓库字典 | ✅ | `pnpm check:architecture-alignment` |
| sdkwork-web-framework | ✅ | 四个 `*-api` route crate + IAM resolver |
| sdkwork-database | ✅ | 双引擎 baseline + migrations（0002–0006）+ `pnpm db:validate` |
| API 输入/输出契约 | ✅ | `pnpm api:envelope:check` + `pnpm api:schema:check` |
| 分页（PAGINATION_SPEC） | ✅ | `check-pagination.mjs`；SQL 层分页 + Admin/客户端单页加载 + Drive 关键游标不透明化 |
| 部署契约 | ✅ | `pnpm deploy:validate`；`deployments/deploy.yaml` 仅使用 `cloud.production`、`cloud.development`、`standalone.production`、`standalone.development`；严格发布门禁使用 `SDKWORK_DEPLOY_VALIDATION=strict pnpm deploy:validate` |
| Drive SDK 消费 | ✅ | `check-app-sdk-consumer-imports.mjs` |
| Runtime topology v4 | ✅ | `pnpm topology:validate` + `pnpm test:sdkwork-command-dev-topology` |
| Drive Uploader composed facade | ✅ | `drive-uploader-client.contract.test.ts` + `driveFileService.test.ts`；兼容标准 Blob 与 jsdom FileReader 读取 |
| 上传完成 saga | ✅ | DB 失败补偿：session 重置 + 孤儿对象 best-effort 删除 |
| SQLite 写事务 | ✅ | `begin_transaction_sql()` + `install_any_schema` 注册 engine |
| 下载包 / 本地存储 OOM | ✅ | 文件夹 BFS 分批 LIMIT；local range seek 读取 |
| 归档同步解压 OOM | ✅ | ZIP 打开不再复制压缩包字节；同步提取先做条目计划校验，压缩包 64MiB、单条目 16MiB、选中总解压 64MiB 上限；写入前再用流式读取到 `std::io::sink()` 校验实际解压总量，逐文件读取写入 |
| 移动目标 / Admin 分页 | ✅ | move destinations 使用 `page_size`/`cursor` + 窗口化 BFS；Storage Providers Admin 使用 `data.items` + `pageInfo.nextCursor` 翻页 |
| 资产 API 退役路由 | ✅ | legacy upload 返回 `410 Gone`；错误方法返回 `405 Method Not Allowed` |
| 409 冲突检测 | ✅ | 数值 platform code + MoveCopy 全量 sibling 扫描 |

---

## 二、2026-07-07 至 2026-07-08 修复项（本轮）

| 项 | 处理 |
|----|------|
| 上传完成 saga | `recover_upload_completion_after_db_failure`：session→uploading + 存储对象 delete |
| upload_handlers 硬编码 `BEGIN` | 改用 `begin_transaction_sql()` |
| transaction engine 注册 | `register_installed_database_engine` 在 `install_any_schema` 结束时调用 |
| transaction 单元测试 | 修正为 `begin_transaction_sql_for_engine` 引擎语义测试 |
| download_packages 宽目录 OOM | 每父目录 LIMIT/OFFSET 分批拉取 |
| archive_entries 同步提取内存 | ZIP inspection 使用 `Cursor<&[u8]>` 避免压缩包复制；extract 先生成计划，再在任何 DB/对象存储写入前流式预检实际解压总量，拒绝超出同步预算的 ZIP bomb / 大文件条目；正式写入时仍逐个条目读取 |
| local_store range read | `File::seek` + bounded read，不再 `fs::read` 全文件 |
| move destination 假分页 / OOM 风险 | 使用 `page_size`/`cursor`，BFS 只保留当前页窗口；每个父目录按 SQL `LIMIT/OFFSET` 小批量读取，并拒绝 legacy query alias |
| Staging admin smoke 分页参数 | GET list/search smoke 请求统一使用 `page_size=20`，契约测试禁止 `pageSize=` |
| SDK generator stub 假生成器 | `tools/sdkwork_sdk_generator_stub.mjs` 改为 fail-closed tombstone，dependency-management 与契约测试禁止本地 stub 产出 SDK |
| maintenance upload cleanup | 每项包裹 `BEGIN IMMEDIATE`/`BEGIN` 事务 |
| auth policy startup panic | 改为 error log，不 panic |
| MoveCopyModal sibling 检测 | `listSiblingFileNames` 分页聚合 |
| Storage Providers Admin | `listProvidersPage` + 翻页 UI |
| Storage Providers 对象列表 | `data.items` + `pageInfo.nextCursor`；公共前缀以 `objectKind=prefix` 表达 |
| 删除类 HTTP 语义 | SDKWork-owned delete 操作返回 `204 No Content`，不再返回 JSON 成功体 |
| 资产 legacy upload / 错误方法 | `/app/v3/api/assets/{upload,presign,upload_sessions}` fail-closed 为 `410 Gone` + `Gone`；资产删除型子资源错误 `POST` 返回 `405 MethodNotAllowed`，不再保留生产 `501/not implemented` |
| 延期发布包 checksum 证据 | macOS DMG / Linux AppImage 延期包移除占位 checksum；`releaseBuildDeferred=true` 时 materialize / readiness / verify 均禁止保留伪造 checksum 字段，等待目标 runner 物化真实 SHA-256 |
| 严格 release-readiness fail-closed | `SDKWORK_RELEASE_VALIDATION=strict pnpm check:release-readiness` 对签名延期、跨平台包 checksum 延期、Catalog 媒体延期一律失败；默认开发模式仍仅告警，避免用伪证据绕过 GA 门禁 |
| 本地 release package 真实证据 | `pnpm release:package` 现在构建并物化 `web.zip`、Windows `app.zip`、standalone gateway `tar.gz`、Catalog 媒体 staging、release evidence 与 SBOM；`release:plan` 无参数不再触发 `scriptArgs` TypeError，gateway package 不再用 `--skip-build` 假设已有本地 release binary |
| K8s digest 严格部署门禁 | `check_drive_deployments.mjs` 支持 `--root` 与 `SDKWORK_DEPLOY_VALIDATION=strict` / `SDKWORK_RELEASE_VALIDATION=strict`；默认模式仅告警占位符，严格模式拒绝 `REPLACE_WITH_RELEASE_DIGEST` 和非 `@sha256:<64 hex>` 镜像引用 |
| deploy/topology profile 对齐 | `check_drive_deployments.mjs` 在 Drive 本地门禁中校验所有 `deployments/deploy.yaml` profiles：profile key 必须存在于 `specs/topology.spec.json#profileFiles`，`overrides.topology.profile` 必须匹配 profile key，`overrides.topology.env` 必须匹配并存在；退役的多段式 topology profile id 在负向测试中被拒绝 |
| 数据库 seed i18n 契约 | `database/seeds/seed.manifest.json` 补齐 `i18nVersion`、`fallbackLocale`、`localeSets`；当前无必需参考数据，common bootstrap seed 明确为 no-op 生命周期脚本 |
| PC authored i18n 目录 | `sdkwork-drive-pc-commons/src/i18n/{en-US,zh-CN}/drive/commons/*` 对齐 `I18N_SPEC.md` `<locale>/<domain>/<capability>/<fragment>`；运行时语言状态升级为 `en-US`/`zh-CN`，兼容迁移旧 `en`/`zh` 偏好值 |
| 409 冲突 | `isDriveConflictError` 识别 40901；上传队列专用 toast |
| Runtime topology v5 profile id | `specs/topology.spec.json` uses two-segment `<deploymentProfile>.<environment>` ids; `etc/topology/{standalone,cloud}.{development,production}.env` owns active profiles; `sdkwork-app` owns public lifecycle selection. |
| Drive API opaque cursor | `sdkwork-drive-contract::api::pagination_cursor` 统一编码 offset/change-sequence 游标；app-api、backend-api、storage-backend-api、permission store 拒绝裸数字 cursor，并按 cursor 类型隔离 offset 与 change sequence |
| Drive PC Admin cursor 消费 | Spaces Admin、Storage Providers Admin 不再构造数字 cursor；只消费服务端 `pageInfo.nextCursor` / `nextPageToken` |
| Drive Uploader Blob 读取 | composed uploader 新增 Blob 读取兼容层：标准运行时使用 `Blob.arrayBuffer()`，缺失时使用 `FileReader.readAsArrayBuffer()`；测试环境不再因 jsdom `File.slice()` 缺少 `arrayBuffer()` 中断真实上传流程 |

---

## 三、架构快照

```
apps/sdkwork-drive-pc          → composition 注册 + Drive App SDK 消费
crates/sdkwork-routes-*-api    → sdkwork-web-framework + IAM + 独立限流
crates/sdkwork-drive-workspace-service → store ports + begin_transaction_sql + outbox
crates/sdkwork-drive-install-worker    → leader 心跳 + 配额对账维护
database/                      → baseline + 0002–0006 migrations（双引擎）
sdks/                          → OpenAPI 权威 → composed SDK facade
```

---

## 四、生产上线门禁

| 条件 | 状态 |
|------|:----:|
| `pnpm check` | ✅ 本地通过（2026-07-08） |
| API envelope + schema | ✅ |
| PostgreSQL/SQLite lifecycle | ✅ baseline + 0002–0006 |
| 多实例 Redis 限流 | ✅ `redis-rate-limit` feature + K8s `SDKWORK_DRIVE_RATE_LIMIT_BACKEND=redis` + fail-closed Redis secret 门禁 |
| Outbox 双引擎 claim + 幂等 fan-out | ✅ |
| Cloud 多实例 outbox | ✅ K8s `SDKWORK_DRIVE_DOMAIN_OUTBOX_EMBEDDED_DISPATCH=false` |
| PostgreSQL 集成测试（CI） | ✅ |
| Deploy profile/env 对齐 | `deployments/deploy.yaml` production profiles align with `specs/topology.spec.json#profileFiles` and `etc/topology/*.env`; development profiles remain source/runtime concerns. |
| 本地 release package / validate | ✅ `pnpm release:package` + `pnpm release:validate` 已生成并校验本地 web、Windows、standalone gateway、Catalog staging、release evidence、SBOM |
| 严格 K8s digest 门禁 | ⏳ `SDKWORK_DEPLOY_VALIDATION=strict pnpm deploy:validate` 当前因 `REPLACE_WITH_RELEASE_DIGEST` 阻塞 |
| 严格 release-readiness 门禁 | ⏳ `SDKWORK_RELEASE_VALIDATION=strict pnpm check:release-readiness` 当前因签名、macOS/Linux checksum、Catalog CDN 媒体证据阻塞 |
| Staging admin smoke / e2e | ⏳ `pnpm smoke:staging-admin`（需 staging 密钥） |

### 部署要点

1. 多实例限流：云端 API Deployment 必须配置 `SDKWORK_DRIVE_RATE_LIMIT_BACKEND=redis`、`SDKWORK_DRIVE_RATE_LIMIT_FAIL_CLOSED=true`，并从 `sdkwork-drive-rate-limit` secret 注入 `SDKWORK_DRIVE_RATE_LIMIT_REDIS_URL`
2. 可信代理后：`SDKWORK_DRIVE_RATE_LIMIT_TRUST_PROXY=true`
3. 生产下载令牌：`SDKWORK_DRIVE_DOWNLOAD_TOKEN_HMAC_SECRET`
4. Cloud 多实例/容器部署：`SDKWORK_DRIVE_DOMAIN_OUTBOX_EMBEDDED_DISPATCH=false`
5. 上传内容策略：`SDKWORK_DRIVE_UPLOAD_CONTENT_POLICY_MODE=enforce`
6. SQLite 仅单实例/单写者；生产使用 PostgreSQL

### 验证命令

```bash
pnpm check
pnpm check:pnpm-script-standard
pnpm test:sdkwork-command-dev-topology
node --test tests/contract/sdkwork-command-dev-topology.contract.test.mjs
pnpm topology:validate
pnpm api:envelope:check
pnpm api:schema:check
node ../sdkwork-specs/tools/check-pagination.mjs --workspace .
node ../sdkwork-specs/tools/check-app-sdk-consumer-imports.mjs --workspace .
node tools/check_sdkwork_drive_architecture_alignment.mjs
cargo test -p sdkwork-drive-contract --test database_tooling_smoke package_scripts_select_postgres_by_default_and_sqlite_explicitly
cargo test -p sdkwork-drive-contract pagination_cursor -- --nocapture
cargo test -p sdkwork-routes-drive-app-api
cargo test -p sdkwork-routes-drive-backend-api
cargo test -p sdkwork-routes-storage-backend-api
cargo test -p sdkwork-routes-drive-app-api --test drive_routes asset -- --nocapture
pnpm --dir apps/sdkwork-drive-pc exec vitest run src/__tests__/drive-uploader-client.contract.test.ts --environment jsdom
pnpm --dir apps/sdkwork-drive-pc exec vitest run packages/sdkwork-drive-pc-core/src/services/driveFileService.test.ts --environment jsdom
pnpm --dir apps/sdkwork-drive-pc typecheck
node --test tools/check_drive_deployments.test.mjs
node --test tools/check_sdkwork_drive_release_readiness.test.mjs
pnpm release:package
pnpm release:validate
pnpm test:release-evidence
pnpm deploy:validate
cargo test -p sdkwork-drive-contract --test database_schema_parity
cargo check -p sdkwork-routes-drive-app-api
cargo check -p sdkwork-drive-workspace-service
```

严格发布阻塞验证：

```bash
SDKWORK_DEPLOY_VALIDATION=strict pnpm deploy:validate
SDKWORK_RELEASE_VALIDATION=strict pnpm check:release-readiness
```

当前默认 `pnpm deploy:validate` 已通过，且不再出现旧拓扑 profile 未列入 `topology.profileFiles` 的 warning。`pnpm release:package` 已能从本地构建链路物化 web、Windows、standalone gateway、Catalog staging、release evidence 与 SBOM，`pnpm release:validate` 通过。严格 deploy 仍会失败，直到 `deployments/kubernetes/drive-services.yaml` 中所有 `REPLACE_WITH_RELEASE_DIGEST` 替换为真实 `@sha256:<64 hex>` release digest；严格 release-readiness 现在会 fail-closed，并列出 6 个真实 GA 阻塞项：签名延期、macOS DMG checksum 延期、Linux AppImage checksum 延期、primary icon 已本地 staged 但 CDN 发布待完成、screenshot 已本地 staged 但 CDN 发布待完成、preview 已本地 staged 但 CDN 发布待完成。

---

## 五、商业化 GA 前运营项（代码外）

应用尚未上线。P0/P1 代码债务已在本轮修复；以下仍阻塞 `publish.status=ACTIVE`：

| 项 | 说明 |
|----|------|
| 制品签名 | CI `security.signatureRequired` |
| macOS / Linux checksum | 目标 runner 产出真实 DMG / AppImage 后写入 SHA-256 |
| Catalog 媒体 CDN | 当前已本地 staged；仍需上传到 CDN、验证公开 URL、清除 `catalogMediaDeferred` |
| K8s 生产 digest | 替换 `REPLACE_WITH_RELEASE_DIGEST`，并通过 `SDKWORK_DEPLOY_VALIDATION=strict pnpm deploy:validate` |
| Admin smoke / staging e2e | [pre-launch-checklist.md](../guides/operator/pre-launch-checklist.md) |

---

## 六、后续能力演进（产品 Non-Goal / P4）

| 项 | 触发条件 |
|----|----------|
| 高流量列表 keyset 化与深 offset 消除 | 租户规模 / 深 offset 性能证据；关键外部 cursor 已不透明化，后续只在压测证明 offset 成为瓶颈时迁移存储查询形态 |
| 内嵌 Office 预览 | 产品决策引入 OnlyOffice/Collabora 或 SaaS 预览服务 |
| 上传 AV 扫描 | 生产内容安全需求 |
| 桌面 delta sync | PRD P4 |
| Azure Blob 适配器 | 具体客户需求 |
| Redis 限流容量模型与压测阈值 | 多实例高并发压测证据 |

---

*本报告反映 2026-07-08 P0/P1 修复后的对齐状态。GA 运营项仍待执行。*
