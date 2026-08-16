from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class PresignedUploadPart:
    upload_url: str
    expires_at_epoch_ms: int
    method: str
    headers: Dict[str, str]
    part_no: int
    upload_id: str
