from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class DownloadPackage:
    id: str
    tenant_id: str
    package_name: str
    state: str
    storage_provider_id: str
    bucket: str
    archive_object_key: str
    content_type: str
    file_count: int
    total_bytes: int
    archive_size_bytes: int
    expires_at_epoch_ms: int
    created_by: str
    updated_by: str
    created_at: str
    updated_at: str
    error_message: Optional[str] = None
