# SDKWork Drive PC Admin Operations

Backend admin UI for audit logs, maintenance sweeps, quota policy, labels, spaces, and download packages. Consumes `@sdkwork/drive-backend-sdk` through `sdkwork-drive-pc-admin-core`.

## Routes

| Section | Path |
| --- | --- |
| Audit | `/admin/audit` |
| Maintenance | `/admin/maintenance` |
| Quotas | `/admin/quotas` |
| Labels | `/admin/labels` |
| Spaces | `/admin/spaces` |
| Download packages | `/admin/download-packages` |

## SDK operations

- `auditEvents.list`
- `maintenance.jobs.list`, `maintenance.*.start`
- `quotas.retrieve`, `quotas.update`
- `labels.list`, `labels.create`, `labels.delete`
- `spaces.admin.list`
- `downloadPackages.list`
