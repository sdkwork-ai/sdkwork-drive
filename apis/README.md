# Drive APIs

`apis/` contains Drive-owned API contract sources and materialized OpenAPI
inputs. It is the application API boundary governed by
`../sdkwork-specs/API_SPEC.md` and
`../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`.

Current layout:

- `open-api/drive/`: Drive open-api surface OpenAPI documents.
- `app-api/drive/`: Drive app-api surface OpenAPI documents.
- `backend-api/drive/`: Drive backend-api surface OpenAPI documents.

Generated SDK family workspaces and generated transport output remain under
`sdks/`. Do not place generated SDK packages, generated SDK `.sdkwork/`
control-plane files, or runtime server state under `apis/`.
