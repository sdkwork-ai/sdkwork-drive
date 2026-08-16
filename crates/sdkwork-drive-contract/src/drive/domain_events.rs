//! Canonical Drive domain event names (`drive.*`) per DRIVE_SPEC and EVENT_SPEC.

/// Space lifecycle events.
pub mod space {
    pub const CREATED: &str = "drive.space.created";
    pub const UPDATED: &str = "drive.space.updated";
    pub const DELETED: &str = "drive.space.deleted";
}

/// Node lifecycle and mutation events.
pub mod node {
    pub const CREATED: &str = "drive.node.created";
    pub const UPDATED: &str = "drive.node.updated";
    pub const MOVED: &str = "drive.node.moved";
    pub const COPIED: &str = "drive.node.copied";
    pub const DELETED: &str = "drive.node.deleted";
    pub const TRASHED: &str = "drive.node.trashed";
    pub const RESTORED: &str = "drive.node.restored";
    pub const VERSION_COMMITTED_V1: &str = "drive.node.version.committed.v1";
    pub const PATH_CHANGED_V1: &str = "drive.node.path.changed.v1";
    pub const ELIGIBILITY_CHANGED_V1: &str = "drive.node.eligibility.changed.v1";
    pub const DELETED_V1: &str = "drive.node.deleted.v1";
}

/// WebsiteRoot generation lifecycle events.
pub mod website_root {
    pub const GENERATION_CHANGED_V1: &str = "drive.website_root.generation.changed.v1";
}

/// Node property mutation events.
pub mod node_property {
    pub const SET: &str = "drive.node_property.set";
    pub const DELETED: &str = "drive.node_property.deleted";
}

/// Node label mutation events.
pub mod node_label {
    pub const APPLIED: &str = "drive.node_label.applied";
    pub const REMOVED: &str = "drive.node_label.removed";
}

/// Favorite mutation events.
pub mod favorite {
    pub const CREATED: &str = "drive.favorite.created";
    pub const DELETED: &str = "drive.favorite.deleted";
}

/// Trash maintenance events.
pub mod trash {
    pub const EMPTIED: &str = "drive.trash.emptied";
}

/// File version mutation events.
pub mod file_version {
    pub const RESTORED: &str = "drive.file_version.restored";
    pub const DELETED: &str = "drive.file_version.deleted";
}

/// Node permission mutation events.
pub mod permission {
    pub const CREATED: &str = "drive.permission.created";
    pub const UPDATED: &str = "drive.permission.updated";
    pub const DELETED: &str = "drive.permission.deleted";
}

/// Share link mutation events.
pub mod share_link {
    pub const CREATED: &str = "drive.share_link.created";
    pub const UPDATED: &str = "drive.share_link.updated";
    pub const REVOKED: &str = "drive.share_link.revoked";
    pub const CLAIMED: &str = "drive.share_link.claimed";
}

/// Comment mutation events.
pub mod comment {
    pub const CREATED: &str = "drive.comment.created";
    pub const UPDATED: &str = "drive.comment.updated";
    pub const DELETED: &str = "drive.comment.deleted";
}

/// Comment reply mutation events.
pub mod comment_reply {
    pub const CREATED: &str = "drive.comment_reply.created";
    pub const UPDATED: &str = "drive.comment_reply.updated";
    pub const DELETED: &str = "drive.comment_reply.deleted";
}

/// Watch channel mutation events.
pub mod watch_channel {
    pub const CREATED: &str = "drive.watch_channel.created";
    pub const STOPPED: &str = "drive.watch_channel.stopped";
}

/// Upload session events (DRIVE_SPEC §13).
pub mod upload_session {
    pub const CREATED: &str = "drive.upload_session.created";
    pub const COMPLETED: &str = "drive.upload_session.completed";
    pub const ABORTED: &str = "drive.upload_session.aborted";
}

/// Uploader pipeline events (DRIVE_SPEC §13).
pub mod uploader {
    pub const UPLOAD_PREPARED: &str = "drive.uploader.upload_prepared";
    pub const PART_UPLOADED: &str = "drive.uploader.part_uploaded";
    pub const UPLOAD_COMPLETED: &str = "drive.uploader.upload_completed";
    pub const CONTENT_EXPIRED: &str = "drive.uploader.content_expired";
}

/// Download grant events (DRIVE_SPEC §13).
pub mod download_grant {
    pub const CREATED: &str = "drive.download_grant.created";
}

/// Storage object events (DRIVE_SPEC §13).
pub mod object {
    pub const QUARANTINED: &str = "drive.object.quarantined";
}

/// Archive extraction events.
pub mod archive {
    pub const FOLDER_CREATED: &str = "drive.archive.folder_created";
    pub const ENTRY_EXTRACTED: &str = "drive.archive.entry_extracted";
}

/// Backend admin audit actions persisted to `dr_drive_audit_event`.
pub mod admin_audit {
    /// Tenant label taxonomy mutations (backend-api).
    pub mod label {
        pub const CREATED: &str = "drive.label.created";
        pub const UPDATED: &str = "drive.label.updated";
        pub const DELETED: &str = "drive.label.deleted";
    }

    /// Storage provider administration mutations (backend-api).
    pub mod storage_provider {
        pub const READ: &str = "drive.storage_provider.read";
        pub const CREATED: &str = "drive.storage_provider.created";
        pub const UPDATED: &str = "drive.storage_provider.updated";
        pub const CREDENTIALS_ROTATED: &str = "drive.storage_provider.credentials_rotated";
        pub const ACTIVATED: &str = "drive.storage_provider.activated";
        pub const DEACTIVATED: &str = "drive.storage_provider.deactivated";
        pub const STATUS_CHANGED: &str = "drive.storage_provider.status_changed";
        pub const TESTED: &str = "drive.storage_provider.tested";
        pub const DELETED: &str = "drive.storage_provider.deleted";
        pub const BUCKET_CREATED: &str = "drive.storage_provider.bucket_created";
        pub const BUCKET_DELETED: &str = "drive.storage_provider.bucket_deleted";
        pub const OBJECT_PUT: &str = "drive.storage_provider.object_put";
        pub const OBJECT_DELETED: &str = "drive.storage_provider.object_deleted";
        pub const OBJECT_COPIED: &str = "drive.storage_provider.object_copied";
    }

    /// Storage provider kind registry mutations (backend-api).
    pub mod storage_provider_kind {
        pub const INITIALIZED: &str = "drive.storage_provider_kind.initialized";
        pub const ENABLED: &str = "drive.storage_provider_kind.enabled";
        pub const DISABLED: &str = "drive.storage_provider_kind.disabled";
    }

    /// Default storage provider binding mutations (backend-api).
    pub mod storage_provider_binding {
        pub const DEFAULT_SET: &str = "drive.storage_provider_binding.default_set";
        pub const DEFAULT_DELETED: &str = "drive.storage_provider_binding.default_deleted";
    }

    /// Maintenance job executions (backend-api).
    pub mod maintenance {
        pub const OBJECT_SWEEP_EXECUTED: &str = "drive.maintenance.object_sweep.executed";
        pub const OBJECT_SWEEP_FAILED: &str = "drive.maintenance.object_sweep.failed";
        pub const UPLOAD_SESSION_SWEEP_EXECUTED: &str =
            "drive.maintenance.upload_session_sweep.executed";
        pub const UPLOAD_SESSION_SWEEP_FAILED: &str =
            "drive.maintenance.upload_session_sweep.failed";
        pub const EXPIRED_UPLOAD_CONTENT_SWEEP_EXECUTED: &str =
            "drive.maintenance.expired_upload_content_sweep.executed";
        pub const EXPIRED_UPLOAD_CONTENT_SWEEP_FAILED: &str =
            "drive.maintenance.expired_upload_content_sweep.failed";
        pub const ABANDONED_UPLOAD_TASK_SWEEP_EXECUTED: &str =
            "drive.maintenance.abandoned_upload_task_sweep.executed";
        pub const ABANDONED_UPLOAD_TASK_SWEEP_FAILED: &str =
            "drive.maintenance.abandoned_upload_task_sweep.failed";
    }

    /// Tenant quota policy mutations (backend-api).
    pub mod quota {
        pub const UPDATED: &str = "drive.quota.updated";
    }

    /// Server sandbox volume administration mutations (backend-api).
    pub mod sandbox_volume {
        pub const CREATED: &str = "drive.sandbox_volume.created";
        pub const UPDATED: &str = "drive.sandbox_volume.updated";
        pub const DELETED: &str = "drive.sandbox_volume.deleted";
    }

    /// Explicit server sandbox grant administration mutations (backend-api).
    pub mod sandbox_grant {
        pub const CREATED: &str = "drive.sandbox_grant.created";
        pub const UPDATED: &str = "drive.sandbox_grant.updated";
        pub const DELETED: &str = "drive.sandbox_grant.deleted";
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_EVENT_NAMES: &[&str] = &[
        space::CREATED,
        space::UPDATED,
        space::DELETED,
        node::CREATED,
        node::UPDATED,
        node::MOVED,
        node::COPIED,
        node::DELETED,
        node::TRASHED,
        node::RESTORED,
        node::VERSION_COMMITTED_V1,
        node::PATH_CHANGED_V1,
        node::ELIGIBILITY_CHANGED_V1,
        node::DELETED_V1,
        node_property::SET,
        node_property::DELETED,
        node_label::APPLIED,
        node_label::REMOVED,
        favorite::CREATED,
        favorite::DELETED,
        trash::EMPTIED,
        file_version::RESTORED,
        file_version::DELETED,
        permission::CREATED,
        permission::UPDATED,
        permission::DELETED,
        share_link::CREATED,
        share_link::UPDATED,
        share_link::REVOKED,
        share_link::CLAIMED,
        comment::CREATED,
        comment::UPDATED,
        comment::DELETED,
        comment_reply::CREATED,
        comment_reply::UPDATED,
        comment_reply::DELETED,
        watch_channel::CREATED,
        watch_channel::STOPPED,
        upload_session::CREATED,
        upload_session::COMPLETED,
        upload_session::ABORTED,
        uploader::UPLOAD_PREPARED,
        uploader::PART_UPLOADED,
        uploader::UPLOAD_COMPLETED,
        uploader::CONTENT_EXPIRED,
        download_grant::CREATED,
        object::QUARANTINED,
        archive::FOLDER_CREATED,
        archive::ENTRY_EXTRACTED,
    ];

    const ALL_ADMIN_AUDIT_ACTIONS: &[&str] = &[
        admin_audit::label::CREATED,
        admin_audit::label::UPDATED,
        admin_audit::label::DELETED,
        admin_audit::storage_provider::READ,
        admin_audit::storage_provider::CREATED,
        admin_audit::storage_provider::UPDATED,
        admin_audit::storage_provider::CREDENTIALS_ROTATED,
        admin_audit::storage_provider::ACTIVATED,
        admin_audit::storage_provider::DEACTIVATED,
        admin_audit::storage_provider::STATUS_CHANGED,
        admin_audit::storage_provider::TESTED,
        admin_audit::storage_provider::DELETED,
        admin_audit::storage_provider::BUCKET_CREATED,
        admin_audit::storage_provider::BUCKET_DELETED,
        admin_audit::storage_provider::OBJECT_DELETED,
        admin_audit::storage_provider::OBJECT_COPIED,
        admin_audit::storage_provider_kind::INITIALIZED,
        admin_audit::storage_provider_kind::ENABLED,
        admin_audit::storage_provider_kind::DISABLED,
        admin_audit::storage_provider_binding::DEFAULT_SET,
        admin_audit::storage_provider_binding::DEFAULT_DELETED,
        admin_audit::maintenance::OBJECT_SWEEP_EXECUTED,
        admin_audit::maintenance::OBJECT_SWEEP_FAILED,
        admin_audit::maintenance::UPLOAD_SESSION_SWEEP_EXECUTED,
        admin_audit::maintenance::UPLOAD_SESSION_SWEEP_FAILED,
        admin_audit::maintenance::EXPIRED_UPLOAD_CONTENT_SWEEP_EXECUTED,
        admin_audit::maintenance::EXPIRED_UPLOAD_CONTENT_SWEEP_FAILED,
        admin_audit::maintenance::ABANDONED_UPLOAD_TASK_SWEEP_EXECUTED,
        admin_audit::maintenance::ABANDONED_UPLOAD_TASK_SWEEP_FAILED,
        admin_audit::quota::UPDATED,
        admin_audit::sandbox_volume::CREATED,
        admin_audit::sandbox_volume::UPDATED,
        admin_audit::sandbox_volume::DELETED,
        admin_audit::sandbox_grant::CREATED,
        admin_audit::sandbox_grant::UPDATED,
        admin_audit::sandbox_grant::DELETED,
    ];

    #[test]
    fn all_admin_audit_actions_use_drive_prefix() {
        for action in ALL_ADMIN_AUDIT_ACTIONS {
            assert!(
                action.starts_with("drive."),
                "admin audit action must use drive. prefix: {action}"
            );
        }
    }

    #[test]
    fn all_domain_event_names_use_drive_prefix() {
        for event_name in ALL_EVENT_NAMES {
            assert!(
                event_name.starts_with("drive."),
                "event name must use drive. prefix: {event_name}"
            );
        }
    }
}
