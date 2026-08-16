from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class UploaderUploadItem:
    id: str
    task_id: str
    actor_type: str
    actor_id: str
    app_id: str
    app_resource_type: str
    app_resource_id: str
    upload_profile_code: str
    file_fingerprint: str
    space_id: str
    node_id: str
    original_file_name: str
    content_type: str
    content_type_group: str
    content_length: int
    chunk_size_bytes: int
    total_parts: int
    uploaded_parts_count: int
    uploaded_bytes: int
    status: str
    retention_mode: str
    cleanup_status: str
    post_process_status: str
    tenant_id: Optional[str] = None
    organization_id: Optional[str] = None
    user_id: Optional[str] = None
    upload_session_id: Optional[str] = None
    storage_provider_id: Optional[str] = None
    storage_upload_id: Optional[str] = None
    file_extension: Optional[str] = None
    detected_content_type: Optional[str] = None
    checksum_sha256hex: Optional[str] = None
    retention_expires_at_epoch_ms: Optional[int] = None
    cleanup_action: Optional[str] = None
    hard_delete_after_epoch_ms: Optional[int] = None
    scene: Optional[str] = None
    source: Optional[str] = None
