from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class UploaderUploadPart:
    id: str
    upload_item_id: str
    upload_session_id: str
    part_no: int
    offset_bytes: int
    size_bytes: int
    etag: str
    status: str
    retry_count: int
    tenant_id: Optional[str] = None
    checksum_sha256hex: Optional[str] = None
    uploaded_at_epoch_ms: Optional[int] = None
