from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class CreateFileRequest:
    id: str
    space_id: str
    node_name: str
    upload_session_id: str
    idempotency_key: str
    expires_at_epoch_ms: int
    parent_node_id: Optional[str] = None
    bucket: Optional[str] = None
    object_key: Optional[str] = None
