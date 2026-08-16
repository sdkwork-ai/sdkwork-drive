# Drive Sibling Naming Standard

> Status: active  
> Authority: `sdkwork-specs/DRIVE_SPEC.md`, `sdkwork-utils` filename helpers

## Scope

Applies to all **active** sibling nodes under the same `(tenant, space, parent)`:

- folder create / inline create
- upload (`prepare_upload`)
- copy / archive extract (auto-rename)
- restore from trash (auto-rename before activate)
- move / rename (fail with HTTP 409 when the user-chosen name conflicts)

## Uniqueness rule

- Partial unique indexes `ux_dr_drive_node_*_name_live` apply only when `lifecycle_status = 'active'`.
- **Trashed** nodes do not reserve a name; users may create/upload a new item with the same name while the trashed copy remains in Trash.
- **Deleted** nodes never participate in uniqueness.

## Naming algorithm

Canonical implementation: `@sdkwork/utils` (`allocateUniqueSiblingName`) and `sdkwork-utils-rust` (`allocate_unique_display_name`).

| Input | First conflict | Pattern |
|-------|----------------|---------|
| `report.txt` | `report (1).txt` | `stem (N).ext` |
| `New folder` | `New folder (1)` | `name (N)` |
| Copy in same parent | `Copy of report.txt` | then numbered variants |

Numbering fills the **lowest free** `(N)` starting at `1`.

## Surface behavior

| Operation | Policy |
|-----------|--------|
| Upload | Server auto-rename; client toast when final name differs |
| New folder (default) | Client suggests unique default; server enforces |
| New folder (user-edited) | Client + server reject duplicate |
| Copy | Client passes resolved `nodeName`; server auto-resolves when omitted |
| Move | Reject if destination has active sibling with same name |
| Rename | Reject with 409 / `nameConflict` |
| Restore | Server renames conflicting nodes before activate |
| Archive extract | Server auto-rename extracted files; folder/path type conflicts still fail |

## API errors

Name conflicts return HTTP **409** `application/problem+json` with detail `node name already exists in parent` per `API_SPEC.md`.

Node mutation and read responses (`nodes.retrieve`, `nodes.update`, `nodes.move`, `nodes.copy`, `nodes.folders.create`, `nodes.files.create`, `trash.create`, `trash.restore`, `versions.restore`) return `SdkWorkApiResponse` with `data.item` (single `DriveNode`) or `data` (`CreateFileResponse` for file create). Space mutations (`spaces.create`, `spaces.retrieve`, `spaces.update`) use `data.item` (`DriveSpace`). Delete operations such as `spaces.delete` return HTTP `204` with no JSON body; `trash.empty` remains a command response with typed `data`. Archive list/extract use `data.items` + `pageInfo` or `data.extractedCount` respectively.

## Migration

`database/migrations/*/0006_drive_node_name_active_only.*` aligns indexes from `lifecycle_status != 'deleted'` to `lifecycle_status = 'active'`.
