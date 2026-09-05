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
    file_count: str
    total_bytes: str
    archive_size_bytes: str
    expires_at_epoch_ms: str
    created_by: str
    updated_by: str
    created_at: str
    updated_at: str
    error_message: Optional[str] = None
