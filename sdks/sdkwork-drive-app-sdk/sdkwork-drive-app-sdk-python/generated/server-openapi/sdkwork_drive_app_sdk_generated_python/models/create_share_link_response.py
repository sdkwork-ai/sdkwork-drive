from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class CreateShareLinkResponse:
    id: str
    node_id: str
    role: str
    download_count: int
    lifecycle_status: str
    version: int
    token: str
    tenant_id: Optional[str] = None
    expires_at_epoch_ms: Optional[int] = None
    download_limit: Optional[int] = None
    access_code_required: Optional[bool] = None
    access_code: Optional[str] = None
