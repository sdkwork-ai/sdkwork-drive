from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any

if TYPE_CHECKING:
    from .download_package_item import DownloadPackageItem


@dataclass
class DownloadPackageResponse:
    id: str
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
    download_url: str
    signed_source_url: str
    method: str
    items: List[DownloadPackageItem]
    tenant_id: Optional[str] = None
