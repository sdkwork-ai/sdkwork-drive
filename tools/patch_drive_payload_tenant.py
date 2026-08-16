#!/usr/bin/env python3
import re
from pathlib import Path

ROUTES = Path(__file__).resolve().parents[1] / "crates/sdkwork-routes-drive-app-api/src/routes.rs"

ROUTE_HANDLERS = {
    "create_file",
    "create_shortcut",
    "prepare_uploader_upload",
    "mark_uploader_part_uploaded",
    "create_permission",
    "create_share_link",
    "create_comment",
    "create_comment_reply",
    "resolve_download_token",
}


def patch_handler(content: str, fn_name: str) -> str:
    pattern = rf"(async fn {fn_name}\([\s\S]*?\) -> [\s\S]*?\{{)"
    match = re.search(pattern, content)
    if not match:
        return content
    body_start = match.end()
    head = content[:body_start]
    rest = content[body_start:]
    if "resolve_tenant_text(payload.tenant_id)" in rest.split("async fn ", 1)[0]:
        body = rest
    else:
        insert = (
            "\n    let tenant_id = ctx.resolve_tenant_text(payload.tenant_id)?;\n"
            "    let operator_id = ctx.resolve_operator_text(payload.operator_id)?;\n"
        )
        body = insert + rest

    fn_body = body.split("\nasync fn ", 1)[0]
    fn_body = fn_body.replace("&payload.tenant_id", "&tenant_id")
    fn_body = fn_body.replace("payload.tenant_id.trim()", "tenant_id.as_str()")
    fn_body = fn_body.replace("&payload.operator_id", "&operator_id")
    fn_body = fn_body.replace("payload.operator_id.trim()", "operator_id.as_str()")
    fn_body = fn_body.replace(".bind(payload.operator_id.trim())", ".bind(operator_id.as_str())")
    fn_body = fn_body.replace("operator_id: payload.operator_id.trim().to_string()", "operator_id: operator_id.clone()")
    fn_body = fn_body.replace("tenant_id: payload.tenant_id.clone()", "tenant_id: tenant_id.clone()")

    tail = body.split("\nasync fn ", 1)
    if len(tail) == 2:
        return head + fn_body + "\nasync fn " + tail[1]
    return head + fn_body


def main() -> None:
    content = ROUTES.read_text(encoding="utf-8")
    for fn_name in ROUTE_HANDLERS:
        content = patch_handler(content, fn_name)
    ROUTES.write_text(content, encoding="utf-8")
    print("patched payload tenant resolution")

if __name__ == "__main__":
    main()
