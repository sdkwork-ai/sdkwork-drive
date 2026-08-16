#!/usr/bin/env python3
import re
from pathlib import Path

ROUTES = Path(__file__).resolve().parents[1] / "crates/sdkwork-routes-drive-app-api/src/routes.rs"
DOWNLOAD = Path(__file__).resolve().parents[1] / "crates/sdkwork-routes-drive-app-api/src/download_packages.rs"

ROUTE_HANDLERS = {
    "list_spaces",
    "create_space",
    "get_space",
    "update_space",
    "delete_space",
    "create_upload_session",
    "get_upload_session",
    "prepare_uploader_upload",
    "mark_uploader_part_uploaded",
    "list_nodes",
    "create_folder",
    "create_file",
    "create_shortcut",
    "get_node",
    "get_node_path",
    "get_node_capabilities",
    "list_archive_entries",
    "extract_archive_entries",
    "list_node_properties",
    "set_node_property",
    "delete_node_property",
    "list_node_labels",
    "apply_node_label",
    "remove_node_label",
    "update_node",
    "move_node",
    "copy_node",
    "delete_node",
    "create_node_download_url",
    "trash_node",
    "restore_trashed_node",
    "list_trashed_nodes",
    "list_recent_nodes",
    "list_shared_with_me",
    "list_favorite_nodes",
    "get_quota_summary",
    "set_favorite",
    "unset_favorite",
    "empty_trash",
    "list_versions",
    "get_version",
    "restore_version",
    "delete_version",
    "list_permissions",
    "get_permission",
    "list_effective_permissions",
    "create_permission",
    "update_permission",
    "delete_permission",
    "list_share_links",
    "get_share_link",
    "create_share_link",
    "update_share_link",
    "revoke_share_link",
    "list_comments",
    "get_comment",
    "create_comment",
    "update_comment",
    "delete_comment",
    "list_comment_replies",
    "get_comment_reply",
    "create_comment_reply",
    "update_comment_reply",
    "delete_comment_reply",
    "search_nodes",
    "list_changes",
    "get_changes_start_page_token",
    "watch_changes",
    "watch_node",
    "list_watch_channels",
    "get_watch_channel",
    "stop_watch_channel",
    "presign_upload_part",
    "complete_upload_session",
    "abort_upload_session",
    "create_download_url",
    "resolve_download_token",
    "create_download_package",
    "resolve_download_package_url",
}


def apply_replacements(content: str) -> str:
    replacements = [
        (
            r"let tenant_id = require_query_value\(query\.tenant_id, \"tenantId\"\)\?;",
            "let tenant_id = ctx.resolve_tenant_id(query.tenant_id)?;",
        ),
        (
            r"let tenant_id = require_body_value\(payload\.tenant_id, \"tenantId\"\)\?;",
            "let tenant_id = ctx.resolve_tenant_id(payload.tenant_id)?;",
        ),
        (
            r"let tenant_id = require_body_value\(query\.tenant_id, \"tenantId\"\)\?;",
            "let tenant_id = ctx.resolve_tenant_id(query.tenant_id)?;",
        ),
        (
            r"let subject_type = require_query_value\(query\.subject_type, \"subjectType\"\)\?\;\n"
            r"\s*let subject_id = require_query_value\(query\.subject_id, \"subjectId\"\)\?;",
            "let (subject_type, subject_id) = ctx.resolve_subject(query.subject_type, query.subject_id)?;",
        ),
        (
            r"let subject_type = require_body_value\(payload\.subject_type, \"subjectType\"\)\?\;\n"
            r"\s*let subject_id = require_body_value\(payload\.subject_id, \"subjectId\"\)\?;",
            "let (subject_type, subject_id) = ctx.resolve_subject(payload.subject_type, payload.subject_id)?;",
        ),
        (
            r"let operator_id = payload\n\s*\.operator_id\n\s*\.unwrap_or_else\(\|\| \"operator-unset\"\.to_string\(\)\);",
            "let operator_id = ctx.resolve_operator_id(payload.operator_id)?;",
        ),
        (
            r"let operator_id = query\n\s*\.operator_id\n\s*\.unwrap_or_else\(\|\| \"operator-unset\"\.to_string\(\)\);",
            "let operator_id = ctx.resolve_operator_id(query.operator_id)?;",
        ),
    ]
    for pattern, repl in replacements:
        content = re.sub(pattern, repl, content)
    return content


def add_extension_to_handlers(content: str) -> str:
    lines = content.splitlines(keepends=True)
    out: list[str] = []
    idx = 0
    while idx < len(lines):
        line = lines[idx]
        out.append(line)
        match = re.match(r"async fn (\w+)\(", line)
        if match and match.group(1) in ROUTE_HANDLERS:
            window = "".join(lines[idx : idx + 10])
            if (
                "State(state): State<AppState>" in window
                and "Extension(ctx): Extension<DriveRequestContext>" not in window
            ):
                j = idx + 1
                while j < len(lines):
                    out.append(lines[j])
                    if "State(state): State<AppState>" in lines[j]:
                        out.append(
                            "    Extension(ctx): Extension<DriveRequestContext>,\n"
                        )
                        idx = j
                        break
                    j += 1
        idx += 1
    return "".join(out)


def patch_list_spaces(content: str) -> str:
    old = """    let tenant_id = match query.tenant_id {
        Some(tenant_id) => tenant_id.trim().to_string(),
        None => {
            sdkwork_drive_observability::observe_route!(
                event = events::APP_SPACES_LIST,
                result = "err",
                latency_ms = elapsed_ms(started),
                error_kind = error_kinds::VALIDATION,
                filter_has_owner_subject_type = filter_has_owner_subject_type,
                filter_has_owner_subject_id = filter_has_owner_subject_id
            );
            return Err(problem(
                StatusCode::BAD_REQUEST,
                "validation failed",
                "tenantId is required",
                "drive.validation.tenant_id_required",
            ));
        }
    };"""
    new = """    let tenant_id = match ctx.resolve_tenant_id(query.tenant_id) {
        Ok(tenant_id) => tenant_id,
        Err(error) => {
            sdkwork_drive_observability::observe_route!(
                event = events::APP_SPACES_LIST,
                result = "err",
                latency_ms = elapsed_ms(started),
                error_kind = error_kinds::VALIDATION,
                filter_has_owner_subject_type = filter_has_owner_subject_type,
                filter_has_owner_subject_id = filter_has_owner_subject_id
            );
            return Err(error);
        }
    };"""
    return content.replace(old, new)


def patch_match_tenant_handlers(content: str) -> str:
    content = re.sub(
        r"let tenant_id = match require_query_value\(query\.tenant_id, \"tenantId\"\) \{\n"
        r"        Ok\(tenant_id\) => tenant_id,\n"
        r"        Err\(error\) => \{",
        "let tenant_id = match ctx.resolve_tenant_id(query.tenant_id) {\n"
        "        Ok(tenant_id) => tenant_id,\n"
        "        Err(error) => {",
        content,
    )
    content = re.sub(
        r"let tenant_id = match require_body_value\(payload\.tenant_id, \"tenantId\"\) \{\n"
        r"        Ok\(tenant_id\) => tenant_id,\n"
        r"        Err\(error\) => \{",
        "let tenant_id = match ctx.resolve_tenant_id(payload.tenant_id) {\n"
        "        Ok(tenant_id) => tenant_id,\n"
        "        Err(error) => {",
        content,
    )
    return content


def patch_resolve_download_token(content: str) -> str:
    old = """    let tenant_id = match query.tenant_id {
        Some(tenant_id) => tenant_id.trim().to_string(),
        None => {
            sdkwork_drive_observability::observe_route!(
                event = events::APP_DOWNLOADS_RESOLVE,
                result = "err",
                latency_ms = elapsed_ms(started),
                error_kind = error_kinds::VALIDATION
            );
            return Err(problem(
                StatusCode::BAD_REQUEST,
                "validation failed",
                "tenantId is required",
                "drive.validation.tenant_id_required",
            ));
        }
    };"""
    new = """    let tenant_id = match ctx.resolve_tenant_id(query.tenant_id) {
        Ok(tenant_id) => tenant_id,
        Err(error) => {
            sdkwork_drive_observability::observe_route!(
                event = events::APP_DOWNLOADS_RESOLVE,
                result = "err",
                latency_ms = elapsed_ms(started),
                error_kind = error_kinds::VALIDATION
            );
            return Err(error);
        }
    };"""
    return content.replace(old, new)


def patch_create_space(content: str) -> str:
    return content.replace(
        "            tenant_id: payload.tenant_id,",
        "            tenant_id: ctx.resolve_tenant_id(payload.tenant_id)?,",
    ).replace(
        "            operator_id: payload.operator_id,",
        "            operator_id: ctx.resolve_operator_id(payload.operator_id)?,",
        1,
    )


def patch_file(content: Path) -> None:
    content_str = file.read_text(encoding="utf-8")
    content_str = apply_replacements(content_str)
    content_str = add_extension_to_handlers(content_str)
    if file == ROUTES:
        content_str = patch_list_spaces(content_str)
        content_str = patch_match_tenant_handlers(content_str)
        content_str = patch_resolve_download_token(content_str)
        content_str = patch_create_space(content_str)
    file.write_text(content_str, encoding="utf-8")


if __name__ == "__main__":
    for file in (ROUTES, DOWNLOAD):
        patch_file(file)
        print(f"patched {file}")
