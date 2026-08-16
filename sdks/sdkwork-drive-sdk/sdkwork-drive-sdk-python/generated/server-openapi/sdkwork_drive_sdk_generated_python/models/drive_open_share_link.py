from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any

if TYPE_CHECKING:
    from .open_node import OpenNode


@dataclass
class DriveOpenShareLink:
    id: str
    tenant_id: str
    role: str
    download_count: int
    node: OpenNode
    expires_at_epoch_ms: Optional[int] = None
    download_limit: Optional[int] = None
    access_code_required: Optional[bool] = None
