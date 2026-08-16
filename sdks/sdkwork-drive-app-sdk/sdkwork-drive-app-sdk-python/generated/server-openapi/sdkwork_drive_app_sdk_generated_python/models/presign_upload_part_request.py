from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class PresignUploadPartRequest:
    upload_id: Optional[str] = None
    requested_ttl_seconds: Optional[int] = None
