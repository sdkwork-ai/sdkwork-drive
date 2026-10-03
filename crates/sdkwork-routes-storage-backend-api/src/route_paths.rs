use axum::extract::DefaultBodyLimit;
use axum::routing::{get, patch, post};
use axum::Router;

use crate::handlers::*;
use crate::object_handlers::MAX_OBJECT_CONTENT_REQUEST_BYTES;
use crate::state::AdminStorageState;

pub(crate) fn storage_drive_routes(prefix: &str) -> Router<AdminStorageState> {
    Router::new()
        /*
         * 对象内容读写单独一层：它必须显式声明请求体上限。
         *
         * 内容接口把对象字节放在 JSON 里以 base64 传输，合法请求体约 11.2 MB（8 MiB 对象）。
         * 不声明时用的是 axum 的默认上限 2 MB——超过 1.4 MB 的文件根本进不到处理器，网关
         * 或框架再把那个 413 归一化成一句没有业务细节的 `Payload too large`，看起来像
         * "对象不存在/超限"，实际是路由没声明自己收多大的包。
         */
        .merge(
            Router::new()
                .route(
                    &format!(
                        "{prefix}/drive/storage/providers/{{provider_id}}/object-contents/{{*object_key}}"
                    ),
                    get(read_storage_provider_object_content)
                        .put(write_storage_provider_object_content),
                )
                .layer(DefaultBodyLimit::max(MAX_OBJECT_CONTENT_REQUEST_BYTES)),
        )
        /*
         * 预签名分片上传：开启 → 签发分片授权 → 完成/中止。
         *
         * 四个操作都只传"元数据"（对象 key、uploadId、分片号与 ETag），字节由客户端直传厂商，
         * 所以它们落在默认请求体上限内，不需要内容接口那层大包上限。uploadId 放在请求体里
         * 而不是路径上：它是厂商的不透明令牌，可能包含斜杠等字符，塞进路径段需要额外转义，
         * 也容易在网关的路径规范化里被改写。
         */
        .route(
            &format!(
                "{prefix}/drive/storage/providers/{{provider_id}}/objects/multipart-uploads"
            ),
            post(create_storage_provider_object_multipart_upload),
        )
        .route(
            &format!(
                "{prefix}/drive/storage/providers/{{provider_id}}/objects/multipart-uploads/parts"
            ),
            post(presign_storage_provider_object_upload_parts),
        )
        .route(
            &format!(
                "{prefix}/drive/storage/providers/{{provider_id}}/objects/multipart-uploads/complete"
            ),
            post(complete_storage_provider_object_multipart_upload),
        )
        .route(
            &format!(
                "{prefix}/drive/storage/providers/{{provider_id}}/objects/multipart-uploads/abort"
            ),
            post(abort_storage_provider_object_multipart_upload),
        )
        .route(
            &format!("{prefix}/drive/storage/providers"),
            get(list_storage_providers).post(create_storage_provider),
        )
        .route(
            &format!("{prefix}/drive/storage/provider-kinds"),
            get(list_storage_provider_kinds).post(initialize_storage_provider_kinds),
        )
        .route(
            &format!("{prefix}/drive/storage/provider-kinds/{{provider_kind}}"),
            patch(set_storage_provider_kind_enabled),
        )
        .route(
            &format!("{prefix}/drive/storage/provider-accounts"),
            get(list_storage_provider_accounts).post(create_storage_provider_account),
        )
        .route(
            &format!("{prefix}/drive/storage/provider-account-defaults"),
            post(initialize_storage_provider_account_defaults),
        )
        .route(
            &format!("{prefix}/drive/storage/providers/{{provider_id}}"),
            get(get_storage_provider)
                .patch(update_storage_provider)
                .delete(delete_storage_provider),
        )
        .route(
            &format!("{prefix}/drive/storage/providers/{{provider_id}}/capabilities"),
            get(get_storage_provider_capabilities),
        )
        .route(
            &format!("{prefix}/drive/storage/providers/{{provider_id}}/test"),
            post(test_storage_provider),
        )
        .route(
            &format!("{prefix}/drive/storage/providers/{{provider_id}}/activate"),
            post(activate_storage_provider),
        )
        .route(
            &format!("{prefix}/drive/storage/providers/{{provider_id}}/deactivate"),
            post(deactivate_storage_provider),
        )
        .route(
            &format!("{prefix}/drive/storage/providers/{{provider_id}}/credentials/rotate"),
            post(rotate_storage_provider_credentials),
        )
        .route(
            &format!("{prefix}/drive/storage/providers/{{provider_id}}/bucket"),
            get(head_storage_provider_bucket)
                .put(create_storage_provider_bucket)
                .delete(delete_storage_provider_bucket),
        )
        .route(
            &format!("{prefix}/drive/storage/providers/{{provider_id}}/buckets"),
            get(list_storage_provider_buckets),
        )
        .route(
            &format!("{prefix}/drive/storage/providers/{{provider_id}}/objects"),
            get(list_storage_provider_objects),
        )
        .route(
            &format!("{prefix}/drive/storage/providers/{{provider_id}}/objects/copy"),
            post(copy_storage_provider_object),
        )
        .route(
            &format!("{prefix}/drive/storage/providers/{{provider_id}}/objects/{{*object_key}}"),
            get(head_storage_provider_object).delete(delete_storage_provider_object),
        )
        .route(
            &format!("{prefix}/drive/storage/bindings/default"),
            get(get_default_storage_provider_binding)
                .put(set_default_storage_provider_binding)
                .delete(delete_default_storage_provider_binding),
        )
        .route(
            &format!("{prefix}/drive/storage/overview"),
            get(get_storage_overview),
        )
        .route(
            &format!("{prefix}/drive/storage/bindings"),
            get(list_storage_provider_bindings),
        )
        // Cross-provider migration. The batch endpoint (`/run`) is the only
        // writable one: a client drives the loop and polls `/migrations/{id}`
        // until `completed` is true. Keeping the copy work behind one verb means
        // there is no second code path that could re-point a half-migrated
        // tenant.
        .route(
            &format!("{prefix}/drive/storage/migrations"),
            get(list_storage_migrations).post(plan_storage_migration),
        )
        .route(
            &format!("{prefix}/drive/storage/migrations/{{migration_id}}"),
            get(get_storage_migration),
        )
        .route(
            &format!("{prefix}/drive/storage/migrations/{{migration_id}}/run"),
            post(run_storage_migration),
        )
        .route(
            &format!("{prefix}/drive/storage/migrations/{{migration_id}}/items"),
            get(list_storage_migration_items),
        )
        .route(
            &format!("{prefix}/drive/storage/migrations/{{migration_id}}/cancel"),
            post(cancel_storage_migration),
        )
}
