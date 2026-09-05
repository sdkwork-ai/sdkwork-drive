from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class DriveShareLink:
    id: str
    node_id: str
    role: str
    download_count: str
    lifecycle_status: str
    version: str
    tenant_id: Optional[str] = None
    expires_at_epoch_ms: Optional[str] = None
    download_limit: Optional[str] = None
    access_code_required: Optional[bool] = None
