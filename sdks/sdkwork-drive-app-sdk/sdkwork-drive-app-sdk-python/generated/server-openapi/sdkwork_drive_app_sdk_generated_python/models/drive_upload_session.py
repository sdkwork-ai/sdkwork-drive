from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class DriveUploadSession:
    id: str
    space_id: str
    node_id: str
    bucket: str
    object_key: str
    idempotency_key: str
    state: str
    expires_at_epoch_ms: int
    version: int
    storage_provider_id: str
    storage_upload_id: str
    tenant_id: Optional[str] = None
