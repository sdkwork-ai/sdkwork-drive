from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any

if TYPE_CHECKING:
    from .uploader_retention_request import UploaderRetentionRequest


@dataclass
class PrepareUploaderUploadRequest:
    id: str
    task_id: str
    app_resource_type: str
    app_resource_id: str
    file_fingerprint: str
    original_file_name: str
    content_type: str
    content_length: int
    chunk_size_bytes: int
    upload_profile_code: Optional[str] = None
    space_id: Optional[str] = None
    parent_node_id: Optional[str] = None
    retention: Optional[UploaderRetentionRequest] = None
    now_epoch_ms: Optional[int] = None
    scene: Optional[str] = None
    source: Optional[str] = None
    share_token: Optional[str] = None
