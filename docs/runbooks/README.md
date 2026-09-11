# runbooks

Operational runbooks for SDKWork Drive production and disaster recovery.

| Runbook | Purpose |
| --- | --- |
| [drive-production-operations.md](drive-production-operations.md) | Health probes, rate limits, tracing, upload policy |
| [drive-backup-disaster-recovery.md](drive-backup-disaster-recovery.md) | PostgreSQL backup, object storage, release rollback |

See `../../sdkwork-specs/DOCUMENTATION_SPEC.md` section 2.


<!-- scaffold-module-runbooks:index -->
## Docker 运维四件套（bin/ 标准，OPERATIONS_SPEC.md §7）

| Runbook | 内容 |
| --- | --- |
| [deploy.md](deploy.md) / [deploy.en.md](deploy.en.md) | 安装 / 升级 / 回滚 / 下线（bin/docker-deploy.sh + bin/docker-image.sh） |
| [troubleshooting.md](troubleshooting.md) / [troubleshooting.en.md](troubleshooting.en.md) | 症状 → doctor 检查 → 处置 |
| [backup-restore.md](backup-restore.md) / [backup-restore.en.md](backup-restore.en.md) | 备份 / 校验 / 恢复 / 演练（bin/backup.sh） |
| [log-reference.md](log-reference.md) / [log-reference.en.md](log-reference.en.md) | 健康日志特征与失败签名（bin/docker-deploy.sh logs） |
