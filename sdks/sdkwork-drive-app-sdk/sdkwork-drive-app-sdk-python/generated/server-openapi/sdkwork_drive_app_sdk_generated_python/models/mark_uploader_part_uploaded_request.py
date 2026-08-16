from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class MarkUploaderPartUploadedRequest:
    upload_session_id: str
    offset_bytes: int
    size_bytes: int
    etag: str
    checksum_sha256hex: Optional[str] = None
    uploaded_at_epoch_ms: Optional[int] = None
